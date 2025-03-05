use anyhow::bail;

const WARN: &str = "cargo::warning=";

fn main() -> anyhow::Result<()> {
    check_and_refine_package_versions()
}

/// - Checks that the package version is consistent bewteen:
///
///    - `Cargo.toml` (i.e. `$CARGO_PKG_VERSION`), and
///    - `package.nix` (i.e. `$version`).
///
/// - Extends the package version, i.e. `$CARGO_PKG_VERSION` with a commit
///   hash suffix generated in package.nix. This is useful for
///   identifying development builds.
///
fn check_and_refine_package_versions() -> anyhow::Result<()> {
    println!("cargo::rerun-if-env-changed=version");

    let set_version = |version: &str| {
        println!("{WARN}setting version to: {version}");
        println!("cargo::rustc-env=CARGO_PKG_VERSION={version}");
    };

    let version_from_cargo = env!("CARGO_PKG_VERSION");
    let version_from_nix = if let Some(version_from_nix) = option_env!("version") {
        if !version_from_nix.starts_with(version_from_cargo) {
            bail!(
                "inconsistent versioning: Cargo.toml={} vs package.nix={}\n{}\n{}",
                version_from_cargo,
                version_from_nix,
                "either update the versions or refresh the development environment",
                "if you are using direnv, prepend `watch_file Cargo.lock` to `.envrc`",
            );
        }
        version_from_nix
    } else {
        version_from_cargo
    };

    set_version(version_from_nix);
    Ok(())
}
