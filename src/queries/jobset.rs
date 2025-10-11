//! A module that formats the details of the specified (or inferred) jobset,
//! from an url like: <https://hydra.nixos.org/jobset/nixpkgs/trunk/evals>.

use anyhow::bail;
use colored::{ColoredString, Colorize};
use indexmap::IndexMap;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    constants::HYDRA_CHECK_HOST_URL, is_skipable_row, FetchHydraReport, ResolvedArgs,
    ShowHydraStatus, SoupFind, StatusIcon, TryAttr,
};

#[skip_serializing_none]
#[derive(Serialize, Debug, Default, Clone)]
/// Status of a single evaluation, can be serialized to a JSON entry
struct EvalStatus {
    icon: StatusIcon,
    finished: Option<bool>,
    id: Option<u64>,
    url: Option<String>,
    datetime: Option<String>,
    relative: Option<String>,
    timestamp: Option<u64>,
    status: String,
    short_rev: Option<String>,
    input_changes: Option<String>,
    succeeded: Option<u64>,
    failed: Option<u64>,
    queued: Option<u64>,
    delta: Option<String>,
}

impl ShowHydraStatus for EvalStatus {
    fn format_as_vec(&self) -> Vec<ColoredString> {
        let mut row = Vec::new();
        let icon = ColoredString::from(&self.icon);
        let description = match &self.input_changes {
            Some(x) => x,
            None => &self.status,
        };
        row.push(format!("{icon} {description}").into());
        let details = if self.url.is_some() {
            let relative = self.relative.clone().unwrap_or_default().into();
            let statistics = [
                (StatusIcon::Succeeded, self.succeeded),
                (StatusIcon::Failed, self.failed),
                (StatusIcon::Queued, self.queued),
            ];
            let [suceeded, failed, queued] = statistics.map(|(icon, text)| -> ColoredString {
                format!(
                    "{} {}",
                    ColoredString::from(&icon),
                    text.unwrap_or_default()
                )
                .into()
            });
            let queued = match self.queued.unwrap_or_default() {
                x if x != 0 => queued.bold(),
                _ => queued.normal(),
            };
            let delta = format!(
                "Δ {}",
                match self.delta.clone().unwrap_or("~".into()).trim() {
                    x if x.starts_with('+') => x.green(),
                    x if x.starts_with('-') => x.red(),
                    x => x.into(),
                }
            )
            .into();
            let url = self.url.clone().unwrap_or_default().dimmed();
            &[relative, suceeded, failed, queued, delta, url]
        } else {
            &Default::default()
        };
        row.extend_from_slice(details);
        row
    }
}

#[derive(Clone)]
/// Container for the eval status and metadata of a jobset
struct JobsetReport<'a> {
    jobset: &'a str,
    url: String,
    /// Status of recent evaluations of the jobset
    evals: Vec<EvalStatus>,
}

impl FetchHydraReport for JobsetReport<'_> {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn finish_with_error(self, status: String) -> Self {
        Self {
            evals: vec![EvalStatus {
                icon: StatusIcon::Warning,
                status,
                ..Default::default()
            }],
            ..self
        }
    }
}

impl<'a> From<&'a ResolvedArgs> for JobsetReport<'a> {
    fn from(args: &'a ResolvedArgs) -> Self {
        //
        // https://hydra.nixos.org/jobset/nixpkgs/trunk/evals
        //
        let url = format!("{}/jobset/{}/evals", &*HYDRA_CHECK_HOST_URL, args.jobset);
        Self {
            jobset: &args.jobset,
            url,
            evals: vec![],
        }
    }
}

impl JobsetReport<'_> {
    fn fetch_and_read(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc, "") {
            Err(stat) => return Ok(stat),
            Ok(tbody) => tbody,
        };
        let mut evals: Vec<EvalStatus> = Vec::new();
        for row in tbody.find_all("tr") {
            let columns = row.find_all("td");
            let [eval_id, timestamp, input_changes, succeeded, failed, queued, delta] =
                columns.as_slice()
            else {
                #[allow(clippy::redundant_else)]
                if is_skipable_row(row)? {
                    continue;
                } else {
                    bail!(
                        "error while parsing Hydra status for jobset '{}': {:?}",
                        self.jobset,
                        row
                    );
                }
            };

            let url = eval_id.find("a")?.try_attr("href")?;
            let eval_id: String = eval_id.text().collect();
            let id: u64 = eval_id.parse()?;

            let time = timestamp.find("time")?;
            let date = time.try_attr("datetime")?;
            let relative = time.text().collect();
            let timestamp = time.try_attr("data-timestamp")?;
            let timestamp: u64 = timestamp.parse()?;

            let status: String = input_changes
                .find("span")
                .map(|x| x.text().collect())
                .unwrap_or_default();

            let short_rev = input_changes.find("tt")?.text().collect();
            let input_changes = {
                let text: String = input_changes.text().collect();
                let text = text.replace(&status, "");
                let texts: Vec<_> = text.split_whitespace().collect();
                texts.join(" ")
            };

            let [succeeded, failed, queued, delta] = [succeeded, failed, queued, delta].map(|x| {
                let text: String = x.text().collect();
                text.trim().to_string()
            });

            let [succeeded, failed, queued]: [Result<u64, _>; 3] =
                [succeeded, failed, queued].map(|text| match text.is_empty() {
                    true => Ok(0),
                    false => text.parse(),
                });
            let delta = match delta {
                x if x.is_empty() => None,
                x => Some(x),
            };

            let finished = queued == Ok(0);
            let icon = match finished {
                true => StatusIcon::Succeeded,
                false => StatusIcon::Queued,
            };

            evals.push(EvalStatus {
                icon,
                finished: Some(finished),
                id: Some(id),
                url: Some(url.into()),
                datetime: Some(date.into()),
                relative: Some(relative),
                timestamp: Some(timestamp),
                status,
                short_rev: Some(short_rev),
                input_changes: Some(input_changes),
                succeeded: Some(succeeded?),
                failed: Some(failed?),
                queued: Some(queued?),
                delta,
            });
        }
        Ok(Self { evals, ..self })
    }
}

impl ResolvedArgs {
    /// - Returns the latest evaluation ID if available.
    /// - Forces `--short` output if `force_short_output` is true.
    ///   This _was_ used with `--eval` to avoid long outputs, but it turns out
    ///   that detailed jobset information is practically useful, so this
    ///   toggle is currently unused.
    pub(crate) fn fetch_and_print_jobset(
        &self,
        force_short_output: bool,
    ) -> anyhow::Result<Option<u64>> {
        let stat = JobsetReport::from(self);
        let (short, json) = match force_short_output {
            true => (true, false),
            false => (self.short, self.json),
        };
        if self.url {
            println!("{}", stat.get_url());
            return Ok(None);
        }
        if !json {
            // print title first, then fetch
            println!(
                "Evaluations of jobset {} {}",
                self.jobset.bold(),
                format!("@ {}", stat.get_url()).dimmed()
            );
        }
        let stat = stat.fetch_and_read()?;
        let first_stat = stat.evals.first();
        let latest_id = first_stat.and_then(|x| x.id);
        if json {
            let mut indexmap = IndexMap::new();
            match short {
                true => indexmap.insert(
                    &stat.jobset,
                    match first_stat {
                        Some(x) => vec![x.to_owned()],
                        None => vec![],
                    },
                ),
                false => indexmap.insert(&stat.jobset, stat.evals),
            };
            println!("{}", serde_json::to_string_pretty(&indexmap)?);
            return Ok(latest_id);
        }
        println!("{}", stat.format_table(short, &stat.evals));
        Ok(latest_id)
    }
}
