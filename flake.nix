{
  description = "Python application managed with poetry2nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    poetry2nix = {
      url = "github:nix-community/poetry2nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.systems.follows = "flake-utils/systems";
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ ];
        };
        poetry2nix = self.inputs.poetry2nix.lib.mkPoetry2Nix {
          inherit pkgs;
        };
        python = pkgs.python312;
        packageName = "hydra-check";
        # update the pyproject.toml too
        packageVersion = "1.3.5";
      in
      {
        packages = {
          hydra-check = python.pkgs.buildPythonApplication {
            pname = packageName;
            version = packageVersion;
            format = "pyproject";
            nativeBuildInputs = with python.pkgs; [ poetry-core ];
            propagatedBuildInputs = with python.pkgs; [ requests beautifulsoup4 colorama ];
            src = builtins.path {
              name = "hydra-check-source";
              path = ./.;
            };
            checkInputs = with python.pkgs; [ mypy ];
            checkPhase = ''
              #export MYPYPATH=$PWD/src
              #mypy --strict .
            '';
          };
          default = self.packages.${system}.hydra-check;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              pyright
              poetry
              (poetry2nix.mkPoetryEnv {
                inherit python;
                projectDir = ./.;
                overrides = poetry2nix.overrides.withDefaults (self: super: { });
                editablePackageSources = {
                  hydra-check = ./src;
                };
                extraPackages = (ps: with ps; [
                ]);
              })
            ] ++ (with python.pkgs; [
              black
              pylint
              mypy
            ]);
            shellHook = ''
              export MYPYPATH=$PWD/src
            '';
          };
        };

      });
}
