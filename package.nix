{
  lib,
  stdenv,
  hydra-check,
  rustPlatform,
  buildPackages,
  source ? builtins.path {
    # `builtins.path` works well with lazy trees
    name = "hydra-check-source";
    path = ./.;
  },
}:

let

  packageVersion = with builtins; (fromTOML (readFile "${source}/Cargo.toml")).package.version;

  # append git revision to the version string, if available
  versionSuffix =
    if (source ? dirtyShortRev || source ? shortRev) then
      "-g${source.dirtyShortRev or source.shortRev}"
    else
      "";

  newVersion = "${packageVersion}${versionSuffix}";

in

hydra-check.overrideAttrs (
  {
    version,
    meta ? { },
    patches ? [ ],
    ...
  }:
  {
    version =
      assert lib.assertMsg (lib.versionAtLeast newVersion version) ''
        hydra-check provided here (${newVersion}) failed to be newer
        than the one provided in nixpkgs (${version}).
      '';
      newVersion;

    patches =
      patches
      ++ lib.optional (
        lib.hasPrefix "1.88" buildPackages.rustc.version && stdenv.buildPlatform.system == "x86_64-darwin"
      ) ./fix-cargo-1_88-reqwest.patch;

    src = source;

    cargoDeps = rustPlatform.importCargoLock {
      lockFile = "${source}/Cargo.lock";
    };

    meta = meta // {
      maintainers = with lib.maintainers; [
        makefu
        artturin
        bryango
      ];
      # to correctly generate meta.position for backtrace:
      inherit (meta) description;
    };
  }
)
