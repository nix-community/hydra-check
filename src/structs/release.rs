use std::fmt::Display;

use colored::{ColoredString, Colorize};
use serde::Serialize;
use serde_with::skip_serializing_none;
use yansi::hyperlink::HyperlinkExt;

use crate::{BuildStatus, EvalStatus, ShowHydraStatus, StatusIcon};

/// Container for the evaluation and test build status of a (potential)
/// channel release.
#[skip_serializing_none]
#[derive(Serialize, Debug, Default, Clone)]
pub(crate) struct ReleaseStatus {
    pub(crate) eval: EvalStatus,
    pub(crate) test: BuildStatus,
    pub(crate) release_url: Option<String>,
}

impl ShowHydraStatus for ReleaseStatus {
    fn format_as_vec(&self) -> Vec<ColoredString> {
        fn format_with_optional_hyperlink(expr: impl Display, url: Option<&String>) -> String {
            let expr = format!("{expr}");
            match url {
                Some(url) => expr.link(url).to_string(),
                None => expr,
            }
        }
        let (eval, test) = (&self.eval, &self.test);
        let mut row = Vec::new();

        // information about the evaluation
        let icon = ColoredString::from(&eval.icon);
        let id_string = eval
            .id
            .map(|id| format_with_optional_hyperlink(id, eval.url.as_ref()))
            .unwrap_or_default();
        row.push(format!("{icon} {id_string}").into());
        let name = if test.evals {
            // if the release test is evaluated, use its name
            // e.g. nixpkgs-25.11pre854150.5d8f4beac036
            let name = test.name.as_deref().unwrap_or_default();
            format_with_optional_hyperlink(name, self.release_url.as_ref())
        } else if let Some(input_changes) = &eval.input_changes {
            input_changes.clone()
        } else {
            eval.status.clone()
        };
        row.push(name.into());
        let details = if eval.url.is_some() {
            let timestamp = eval.relative.as_deref().unwrap_or_default().into();
            let statistics = [
                (StatusIcon::Succeeded, eval.succeeded),
                (StatusIcon::Failed, eval.failed),
                (StatusIcon::Queued, eval.queued),
            ];
            let [suceeded, failed, queued] = statistics.map(|(icon, text)| -> ColoredString {
                format!(
                    "{} {}",
                    ColoredString::from(&icon),
                    text.unwrap_or_default()
                )
                .into()
            });
            let queued = match eval.queued.unwrap_or_default() {
                x if x != 0 => queued.bold(),
                _ => queued.normal(),
            };
            let delta = format!(
                "Δ {}",
                match eval.delta.clone().unwrap_or("~".into()).trim() {
                    x if x.starts_with('+') => x.green(),
                    x if x.starts_with('-') => x.red(),
                    x => x.into(),
                }
            )
            .into();
            &[timestamp, suceeded, failed, queued, delta]
        } else {
            &Default::default()
        };
        row.extend_from_slice(details);

        // information about the test build
        let icon = ColoredString::from(&test.icon);
        let test_info = if test.evals {
            test.timestamp
                .as_deref()
                .unwrap_or_default()
                .split_once('T')
                .unwrap_or_default()
                .0
                .into()
        } else if let Some(build_id) = &test.build_id {
            format!("build/{build_id}")
        } else {
            "".into()
        };
        let test_info_with_url =
            format_with_optional_hyperlink(&test_info, test.build_url.as_ref());
        let test_status = match test.evals {
            false => format!("{icon} {} {test_info_with_url}", test.status),
            true => format!("{icon} {test_info_with_url}"),
        };
        row.push(test_status.into());
        row
    }
}
