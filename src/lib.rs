#![warn(missing_docs)]

//! <div class="warning">
//!
//! Currently hydra-check is only provided as a CLI application, _not_ as a library.
//! We exposed the library documentation here for internal reference.
//! We do _not_ provide any guarantee about the stability of these interfaces.
//!
//! </div>
//!
#![allow(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

mod args;
mod fetch_stable;
mod queries;
mod structs;

pub mod constants;
pub mod soup;

pub use args::HydraCheckCli;
pub use fetch_stable::NixpkgsChannelVersion;

use args::ResolvedArgs;
use soup::{SoupFind, TryAttr};
use structs::{BuildStatus, EvalInput, EvalStatus, Evaluation, StatusIcon};

use colored::{ColoredString, Colorize};
use comfy_table::Table;
use flexi_logger::filter::LogLineFilter;
use scraper::{ElementRef, Html};
use std::time::Duration;

/// Trait for a single `Status` entry from a Hydra report.
/// This usually corresponds to a single line in the tables from Hydra's
/// web interface, such as a single [`BuildStatus`].
trait ShowHydraStatus {
    fn format_as_vec(&self) -> Vec<ColoredString>;
}

/// Trait for all kinds of Hydra `Report`.
/// This usually corresponds to a summary page from Hydra's web interface,
/// such as <https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-linux>.
trait FetchHydraReport: Clone {
    fn get_url(&self) -> &str;
    fn fetch_document(&self) -> anyhow::Result<Html> {
        let document = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(constants::APP_USER_AGENT)
            .build()?
            .get(self.get_url())
            .send()?
            .error_for_status()?
            .text()?;
        Ok(Html::parse_document(&document))
    }

    fn finish_with_error(self, status: String) -> Self;

    /// Checks if the fetched [Html] contains a `tbody` tag (table body).
    /// If not, returns the alert text. If yes, returns the found element.
    fn find_tbody<'a>(&self, doc: &'a Html, selector: &str) -> Result<ElementRef<'a>, Self> {
        let selectors = format!("{selector} tbody");
        match doc.find(selectors.trim()) {
            Err(_) => {
                // either the package was not evaluated (due to being e.g. unfree)
                // or the package does not exist
                let status = if let Ok(alert) = doc.find("div.alert") {
                    alert.text().collect()
                } else {
                    format!(
                        "Unknown Hydra Error with '{}' found at {}",
                        selectors,
                        self.get_url()
                    )
                };
                // sanitize the text a little bit
                let status: Vec<&str> = status.lines().map(str::trim).collect();
                let status: String = status.join(" ");
                Err(self.clone().finish_with_error(status))
            }
            Ok(tbody) => Ok(tbody),
        }
    }

    fn format_table<T: ShowHydraStatus>(&self, short: bool, entries: &Vec<T>) -> String {
        let mut table = Table::new();
        table.load_preset(comfy_table::presets::NOTHING);
        // .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
        for entry in entries {
            table.add_row(entry.format_as_vec());
            if short {
                break;
            }
        }
        for (idx, column) in table.column_iter_mut().enumerate() {
            if idx == 0 {
                column.set_padding((0, 1));
            }
            // column.set_constraint(comfy_table::ColumnConstraint::ContentWidth);
        }
        table.trim_fmt()
    }
}

fn is_skipable_row(row: ElementRef<'_>) -> anyhow::Result<bool> {
    let link = row.find("td")?.find("a")?.try_attr("href")?;
    let skipable = link.ends_with("/all") || link.contains("full=1");
    Ok(skipable)
}

fn log_format(
    w: &mut dyn std::io::Write,
    _now: &mut flexi_logger::DeferredNow,
    record: &log::Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    let color = match level {
        log::Level::Error => "red",
        log::Level::Warn => "yellow",
        _ => "",
    };
    let level = format!("{level}:").to_lowercase().color(color).bold();
    let module = record.module_path().unwrap_or_default();
    write!(
        w,
        "{} {}{}",
        level,
        (!module.starts_with("hydra_check"))
            .then_some(format!("[{module}] "))
            .unwrap_or_default(),
        &record.args()
    )
}

struct LogFilter;
impl LogLineFilter for LogFilter {
    fn write(
        &self,
        now: &mut flexi_logger::DeferredNow,
        record: &log::Record,
        log_line_writer: &dyn flexi_logger::filter::LogLineWriter,
    ) -> std::io::Result<()> {
        let module = record.module_path().unwrap_or_default();
        let blacklist = ["selectors", "html5ever"];
        if blacklist.iter().any(|x| module.starts_with(x)) && record.level() != log::Level::Trace {
            return Ok(()); // skip log
        }
        log_line_writer.write(now, record)
    }
}
