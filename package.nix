{
  lib,
  hydra-check,
  rustPlatform,
  versionCheckHook,
}:

hydra-check.overrideAttrs (
  {
    meta ? { },
    nativeInstallCheckInputs ? [ ],
    ...
  }:
  {
    version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

    # `builtins.path` works well with lazy trees
    src = builtins.path {
      name = "hydra-check-source";
      path = ./.;
    };

    cargoDeps = rustPlatform.importCargoLock {
      lockFile = builtins.path {
        name = "hydra-check-Cargo.lock";
        path = ./Cargo.lock;
      };
    };

    nativeInstallCheckInputs = nativeInstallCheckInputs ++ [
      versionCheckHook
    ];

    doInstallCheck = true;

    meta = meta // {
      maintainers = with lib.maintainers; [
        makefu
        artturin
        bryango
      ];
    };
  }
)
