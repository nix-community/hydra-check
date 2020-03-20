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
  postUnpack = ''
    echo -e "\x1b[32m## build on ${pkgs.lib.version} \x1b[0m"
  '';
  checkInputs = [ black mypy jq flake8 ];
  checkPhase = ''
    echo -e "\x1b[32m## run black\x1b[0m"
    LC_ALL=en_US.utf-8 black --check .
    echo -e "\x1b[32m## run flake8\x1b[0m"
    flake8 hydracheck
    echo -e "\x1b[32m## run mypy\x1b[0m"
    mypy hydracheck
  '';
}
