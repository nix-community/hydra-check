//! Useful constants shared across the program

use log::{debug, info};
use std::sync::LazyLock;

/// Currently supported systems (`supportedSystems`) on [hydra.nixos.org](https://hydra.nixos.org).
///
/// This is invoked in the jobset: [nixpkgs/trunk](https://hydra.nixos.org/jobset/nixpkgs/trunk#tabs-configuration),
/// and defined by the following expressions in nixpkgs:
/// - [pkgs/top-level/release.nix](https://github.com/NixOS/nixpkgs/blob/master/pkgs/top-level/release.nix)
/// - [ci/supportedSystems.nix](https://github.com/NixOS/nixpkgs/blob/master/ci/supportedSystems.nix).
///
/// This may change in the future.
///
/// ```
/// assert_eq!(hydra_check::constants::KNOWN_ARCHITECTURES, [
///     "x86_64-linux",
///     "aarch64-linux",
///     "x86_64-darwin",
///     "aarch64-darwin",
/// ]);
/// ```
///
pub const KNOWN_ARCHITECTURES: [&str; 4] = [
    "x86_64-linux",
    "aarch64-linux",
    "x86_64-darwin",
    "aarch64-darwin",
];

/// Currently supported systems (`supportedSystems`) for NixOS.
///
/// This is invoked in the jobset: [nixos/trunk-combined](https://hydra.nixos.org/jobset/nixos/trunk-combined#tabs-configuration),
/// and defined by the expression: [nixpkgs: nixos/release-combined.nix](https://github.com/NixOS/nixpkgs/blob/master/nixos/release-combined.nix).
///
/// This may change in the future.
///
/// ```
/// assert_eq!(hydra_check::constants::NIXOS_ARCHITECTURES, [
///     "x86_64-linux",
///     "aarch64-linux",
/// ]);
/// ```
///
pub const NIXOS_ARCHITECTURES: [&str; 2] = ["x86_64-linux", "aarch64-linux"];

/// Default package filter for the details of a specific evaluation.
pub const DEFAULT_EVALUATION_FILTER: &str = "nixVersions.stable";

/// User agent header that we send along to hydra for identifying this app.
///
/// ```
/// assert!(
///     hydra_check::constants::APP_USER_AGENT.starts_with("hydra-check/")
/// );
/// ```
///
pub const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Host URL for the Hydra instance, can be overridden by an environment variable
/// with the same name.
///
/// ```
/// temp_env::with_var(
///     "HYDRA_CHECK_HOST_URL", None::<&str>, // when the env var is unset
///     || assert_eq!(
///         *hydra_check::constants::HYDRA_CHECK_HOST_URL,
///         "https://hydra.nixos.org"         // ... use the NixOS default
///     )                                     // ... otherwise use the env var
/// );
/// ```
///
pub static HYDRA_CHECK_HOST_URL: LazyLock<String> = LazyLock::new(get_host_url);

/// Hardcoded default host URL of the official NixOS Hydra instance.
/// This is intentionally not `pub` so that it cannot be misused outside
/// this module. They should always use [`HYDRA_CHECK_HOST_URL`] instead.
///
const HYDRA_CHECK_DEFAULT_HOST_URL: &str = "https://hydra.nixos.org";

/// Gets the hydra host URL from the environment variable $HYDRA_CHECK_HOST_URL.
/// Falls back to the default URL if the variable is not set or empty.
fn get_host_url() -> String {
    let var_name = "HYDRA_CHECK_HOST_URL";
    let url_default = HYDRA_CHECK_DEFAULT_HOST_URL;
    std::env::var(var_name)
        .ok()
        .map(|url_env| url_env.trim().to_string())
        .filter(|url_env| {
            let is_empty = url_env.is_empty();
            match is_empty {
                true => info!("${var_name} is set to an empty string"),
                false => info!("using hydra host URL from ${var_name}: {url_env}"),
            }
            !is_empty
        })
        .unwrap_or_else(|| {
            debug!("using default hydra host URL: {url_default}");
            url_default.into()
        })
}

pub(crate) fn is_default_host_url() -> bool {
    *HYDRA_CHECK_HOST_URL == HYDRA_CHECK_DEFAULT_HOST_URL
}

#[test]
fn host_url_once_lock() {
    temp_env::with_var(
        "HYDRA_CHECK_HOST_URL",
        Some("https://hydra.example.com"),
        || {
            assert_eq!(&*HYDRA_CHECK_HOST_URL, "https://hydra.example.com");
        },
    );
    // once the lazy static is initialized, it cannot be altered
    // even if the environment variable is changed
    temp_env::with_var("HYDRA_CHECK_HOST_URL", None::<&str>, || {
        assert_eq!(&*HYDRA_CHECK_HOST_URL, "https://hydra.example.com");
    });
}
