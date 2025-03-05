#!/usr/bin/env nix-shell
#! nix-shell -p bash cargo cargo-edit -i bash
# shellcheck shell=bash

set -euo pipefail

NEWVERSION="$1"

if [[ $NEWVERSION == "" ]]; then
    echo "No version specified!"
    exit 1
fi

cargo set-version "$NEWVERSION"
cargo build

# Commit and tag the update
git add --patch Cargo.toml Cargo.lock
git commit -m "build(release): v${NEWVERSION}"
git push origin "$(git branch --show-current)"
gh release create "v${NEWVERSION}" -t "v${NEWVERSION}" --target "$(git branch --show-current)" --generate-notes
