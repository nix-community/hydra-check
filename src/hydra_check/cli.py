import json
import logging
from sys import exit as sysexit
from typing import Dict, Iterator, Union

import requests
from bs4 import BeautifulSoup, element
from colorama import Fore

from hydra_check.arguments import process_args

# TODO: use TypedDict
BuildStatus = Dict[str, Union[str, bool]]


# guess functions are intended to be fast without external queries
def guess_jobset(channel: str) -> str:
    # TODO guess the latest stable channel
    match channel:
        case "master":
            return "nixpkgs/trunk"
        case "unstable":
            return "nixos/trunk-combined"
        case "staging":
            return "nixos/staging"
        case _:
            if channel[0].isdigit():
                # 19.09, 20.03 etc
                return f"nixos/release-{channel}"
            return channel


def guess_packagename(package: str, arch: str, is_channel: bool) -> str:
    # TODO: maybe someone provides the architecture in the package name?
    if package.startswith("nixpkgs.") or package.startswith("nixos."):
        # we assume user knows the full package name
        return f"{package}.{arch}"

    if is_channel:
        # we simply guess, that the user searches for a package and not a test
        return f"nixpkgs.{package}.{arch}"

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
        sysexit(1)
    return resp.text


def parse_build_html(data: str) -> Iterator[BuildStatus]:
    doc = BeautifulSoup(data, features="html.parser")
    if not doc.find("tbody"):
        # Either the package was not evaluated (due to being unfree)
        # or the package does not exist
        alert_text = ""
        if result := doc.find("div", {"class": "alert"}):
            alert_text = result.text.replace("\n", " ")
        else:
            alert_text = "Unknown Hydra Error, check the package with --url to find out what went wrong"

        yield {"icon": "⚠", "success": False, "evals": False, "status": alert_text}
        return

    if tbody := doc.find("tbody"):
        if isinstance(tbody, element.Tag):
            for row in tbody.find_all("tr"):
                try:
                    status, build, timestamp, name, arch = row.find_all("td")
                except ValueError:
                    if row.find("td").find("a")["href"].endswith("/all"):
                        continue
                    raise

                span_status = status.find("span")
                if span_status is not None:
                    if span_status.string == "Queued":
                        alert_text = "No build has been attempted for this package yet (still queued)"
                    else:
                        alert_text = f"Unknown Hydra status: {span_status.string}"

                    yield {"icon": "⧖", "success": False, "evals": False, "status": alert_text}

                    continue

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
    match build["icon"]:
        case "✖":
            print(Fore.RED, end="")
        case "⚠" | "⧖":
            print(Fore.YELLOW, end="")
        case "✔":
            print(Fore.GREEN, end="")
    if build["evals"]:
        extra = "" if build["success"] else f" ({build['status']})"
        print(
            f"{build['icon']}{extra} {build['name']} from "
            f"{str(build['timestamp']).split('T', maxsplit=1)[0]} - {build['build_url']}",
        )
    else:
        print(f"{build['icon']} {build['status']}")


def main() -> None:
    logging.basicConfig(format='%(levelname)s: %(message)s')

    args = process_args()

    channel = args.channel
    packages: list[str] = args.PACKAGES
    only_url = args.url
    jobset = guess_jobset(channel)
    is_channel = jobset.startswith("nixos/")
    as_json = args.json
    all_builds = {}

    for package in packages:
        if package.startswith("python3Packages") or package.startswith("python3.pkgs"):
            logging.error("instead of '%s', you want python3XPackages... (replace X)", package)
            continue
        package_name = guess_packagename(package, args.arch, is_channel)
        ident = f"{jobset}/{package_name}"
        if only_url:
            print(get_url(ident))
            continue

        resp = fetch_data(ident)
        builds = list(parse_build_html(resp))
        all_builds[package] = builds

        if not as_json:
            latest = builds[0]
            match latest["icon"]:
                case "✖":
                    print(Fore.RED, end="")
                case "⚠" | "⧖":
                    print(Fore.YELLOW, end="")
                case "✔":
                    print(Fore.GREEN, end="")
            print(f"Build Status for {package_name} on {channel}")
            print_buildreport(latest)
            if not latest["success"] and latest["evals"] and not args.short:
                print()
                print("Last Builds:")
                for build in builds[1:]:
                    print_buildreport(build)
    if as_json:
        print(json.dumps(all_builds))


if __name__ == "__main__":
    main()
