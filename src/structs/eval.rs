use colored::{ColoredString, Colorize};
use log::info;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{constants, ShowHydraStatus, StatusIcon};

/// Specification for a single Hydra evaluation, with an optional filter.
/// Should only be constructed with [`Evaluation::guess_from_spec`]
/// to ensure a correct [`Evaluation::spec`], thus marked [`non_exhaustive`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Evaluation {
    #[serde(skip)]
    pub(crate) spec: String,
    pub(crate) id: u64,
    pub(crate) filter: Option<String>,
    pub(crate) more: bool,
}

#[skip_serializing_none]
#[derive(Serialize, Debug, Default, Clone)]
/// Status of a single evaluation, can be serialized to a JSON entry
pub(crate) struct EvalStatus {
    pub(crate) icon: StatusIcon,
    pub(crate) finished: Option<bool>,
    pub(crate) id: Option<u64>,
    pub(crate) url: Option<String>,
    pub(crate) datetime: Option<String>,
    pub(crate) relative: Option<String>,
    pub(crate) timestamp: Option<u64>,
    pub(crate) status: String,
    pub(crate) short_rev: Option<String>,
    pub(crate) input_changes: Option<String>,
    pub(crate) succeeded: Option<u64>,
    pub(crate) failed: Option<u64>,
    pub(crate) queued: Option<u64>,
    pub(crate) delta: Option<String>,
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

impl Evaluation {
    /// Parses an evaluation from a plain text specification.
    pub(crate) fn guess_from_spec(spec: &str, more: bool) -> Self {
        let spec = spec.trim();

        let mut split_spec = spec.splitn(2, '/');
        let id = split_spec.next().unwrap().trim();
        let filter = split_spec.next();

        let (id, filter) = match id.parse() {
            Ok(x) => (x, filter),
            Err(_) => (
                0u64, // for the latest eval
                match id.is_empty() {
                    true => filter,
                    false => Some(spec),
                },
            ),
        };
        let filter = match filter {
            None => {
                let default = constants::DEFAULT_EVALUATION_FILTER.to_string();
                info!(
                    "{}, so the default filter '/{default}' is used {}",
                    "no package filter has been specified", "for better performance"
                );
                info!(
                    "specify another filter with --eval '{}', {}\n",
                    format!(
                        "{}/<filter>",
                        match id {
                            0 => "<id>".into(),
                            x => x.to_string(),
                        }
                    ),
                    "or force an empty filter with a trailing slash '/'",
                );
                Some(default)
            }
            Some(x) if x.trim().is_empty() => None,
            Some(x) => Some(x.into()),
        };
        Self {
            spec: format!(
                "{id}{}",
                match &filter {
                    Some(x) => format!("/{x}"),
                    None => "".into(),
                }
            ),
            id,
            filter,
            more,
        }
    }
}

#[test]
fn guess_eval_from_spec() {
    let default_filter = constants::DEFAULT_EVALUATION_FILTER;
    #[allow(clippy::unreadable_literal)]
    for (spec, id, filter) in [
        ("123456", 123456, Some(default_filter.into())),
        ("123456/", 123456, None),
        ("123456/rustc", 123456, Some("rustc".into())),
        ("", 0, Some(default_filter.into())),
        ("/", 0, None),
        ("/rustc", 0, Some("rustc".into())),
        ("rustc", 0, Some("rustc".into())),
        ("weird/filter", 0, Some("weird/filter".into())),
    ] {
        let eval = Evaluation::guess_from_spec(spec, false);
        println!("{eval:?}");
        assert!(eval.id == id && eval.filter == filter);
    }
}
