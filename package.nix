{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  stdenv,
  darwin,
  installShellFiles,
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
    installShellFiles
  ];

  buildInputs =
    [
      openssl
    ]
    ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.Security
      darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    installShellCompletion --cmd hydra-check \
      --bash <($out/bin/hydra-check --shell-completion bash) \
      --fish <($out/bin/hydra-check --shell-completion fish) \
      --zsh <($out/bin/hydra-check --shell-completion zsh)
  '';

  meta = {
    description = "scrape hydra for the build status of a package";
    homepage = "https://github.com/bryango/hydra-check";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ bryango ];
    mainProgram = "hydra-check";
  };
}
