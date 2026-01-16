{
  description = "scrape hydra for the build status of a package";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:NixOS/flake-compat";
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
          hydra-check = pkgs.callPackage ./package.nix { source = self; };
          default = self.packages.${system}.hydra-check;
        };

        devShells.default = (self.packages.${system}.hydra-check.override {
          # prevent dependence on the source ./.; see ./package.nix
          source = null;
        }).overrideAttrs ({ nativeBuildInputs ? [], env ? {}, ... }: {
          nativeBuildInputs = with pkgs.buildPackages; [
            git
            cargo # with shell completions, instead of cargo-auditable
            cargo-insta # for updating insta snapshots
            clippy # more lints for better rust code
            nixfmt-rfc-style # for formatting nix code
          ] ++ nativeBuildInputs;

          env = with pkgs.buildPackages; (removeAttrs env ["RUST_LOG"]) // {
            # for developments, e.g. symbol lookup in std library
            RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
            # for debugging
            RUST_LIB_BACKTRACE = "1";
          };
        });
      });
}
