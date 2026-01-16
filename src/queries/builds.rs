//! A module that formats the details of one (or multiple) build(s),
//! from urls such as <https://hydra.nixos.org/build/290062156>.
//!
//! This module is adapted from the `evals` module as the two are similar
//! in structure. The module is currently only used by the `builds` module,
//! hence the relevant interfaces are marked as `pub(super)`.

use serde::Serialize;

use crate::{EvalInput, FetchHydraReport, StatusIcon};

#[non_exhaustive]
#[derive(Serialize, Clone)]
pub(super) struct BuildReport {
    url: String,
    pub(super) inputs: Vec<EvalInput>,
}

impl FetchHydraReport for BuildReport {
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

impl BuildReport {
    pub(super) fn from_url(url: &str) -> Self {
        Self {
            url: url.to_string(),
            inputs: vec![],
        }
    }

    pub(super) fn fetch_and_read(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc, "div#tabs-buildinputs") {
            // inputs are essential information, so exit early if this fails:
            Err(stat) => return Ok(stat),
            Ok(tbody) => tbody,
        };
        let inputs = EvalInput::from_tbody(tbody, &self.url)?;
        Ok(Self { inputs, ..self })
    }
}
