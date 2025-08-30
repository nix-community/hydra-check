use log::info;
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
    pub(crate) more: bool,
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
