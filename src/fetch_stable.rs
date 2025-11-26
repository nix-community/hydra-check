use anyhow::bail;
use log::debug;
use once_cell::sync::OnceCell as OnceLock;
use scraper::Html;
use serde::Deserialize;

use crate::{SoupFind, TryAttr};

/// Static cache for the current stable version of Nixpkgs, set and used
/// internally by [`NixpkgsChannelVersion::stable()`].
static NIXPKGS_STABLE_VERSION: OnceLock<String> = OnceLock::new();

/// Currently supported Nixpkgs channel version
///
/// This provides a extremely hacky way of obtaining the latest release
/// number (e.g. 24.05) of Nixpkgs, by parsing the manual on nixos.org.
///
#[derive(Deserialize, Debug, Clone)]
pub struct NixpkgsChannelVersion {
    #[serde(rename = "channel")]
    status: String,
    version: String,
}

impl NixpkgsChannelVersion {
    fn fetch() -> anyhow::Result<Vec<Self>> {
        debug!("fetching the latest channel version from nixos.org/manual");
        let document = reqwest::blocking::get("https://nixos.org/manual/nixpkgs/stable/")?
            .error_for_status()?
            .text()?;
        let html = Html::parse_document(&document);
        let channels_spec = html.find("body")?.try_attr("data-nixpkgs-channels")?;
        Ok(serde_json::from_str(channels_spec)?)
    }

    fn fetch_channel(spec: &str) -> anyhow::Result<String> {
        let channels = Self::fetch()?;
        for channel in channels.clone() {
            if channel.status == spec {
                return Ok(channel.version);
            }
        }
        bail!(
            "could not find '{spec}' from supported channels: {:?}",
            channels
        )
    }

    /// Fetches the current stable version number of Nixpkgs
    pub fn stable() -> anyhow::Result<&'static str> {
        let version = NIXPKGS_STABLE_VERSION.get_or_try_init(|| Self::fetch_channel("stable"))?;
        Ok(version.as_str())
    }
}

#[test]
#[ignore = "require internet connection"]
fn fetch_stable() {
    let _logger_handle = crate::set_up_logger("debug").unwrap();
    let ver = NixpkgsChannelVersion::stable().unwrap();
    println!("latest stable version: {ver}");
    debug_assert!(regex::Regex::new(r"^[0-9]+\.[0-9]+").unwrap().is_match(ver));
}
