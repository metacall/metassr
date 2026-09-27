#!/bin/bash
# Regenerate CHANGELOG.md with git-cliff.
#
#   ./scripts/changelog.sh            regenerate the full changelog (unreleased + tagged releases)
#   ./scripts/changelog.sh --preview  print the changelog to stdout instead of writing the file

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="$ROOT/.git-cliff.toml"

if ! command -v git-cliff >/dev/null 2>&1; then
    echo "error: git-cliff is not installed" >&2
    echo "install it with: cargo install git-cliff  (or enter the nix dev shell)" >&2
    exit 1
fi

if [[ "${1:-}" == "--preview" ]]; then
    git-cliff --config "$CONFIG" --output /dev/stdout
else
    git-cliff --config "$CONFIG" --output "$ROOT/CHANGELOG.md"
    echo "CHANGELOG.md regenerated."
fi