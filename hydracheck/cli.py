"""usage: hydra-check [options] PACKAGES...

options:
    --arch=SYSTEM        system architecture to check [default: x86_64-linux]
    --json               write builds in machine-readable format
    --short              write only the latest build even if last build failed
    --url                only print the hydra build url, then exit
    --channel=CHAN       Channel to check packages for [Default: unstable]

Other channels can be:
    unstable  - alias for nixos/trunk-combined (Default)
    master    - alias for nixpkgs/trunk
    staging   - alias for nixos/staging
    19.03     - alias for nixos/release-19.03
    19.09     - alias for nixos/release-19.03
    20.03     - alias for nixos/release-19.03
    nixpkgs/nixpkgs-20.03-darwin    - verbatim jobset name

Jobset names can be constructed with the project name (e.g. `nixos/` or `nixpkgs/`)
followed by a branch name. The available jobsets can be found at:
* https://hydra.nixos.org/project/nixos
* https://hydra.nixos.org/project/nixpkgs

"""
from bs4 import BeautifulSoup
import requests
import json
from typing import Dict, Iterator, Union

from sys import exit

# TODO: use TypedDict
BuildStatus = Dict[str, Union[str, bool]]


# guess functions are intended to be fast without external queries
def guess_jobset(channel: str) -> str:
    # TODO guess the latest stable channel
    if channel == "master":
        return "nixpkgs/trunk"
    elif channel == "unstable":
        return "nixos/trunk-combined"
    elif channel == "staging":
        return "nixos/staging"
    elif channel[0].isdigit():
        # 19.09, 20.03 etc
        return f"nixos/release-{channel}"
    else:
        # we asume that the user knows the jobset name ( nixos/release-19.09 )
        return channel


def guess_packagename(package: str, arch: str, is_channel: bool) -> str:
    # TODO: maybe someone provides the architecture in the package name?
    if package.startswith("nixpkgs.") or package.startswith("nixos."):
        # we assume user knows the full package name
        return f"{package}.{arch}"
    elif is_channel:
        # we simply guess, that the user searches for a package and not a test
        return f"nixpkgs.{package}.{arch}"
    else:
        return f"{package}.{arch}"


def get_url(ident: str) -> str:
    # there is also {url}/all which is a lot slower
    return f"https://hydra.nixos.org/job/{ident}"


def fetch_data(ident: str) -> str:
    # https://hydra.nixos.org/job/nixos/release-19.09/nixpkgs.hello.x86_64-linux/latest
    # https://hydra.nixos.org/job/nixos/release-19.09/nixos.tests.installer.simpleUefiGrub.aarch64-linux
    # https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-linux/all
    url = get_url(ident)
    resp = requests.get(url, timeout=20)
    if resp.status_code == 404:
        print(f"package {ident} not found at url {url}")
        exit(1)
    return resp.text


def parse_build_html(data: str) -> Iterator[BuildStatus]:
    doc = BeautifulSoup(data, features="html.parser")
    if not doc.find("tbody"):
        # Either the package was not evaluated (due to being unfree)
        # or the package does not exist
        alert_text = (
            doc.find("div", {"class": "alert"}).text.replace("\n", " ")
            or "Unknown Hydra Error, check the package with --url to find out what went wrong"
        )
        yield {"icon": "⚠", "success": False, "evals": False, "status": alert_text}
        return

    for row in doc.find("tbody").find_all("tr"):
        try:
            status, build, timestamp, name, arch = row.find_all("td")
        except ValueError:
            if row.find("td").find("a")["href"].endswith("/all"):
                continue
            else:
                raise
        status = status.find("img")["title"]
        build_id = build.find("a").text
        build_url = build.find("a")["href"]
        timestamp = timestamp.find("time")["datetime"]
        name = name.text
        arch = arch.find("tt").text
        success = status == "Succeeded"
        icon = "✔" if success else "✖"
        yield {
            "icon": icon,
            "success": success,
            "status": status,
            "timestamp": timestamp,
            "build_id": build_id,
            "build_url": build_url,
            "name": name,
            "arch": arch,
            "evals": True,
        }


def print_buildreport(build: BuildStatus) -> None:
    if build["evals"]:
        extra = "" if build["success"] else f" ({build['status']})"
        print(
            f"{build['icon']}{extra} {build['name']} from {str(build['timestamp']).split('T')[0]} - {build['build_url']}"
        )
    else:
        print(f"{build['icon']} {build['status']}")


def main() -> None:
    from docopt import docopt

    args = docopt(__doc__)
    channel = args["--channel"]
    packages = args["PACKAGES"]
    arch = args["--arch"]
    only_url = args["--url"]
    jobset = guess_jobset(channel)
    is_channel = jobset.startswith("nixos/")
    as_json = args["--json"]
    all_builds = {}

    for package in packages:
        package_name = guess_packagename(package, arch, is_channel)
        ident = f"{jobset}/{package_name}"
        if only_url:
            print(get_url(ident))
            continue

        resp = fetch_data(ident)
        builds = list(parse_build_html(resp))
        all_builds[package] = builds

        if not as_json:
            latest = builds[0]
            print(f"Build Status for {package_name} on {channel}")
            print_buildreport(latest)
            if not latest["success"] and latest["evals"] and not args["--short"]:
                print()
                print("Last Builds:")
                for build in builds[1:]:
                    print_buildreport(build)
    if as_json:
        print(json.dumps(all_builds))


if __name__ == "__main__":
    main()
