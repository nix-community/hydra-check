use log::{error, info};
use serde::Serialize;

use crate::constants;

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
}

impl Evaluation {
    /// Parses an evaluation from a plain text specification.
    pub(crate) fn guess_from_spec(spec: &str) -> Self {
        let mut spec = spec.splitn(2, "/");
        let id = spec.next().unwrap();
        let id = match id.parse() {
            Ok(id) => id,
            Err(err) => {
                error!(
                    "evaluations must be identified by a number {} {} '{}': {}",
                    "(slash an optional filter), e.g. '1809585/coreutils'.",
                    "Instead we get",
                    id,
                    err
                );
                std::process::exit(1);
            }
        };
        let filter = match spec.next() {
            None => {
                let default = constants::DEFAULT_EVALUATION_FILTER.to_string();
                info!(
                    "{}, so the default filter '/{default}' is used {}",
                    "no package filter has been specified", "for better performance"
                );
                info!(
                    "specify another filter with --eval '{}', {}: '{}'\n",
                    format!("{id}/<filter>"),
                    "or force an empty filter with a trailing slash",
                    format!("{id}/")
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
        }
    }
}

#[test]
fn guess_eval_from_spec() {
    let default_filter = constants::DEFAULT_EVALUATION_FILTER.into();
    for (spec, id, filter) in [
        ("123456", 123456, Some(default_filter)),
        ("123456/", 123456, None),
        ("123456/rustc", 123456, Some("rustc".into())),
    ] {
        let eval = Evaluation::guess_from_spec(&spec);
        debug_assert!(eval.id == id && eval.filter == filter);
    }
}
