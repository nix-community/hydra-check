{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  stdenv,
  installShellFiles,
  versionCheckHook,
}:

rustPlatform.buildRustPackage {
  pname = "hydra-check";
  version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

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

  buildInputs = [
    openssl
  ];

  postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    installShellCompletion --cmd hydra-check \
      --bash <($out/bin/hydra-check --shell-completion bash) \
      --fish <($out/bin/hydra-check --shell-completion fish) \
      --zsh <($out/bin/hydra-check --shell-completion zsh)
  '';

  nativeInstallCheckInputs = [
    versionCheckHook
  ];

  doInstallCheck = true;

  meta = {
    description = "Check hydra for the build status of a package";
    homepage = "https://github.com/nix-community/hydra-check";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [
      makefu
      artturin
      bryango
    ];
    mainProgram = "hydra-check";
  };
}
