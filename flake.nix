{
  description = "Python application managed with poetry2nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils = { url = "github:numtide/flake-utils"; };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, flake-compat, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ ];
        };
        python = pkgs.python310;
        packageName = "hydra-check";
        # update the pyproject.toml too
        packageVersion = "1.3.5";
      in
      {
        packages = rec {
          hydra-check = python.pkgs.buildPythonApplication rec {
            pname = packageName;
            version = packageVersion;
            format = "pyproject";
            nativeBuildInputs = with python.pkgs; [ poetry-core ];
            propagatedBuildInputs = with python.pkgs; [ requests beautifulsoup4 colorama ];
            src = ./.;
            checkInputs = with pkgs; [ python310.pkgs.mypy ];
            checkPhase = ''
              #export MYPYPATH=$PWD/src
              #mypy --strict .
            '';
          };
          default = hydra-check;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              pyright
              poetry
              (pkgs.poetry2nix.mkPoetryEnv {
                inherit python;
                projectDir = ./.;
                overrides = pkgs.poetry2nix.overrides.withDefaults (self: super: { });
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
