#!/usr/bin/env nix-shell
#! nix-shell -p bash cargo cargo-edit -i bash
# shellcheck shell=bash

set -euo pipefail

NEWVERSION="$1"
REPO=nix-community/hydra-check

if [[ $NEWVERSION == "" ]]; then
    echo "No version specified!"
    exit 1
fi

cargo set-version "$NEWVERSION"
cargo build

# commit the update
git add --patch Cargo.toml Cargo.lock
git commit -m "build(release): v${NEWVERSION}"

# push & tag
set -x
CURRENT_BRANCH="$(git branch --show-current)"
REMOTE=$(git remote --verbose | grep "REPO" | uniq)
git push "$REMOTE" "$CURRENT_BRANCH"
gh release --repo "$REPO" create "v${NEWVERSION}" -t "v${NEWVERSION}" --prerelease --target "$CURRENT_BRANCH" --generate-notes
