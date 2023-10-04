import argparse
import textwrap


def process_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
            formatter_class=argparse.RawDescriptionHelpFormatter,
            epilog=textwrap.dedent('''\
                Other channels can be:
                    unstable  - alias for nixos/trunk-combined (Default for Linux architectures)
                    master    - alias for nixpkgs/trunk (Default for Darwin architectures)
                    staging   - alias for nixos/staging
                    19.03     - alias for nixos/release-19.03
                    19.09     - alias for nixos/release-19.09
                    20.03     - alias for nixos/release-20.03

                Usually using the above as --channel arguments, should fit most usages.
                However, you can use a verbatim jobset name such as:

                    nixpkgs/nixpkgs-20.03-darwin

                Jobset names can be constructed with the project name (e.g. `nixos/` or `nixpkgs/`)
                followed by a branch name. The available jobsets can be found at:
                * https://hydra.nixos.org/project/nixos
                * https://hydra.nixos.org/project/nixpkgs
            ''')
            )
    parser.add_argument(
        "PACKAGES",
        action="extend",
        nargs="+",
        type=str,
    )
    parser.add_argument(
        "--url",
        action="store_true",
        help="only print the hydra build url, then exit",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="output json",
    )
    parser.add_argument(
        "--short",
        action="store_true",
        help="write only the latest build even if last build failed",
    )
    parser.add_argument(
        "--arch",
        default="x86_64-linux",
        help="system architecture to check",
    )
    parser.add_argument(
        "--channel",
        # Sort of changes to "master" when arch is darwin
        default="unstable",
        help="Channel to check packages for",
    )
    parser.add_argument(
        "--jobset",
        default="",
        help="Jobset to check packages for",
    )
    return parser.parse_args()
