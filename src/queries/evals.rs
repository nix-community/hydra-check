//! A module that formats the details of one (or multiple) evaluation(s),
//! from urls such as <https://hydra.nixos.org/eval/1809808>.
//!
//! This is a long pile of spaghetti that serves a single purpose.
//! Splitting it into separate modules would also be clumsy as we then need
//! to add a bunch of `pub(super)` identifiers to the various data structures.

use anyhow::{anyhow, bail};
use colored::Colorize;
use indexmap::IndexMap;
use log::{info, warn};
use regex::Regex;
use scraper::Html;
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;
use std::fmt::Display;

#[cfg(test)]
use insta::assert_snapshot;

use crate::{
    constants::HYDRA_CHECK_HOST_URL, BuildStatus, EvalInput, Evaluation, FetchHydraReport,
    ResolvedArgs, SoupFind, StatusIcon,
};

#[skip_serializing_none]
#[derive(Serialize, Clone)]
struct EvalInputChanges {
    input: String,
    description: String,
    url: Option<String>,
    revs: Option<(String, String)>,
    short_revs: Option<(String, String)>,
}

impl Display for EvalInputChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(&self).expect("EvalInput should be serialized into json");
        let json: Value = serde_json::from_str(&json).unwrap();
        let strings: Vec<_> = ["input", "description", "url", "revs"]
            .iter()
            .filter_map(|key| match &json[key] {
                Value::Null => None,
                // unquote the string:
                Value::String(value) => {
                    let key = match *key {
                        "input" => "changed_input",
                        "description" => "changes",
                        k => k,
                    };
                    Some(format!("{}: {}", key.bold(), value))
                }
                Value::Array(vec) => {
                    let texts: Vec<_> = vec.iter().filter_map(|x| x.as_str()).collect();
                    Some(format!("{}: {}", key.bold(), texts.join(" -> ")))
                }
                value => Some(format!("{}: {}", key.bold(), value)),
            })
            .collect();
        write!(f, "{}", strings.join("\n"))
    }
}

