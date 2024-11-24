{
  description = "scrape hydra for the build status of a package";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          hydra-check = pkgs.callPackage ./package.nix { };
          default = self.packages.${system}.hydra-check;
        };

        devShells.default = self.packages.${system}.hydra-check.overrideAttrs ({ nativeBuildInputs, ... }: {
          nativeBuildInputs = with pkgs.buildPackages; [
            cargo # with shell completions, instead of cargo-auditable
            cargo-insta # for updating insta snapshots
            clippy # more lints for better rust code
          ] ++ nativeBuildInputs;

          env = with pkgs.buildPackages; {
            # for developments, e.g. symbol lookup in std library
            RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
            # for debugging
            RUST_LIB_BACKTRACE = "1";
          };
        });
      });
}
