"""usage: hydra-check [options] PACKAGE [CHANNEL]

options:
    --arch=SYSTEM        system architecture to check [default: x86_64-linux]
    --json               write builds in machine-readable format
    --short              write only the latest build even if last build failed


CHANNEL: channel to use:
    master
    unstable (Default)
    19.03 19.09 20.03
    nixos/release-19.09


"""
from bs4 import BeautifulSoup
import requests
import json
from sys import exit

# guess functions are intended to be fast without external queries
def guess_jobset(channel):
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


def guess_packagename(package, arch, is_channel):
    # TODO: maybe someone provides the architecture in the package name?
    if package.startswith("nixpkgs.") or package.startswith("nixos."):
        # we assume user knows the full package name
        return f"{package}.{arch}"
    elif is_channel:
        # we simply guess, that the user searches for a package and not a test
        return f"nixpkgs.{package}.{arch}"
    else:
        return f"{package}.{arch}"


def fetch_data(ident):
    # https://hydra.nixos.org/job/nixos/release-19.09/nixpkgs.hello.x86_64-linux/latest
    # https://hydra.nixos.org/job/nixos/release-19.09/nixos.tests.installer.simpleUefiGrub.aarch64-linux/all
    # https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-linux/all
    url = f"https://hydra.nixos.org/job/{ident}/all"
    resp = requests.get(url)
    if resp.status_code == 404:
        print(f"package {ident} not found at url {url}")
        exit(1)
    return resp.text


def parse_build_html(data):
    doc = BeautifulSoup(data, features="html.parser")
    for row in doc.find("tbody").find_all("tr"):
        status, build, timestamp, name, arch = row.find_all("td")
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
        }


def print_build(build):
    extra = "" if build["success"] else f" ({build['status']})"
    print(
        f"{build['icon']}{extra} {build['name']} from {build['timestamp'].split('T')[0]} - {build['build_url']}"
    )


def main():
    from docopt import docopt

    args = docopt(__doc__)
    channel = args["CHANNEL"] or "unstable"
    package = args["PACKAGE"]
    arch = args["--arch"]
    jobset = guess_jobset(channel)
    is_channel = jobset.startswith("nixos/")
    package_name = guess_packagename(package, arch, is_channel)
    ident = f"{jobset}/{package_name}"
    resp = fetch_data(ident)
    builds = list(parse_build_html(resp))
    if not args["--json"]:
        latest = builds[0]
        print_build(latest)
        if not latest["success"] and not args["--short"]:
            print()
            print("Last Builds:")
            for build in builds[1:]:
                print_build(build)
    else:
        print(json.dumps(builds))


if __name__ == "__main__":
    main()