#[test]
fn format_input_changes() {
    let input_changes = EvalInputChanges {
        input: "nixpkgs".into(),
        description: "8c4dc69b9732 to 1e9e641a3fc1".into(),
        url: Some("https://hydra.nixos.org/api/scmdiff?blah/blah/blah".into()),
        revs: Some((
            "8c4dc69b9732f6bbe826b5fbb32184987520ff26".into(),
            "1e9e641a3fc1b22fbdb823a99d8ff96692cc4fba".into(),
        )),
        short_revs: Some(("8c4dc69b9732".into(), "1e9e641a3fc1".into())),
    };
    assert_snapshot!(input_changes.to_string(), @r#"
        [1mchanged_input[0m: nixpkgs
        [1mchanges[0m: 8c4dc69b9732 to 1e9e641a3fc1
        [1murl[0m: https://hydra.nixos.org/api/scmdiff?blah/blah/blah
        [1mrevs[0m: 8c4dc69b9732f6bbe826b5fbb32184987520ff26 -> 1e9e641a3fc1b22fbdb823a99d8ff96692cc4fba
    "#);
}

impl EvalInputChanges {
    #[allow(clippy::similar_names)]
    fn from_html(doc: &Html) -> anyhow::Result<Vec<Self>> {
        let tables = doc.find_all("div#tabs-inputs table");
        #[allow(clippy::redundant_closure_for_method_calls)]
        let err = || {
            anyhow!(
                "could not parse the table of changed inputs in {:?}",
                tables.iter().map(|x| x.html()).collect::<Vec<_>>()
            )
        };
        // table of input changes:
        let table = tables.get(1).ok_or_else(err)?;
        let thead: Vec<String> = table
            .find("tr")?
            .find_all("th")
            .iter()
            .map(|x| x.text().collect())
            .collect();
        if !thead
            .iter()
            .all(|x| x.trim().contains("Input") || x.trim().contains("Changes"))
        {
            bail!(err());
        }
        let tbody = table.find_all("tr");
        let rows = tbody.get(1..).ok_or_else(err)?;
        let mut input_changes = Vec::new();

        let regex_short_revs = Regex::new("^([0-9a-z]+) to ([0-9a-z]+)$").unwrap();
        let [regex_rev1, regex_rev2] = ["rev1", "rev2"].map(|rev_id| {
            let re = format!("^.*{rev_id}=([0-9a-z]+).*$");
            Regex::new(&re).unwrap()
        });

        for row in rows {
            let columns = row.find_all("td");
            let mut columns = columns.iter();
            let input: String = columns.next().ok_or_else(err)?.text().collect();
            let input = input.trim().to_string();

            let changes = columns.next().ok_or_else(err)?;
            let description: String = changes.text().collect();
            let description = description.trim().to_string();

            // the following entries are non-essential,
            // so we avoid using `?` for premature exits
            let url = changes
                .find("a")
                .ok()
                .and_then(|x| x.attr("href"))
                .map(str::to_string);

            let revs = if let Some(url) = &url {
                // note that the returned url is not deterministic:
                // the position of the query parameters may float around
                let [rev1, rev2] = [&regex_rev1, &regex_rev2].map(|regex_rev| {
                    match regex_rev.captures(url).map(|x| x.extract()) {
                        Some((_, [rev])) if !rev.is_empty() => Some(rev.to_string()),
                        _ => None,
                    }
                });

                match (rev1, rev2) {
                    (Some(rev1), Some(rev2)) => Some((rev1, rev2)),
                    _ => None,
                }
            } else {
                None
            };

            let short_revs = if description.is_empty() {
                None
            } else {
                match regex_short_revs.captures(&description).map(|x| x.extract()) {
                    Some((_, [rev1, rev2])) if (!rev1.is_empty()) && (!rev2.is_empty()) => {
                        Some((rev1.to_string(), rev2.to_string()))
                    }
                    _ => None,
                }
            };

            input_changes.push(EvalInputChanges {
                input,
                description,
                url,
                revs,
                short_revs,
            });
        }
        Ok(input_changes)
    }
}

#[derive(Serialize, Clone)]
struct EvalReport<'a> {
    #[serde(flatten)]
    eval: &'a Evaluation,
    url: String,
    inputs: Vec<EvalInput>,
    changes: Vec<EvalInputChanges>,
    aborted: Vec<BuildStatus>,
    now_fail: Vec<BuildStatus>,
    now_succeed: Vec<BuildStatus>,
    new: Vec<BuildStatus>,
    removed: Vec<BuildStatus>,
    still_fail: Vec<BuildStatus>,
    still_succeed: Vec<BuildStatus>,
    unfinished: Vec<BuildStatus>,
}

impl FetchHydraReport for EvalReport<'_> {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn finish_with_error(self, status: String) -> Self {
        Self {
            inputs: vec![EvalInput {
                name: Some(StatusIcon::Warning.to_string()),
                value: Some(status),
                ..Default::default()
            }],
            ..self
        }
    }
}

impl<'a> From<&'a Evaluation> for EvalReport<'a> {
    fn from(eval: &'a Evaluation) -> Self {
        let url = format!("{}/eval/{}", &*HYDRA_CHECK_HOST_URL, eval.id);
        let url = match &eval.filter {
            Some(x) => format!("{url}?filter={x}"),
            None => url,
        };
        Self {
            eval,
            url,
            inputs: vec![],
            changes: vec![],
            aborted: vec![],
            now_fail: vec![],
            now_succeed: vec![],
            new: vec![],
            removed: vec![],
            still_fail: vec![],
            still_succeed: vec![],
            unfinished: vec![],
        }
    }
}

impl EvalReport<'_> {
    fn parse_build_stats(&self, doc: &Html, selector: &str) -> anyhow::Result<Vec<BuildStatus>> {
        let err = || {
            anyhow!(
                "could not parse the table of build stats '{:?}' in {}",
                selector,
                doc.html()
            )
        };
        let tbody = match self.find_tbody(doc, selector) {
            Err(stat) => bail!("{:?}", stat.inputs.first().ok_or_else(err)?.value),
            Ok(tbody) => tbody,
        };
        BuildStatus::from_tbody(tbody)
    }

    fn fetch_and_read(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc, "div#tabs-inputs") {
            // inputs are essential information, so exit early if this fails:
            Err(stat) => return Ok(stat),
            Ok(tbody) => tbody,
        };
        let inputs = EvalInput::from_tbody(tbody, self.eval.id.to_string().as_str())?;
        let changes = EvalInputChanges::from_html(&doc).unwrap_or_else(|err| {
            warn!("{}\n{}", err, err.backtrace());
            vec![]
        });

        let [aborted, now_fail, now_succeed, new, removed, still_fail, still_succeed, unfinished] =
            [
                "aborted",
                "now-fail",
                "now-succeed",
                "new",
                "removed",
                "still-fail",
                "still-succeed",
                "unfinished",
            ]
            .map(|selector| {
                let selector = format!("div#tabs-{selector}");
                self.parse_build_stats(&doc, &selector)
                    .unwrap_or_else(|err| {
                        warn!("{}\n{}", err, err.backtrace());
                        vec![]
                    })
            });

        Ok(Self {
            inputs,
            changes,
            aborted,
            now_fail,
            now_succeed,
            new,
            removed,
            still_fail,
            still_succeed,
            unfinished,
            ..self
        })
    }
}

impl ResolvedArgs {
    pub(crate) fn fetch_and_print_evaluations(&self, evals: &[Evaluation]) -> anyhow::Result<bool> {
        let mut indexmap = IndexMap::new();
        let evals: Vec<_> = match evals.iter().any(|eval| eval.id == 0) {
            false => evals.to_owned(),
            true => {
                info!(
                    "querying the latest evaluation of --jobset '{}'",
                    self.jobset
                );
                let err = || {
                    anyhow!(
                        "could not find the latest evaluation for --jobset '{}'",
                        self.jobset
                    )
                };
                eprintln!();
                let id = self.fetch_and_print_jobset(false)?.ok_or_else(err)?;
                println!();
                evals
                    .iter()
                    .map(|eval| match &eval.id {
                        0 => Evaluation { id, ..eval.clone() },
                        _ => eval.clone(),
                    })
                    .collect()
            }
        };
        for (idx, eval) in evals.iter().enumerate() {
            let stat = EvalReport::from(eval);
            if self.url {
                println!("{}", stat.get_url());
                continue;
            }
            if !self.json {
                // print title first, then fetch
                if idx > 0 && !self.short {
                    println!(); // vertical whitespace
                }
                println!(
                    "Evaluation {}{} {}",
                    stat.eval.id.to_string().bold(),
                    match &stat.eval.filter {
                        Some(x) => format!(" filtered by '{}'", x.bold()),
                        None => "".into(),
                    },
                    format!("@ {}", stat.get_url()).dimmed(),
                );
            }
            let stat = stat.fetch_and_read()?;
            if self.json {
                indexmap.insert(&stat.eval.spec, stat);
                continue;
            }
            for entry in &stat.inputs {
                println!(); // vertical separation
                println!("{entry}");
            }
            for entry in &stat.changes {
                println!(); // vertical separation
                println!("{entry}");
            }
            if self.short {
                continue;
            }
            for (build_stats, prompt) in [
                (&stat.aborted, "Aborted / Timed out:".bold()),
                (&stat.now_fail, "Newly Failing:".bold()),
                (&stat.now_succeed, "Newly Succeeding:".bold()),
                (&stat.new, "New Jobs:".bold()),
                (&stat.removed, "Removed Jobs:".bold()),
                (&stat.still_fail, "Still Failing:".bold()),
                (&stat.still_succeed, "Still Succeeding:".bold()),
                (&stat.unfinished, "Queued Jobs:".bold()),
            ] {
                #[allow(clippy::uninlined_format_args)]
                if !build_stats.is_empty() {
                    println!();
                    println!("{}", prompt);
                    println!("{}", stat.format_table(false, build_stats));
                }
            }
        }
        if self.json {
            println!("{}", serde_json::to_string_pretty(&indexmap)?);
        }
        Ok(true)
    }
}
