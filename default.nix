{ pkgs ? import <nixpkgs> {} }:

with pkgs.python3.pkgs;
buildPythonPackage {
  name = "hydra-check";
  src = ./.;
  propagatedBuildInputs = [
    docopt
    requests
    beautifulsoup4
  ];
  checkInputs = [ black jq ];
}
