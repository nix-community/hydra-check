use anyhow::bail;
use colored::Colorize;
use scraper::ElementRef;
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;
use std::fmt::Display;

#[cfg(test)]
use insta::assert_snapshot;

use crate::{is_skipable_row, SoupFind};

#[skip_serializing_none]
#[derive(Serialize, Clone, Default, Debug)]
/// Inputs of a given evaluation (which is also the inputs of a package build)
pub(crate) struct EvalInput {
    pub(crate) name: Option<String>,
    #[serde(rename = "type")]
    pub(crate) input_type: Option<String>,
    pub(crate) value: Option<String>,
    pub(crate) revision: Option<String>,
    pub(crate) store_path: Option<String>,
}

impl Display for EvalInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(&self).expect("EvalInput should be serialized into json");
        let json: Value = serde_json::from_str(&json).unwrap();
        let strings: Vec<_> = ["name", "type", "value", "revision", "store_path"]
            .iter()
            .filter_map(|key| match &json[key] {
                Value::Null => None,
                // unquote the string:
                Value::String(value) => {
                    let key = match *key {
                        "name" => "input",
                        k => k,
                    };
                    Some(format!("{}: {}", key.bold(), value))
                }
                value => Some(format!("{}: {}", key.bold(), value)),
            })
            .collect();
        write!(f, "{}", strings.join("\n"))
    }
}

impl EvalInput {
    pub(crate) fn from_tbody(tbody: ElementRef<'_>, caller_id: &str) -> anyhow::Result<Vec<Self>> {
        let mut inputs: Vec<EvalInput> = Vec::new();
        for row in tbody.find_all("tr") {
            let columns = row.find_all("td");
            let columns: Vec<_> = columns
                .iter()
                .map(|x| {
                    let text: String = x.text().collect();
                    match text.trim() {
                        "" => None,
                        x => Some(x.to_string()),
                    }
                })
                .collect();
            let [name, input_type, value, revision, store_path] = columns.as_slice() else {
                #[allow(clippy::redundant_else)]
                if let Ok(true) = is_skipable_row(row) {
                    continue;
                } else {
                    bail!(
                        "error while parsing inputs for {}: {:?}",
                        caller_id,
                        row.html()
                    );
                }
            };
            inputs.push(EvalInput {
                name: name.to_owned(),
                input_type: input_type.to_owned(),
                value: value.to_owned(),
                revision: revision.to_owned(),
                store_path: store_path.to_owned(),
            });
        }
        Ok(inputs)
    }
}

#[test]
fn format_eval_input() {
    let eval_input = EvalInput {
        name: Some("nixpkgs".into()),
        input_type: Some("Git checkout".into()),
        value: Some("https://github.com/nixos/nixpkgs.git".into()),
        revision: Some("1e9e641a3fc1b22fbdb823a99d8ff96692cc4fba".into()),
        store_path: Some("/nix/store/ln479gq56q3kyzyl0mm00xglpmfpzqx4-source".into()),
    };
    assert_snapshot!(eval_input.to_string(), @r#"
        [1minput[0m: nixpkgs
        [1mtype[0m: Git checkout
        [1mvalue[0m: https://github.com/nixos/nixpkgs.git
        [1mrevision[0m: 1e9e641a3fc1b22fbdb823a99d8ff96692cc4fba
        [1mstore_path[0m: /nix/store/ln479gq56q3kyzyl0mm00xglpmfpzqx4-source
    "#);
}
