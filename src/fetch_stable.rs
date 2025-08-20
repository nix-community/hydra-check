use anyhow::bail;
use log::debug;
use scraper::Html;
use serde::Deserialize;
use std::sync::OnceLock;

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
    pub fn stable() -> anyhow::Result<String> {
        if let Some(version) = NIXPKGS_STABLE_VERSION.get() {
            return Ok(version.clone());
        }

        let version = Self::fetch_channel("stable")?;
        let new_version = version.clone();
        std::thread::spawn(move || {
            NIXPKGS_STABLE_VERSION
                .set(new_version.clone())
                .unwrap_or_else(|cached_version| {
                    if new_version != cached_version {
                        debug!(
                            "failed to cache the stable version '{new_version}': found already cached version '{cached_version}'"
                        );
                    }
                });
        });
        Ok(version)
    }
}

#[test]
#[ignore = "require internet connection"]
fn fetch_stable() {
    let ver = NixpkgsChannelVersion::stable().unwrap();
    println!("latest stable version: {ver}");
    debug_assert!(regex::Regex::new(r"^[0-9]+\.[0-9]+")
        .unwrap()
        .is_match(&ver));
}
