with import <nixpkgs> {};
with python3.pkgs;
buildPythonPackage {
  name = "env";
  src = ./.;
  propagatedBuildInputs = [
    docopt
    requests
    beautifulsoup4
  ];
  checkInputs = [ python3.pkgs.black ];
}
