{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  stdenv,
  darwin,
}:

rustPlatform.buildRustPackage {
  pname = "hydra-check";
  version = "2.0.0";

  src = builtins.path {
    name = "hydra-check-source";
    path = ./.;
  };

  cargoLock = {
    lockFile = builtins.path {
      name = "hydra-check-Cargo.lock";
      path = ./Cargo.lock;
    };
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs =
    [
      openssl
    ]
    ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.Security
      darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  meta = {
    description = "scrape hydra for the build status of a package";
    homepage = "https://github.com/bryango/hydra-check";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ bryango ];
    mainProgram = "hydra-check";
  };
}
