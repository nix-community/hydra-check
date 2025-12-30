use anyhow::bail;
use colored::{ColoredString, Colorize};
use scraper::ElementRef;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{is_skipable_row, ShowHydraStatus, SoupFind, StatusIcon, TryAttr};

#[skip_serializing_none]
#[derive(Serialize, Debug, Default, Clone)]
/// Status of a single build attempt, can be serialized to a JSON entry
pub(crate) struct BuildStatus {
    pub(crate) icon: StatusIcon,
    pub(crate) success: bool,
    pub(crate) status: String,
    pub(crate) timestamp: Option<String>,
    pub(crate) build_id: Option<String>,
    pub(crate) build_url: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) arch: Option<String>,
    pub(crate) evals: bool,
    pub(crate) job_name: Option<String>,
}

impl ShowHydraStatus for BuildStatus {
    fn format_as_vec(&self) -> Vec<ColoredString> {
        let mut row = Vec::new();
        let icon = ColoredString::from(&self.icon);
        let status = match (self.evals, self.success) {
            (false, _) => format!("{icon} {}", self.status),
            (true, false) => format!("{icon} ({})", self.status),
            (true, true) => format!("{icon}"),
        };
        row.push(status.into());
        // job name is only available in some cases (e.g. in eval details)
        if let Some(x) = self.job_name.as_deref() {
            row.push(x.into());
        }
        let details = if self.evals {
            let name = self.name.clone().unwrap_or_default().into();
            let timestamp = self
                .timestamp
                .as_deref()
                .unwrap_or_default()
                .split_once('T')
                .unwrap_or_default()
                .0
                .into();
            &[name, timestamp]
        } else {
            &Default::default()
        };
        row.extend_from_slice(details);
        let build_url = self.build_url.clone().unwrap_or_default().dimmed();
        row.push(build_url);
        row
    }
}

impl BuildStatus {
    pub(crate) fn from_tbody(tbody: ElementRef<'_>) -> anyhow::Result<Vec<Self>> {
        let mut builds = Vec::new();
        for row in tbody.find_all("tr") {
            let columns = row.find_all("td");
            let (status, build, job_name, timestamp, name, arch) =
            // case I: the job is removed:
            if let [job_name, arch] =
                columns.as_slice()
            {
                let build_url = job_name.find("a")?.attr("href");
                let job_name: String = job_name.text().collect();
                let arch = arch.find("tt")?.text().collect();
                builds.push(BuildStatus {
                    icon: StatusIcon::Warning,
                    status: "Removed".into(),
                    build_url: build_url.map(str::to_string),
                    arch: Some(arch),
                    job_name: Some(job_name.trim().into()),
                    ..Default::default()
                });
                continue;
            } else
            // case II: there is no `job_name` column
            if let [status, build, timestamp, name, arch] = columns.as_slice() {
                let job_name = None;
                (status, build, job_name, timestamp, name, arch)
            } else
            // case III: there is a `job_name` column (e.g. in eval details page)
            if let [status, build, job_name, timestamp, name, arch] = columns.as_slice() {
                (status, build, Some(job_name), timestamp, name, arch)
            } else {
                #[allow(clippy::redundant_else)]
                if is_skipable_row(row)? {
                    continue;
                } else {
                    bail!("error while parsing build status from: {}", row.html());
                }
            };
            if let Ok(span_status) = status.find("span") {
                let span_status: String = span_status.text().collect();
                let status = if span_status.trim() == "Queued" {
                    "Queued: no build has been attempted for this package yet (still queued)"
                        .to_string()
                } else {
                    format!("Unknown Hydra status: {span_status}")
                };
                builds.push(BuildStatus {
                    icon: StatusIcon::Queued,
                    status,
                    ..Default::default()
                });
                continue;
            }
            let status = status.find("img")?.try_attr("title")?;
            let build_id = build.find("a")?.text().collect();
            let build_url = build.find("a")?.attr("href");
            let timestamp = timestamp.find("time").ok().and_then(|x| x.attr("datetime"));
            let name = name.text().collect();
            let job_name = job_name.map(|x| x.text().collect::<String>().trim().into());
            let arch = arch.find("tt")?.text().collect();
            let success = status == "Succeeded";
            let icon = match (success, status) {
                (true, _) => StatusIcon::Succeeded,
                (false, "Cancelled") => StatusIcon::Cancelled,
                (false, "Queued") => StatusIcon::Queued,
                (false, _) => StatusIcon::Failed,
            };
            let evals = true;
            builds.push(BuildStatus {
                icon,
                success,
                status: status.into(),
                timestamp: timestamp.map(str::to_string),
                build_id: Some(build_id),
                build_url: build_url.map(str::to_string),
                name: Some(name),
                arch: Some(arch),
                evals,
                job_name,
            });
        }
        Ok(builds)
    }
}
