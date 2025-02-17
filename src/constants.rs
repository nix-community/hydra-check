//! Useful constants shared across the program

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
