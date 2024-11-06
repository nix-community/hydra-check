//! Useful constants shared across the program

/// Currently supported systems on [hydra.nixos.org][hydra].
/// This may change in the future.
///
/// [hydra]: https://hydra.nixos.org
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

/// Default package filter for the details of a specific evaluation.
pub const DEFAULT_EVALUATION_FILTER: &str = "nixVersions.stable";
