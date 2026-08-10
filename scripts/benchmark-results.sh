#!/usr/bin/env bash
#
# Manage the persistent benchmark results stored on the `benchmark-data` branch.
#
# Usage:
#   benchmark-results.sh fetch <dir>
#   benchmark-results.sh store <dir> <sha> <branch> <results.json>
#
# fetch clones (or bootstraps) the `benchmark-data` branch into <dir>.
# store writes <sha>.json and latest.json under results/<branch>/ and pushes
# the update back to the `benchmark-data` branch.
#
# Requires GITHUB_REPOSITORY, GITHUB_TOKEN and GITHUB_ACTOR to be set.

set -euo pipefail

DATA_BRANCH="benchmark-data"
REMOTE_URL="${BENCHMARK_REMOTE_URL:-https://x-access-token:${GITHUB_TOKEN}@github.com/${GITHUB_REPOSITORY}.git}"

fetch_branch() {
    local dir="$1"

    if [ ! -d "$dir/.git" ]; then
        rm -rf "$dir"
        mkdir -p "$dir"
        git init -q "$dir"
        git -C "$dir" remote add origin "$REMOTE_URL"
    fi

    git -C "$dir" fetch -q origin "$DATA_BRANCH" 2>/dev/null || true

    if git -C "$dir" rev-parse --verify "refs/remotes/origin/$DATA_BRANCH" >/dev/null 2>&1; then
        git -C "$dir" checkout -q -B "$DATA_BRANCH" "origin/$DATA_BRANCH"
    else
        git -C "$dir" checkout -q --orphan "$DATA_BRANCH"
        git -C "$dir" rm -rf -q --ignore-unmatch .
    fi

    echo "fetched $DATA_BRANCH into $dir"
}

store_results() {
    local dir="$1"
    local sha="$2"
    local branch="$3"
    local results_file="$4"
    local results_dir

    if [ ! -f "$results_file" ]; then
        echo "error: results file not found: $results_file" >&2
        exit 1
    fi

    results_dir="$dir/results/$branch"
    mkdir -p "$results_dir"
    cp "$results_file" "$results_dir/$sha.json"
    cp "$results_file" "$results_dir/latest.json"

    git -C "$dir" config user.name "${GITHUB_ACTOR:-github-actions[bot]}"
    git -C "$dir" config user.email "${GITHUB_ACTOR:-github-actions[bot]}@users.noreply.github.com"

    git -C "$dir" add -A
    if git -C "$dir" diff --cached --quiet; then
        echo "no changes to commit"
        return 0
    fi

    git -C "$dir" commit -q -m "benchmark results for $sha ($branch)"
    git -C "$dir" push -q origin "$DATA_BRANCH:$DATA_BRANCH"

    echo "stored benchmark results for $sha ($branch) on $DATA_BRANCH"
}

main() {
    if [ $# -lt 1 ]; then
        echo "usage: benchmark-results.sh {fetch <dir> | store <dir> <sha> <branch> <results.json>}" >&2
        exit 1
    fi

    if [ -z "${GITHUB_REPOSITORY:-}" ] || [ -z "${GITHUB_TOKEN:-}" ]; then
        echo "error: GITHUB_REPOSITORY and GITHUB_TOKEN must be set" >&2
        exit 1
    fi

    local cmd="$1"
    shift

    case "$cmd" in
        fetch)
            if [ $# -ne 1 ]; then
                echo "usage: benchmark-results.sh fetch <dir>" >&2
                exit 1
            fi
            fetch_branch "$1"
            ;;
        store)
            if [ $# -ne 4 ]; then
                echo "usage: benchmark-results.sh store <dir> <sha> <branch> <results.json>" >&2
                exit 1
            fi
            store_results "$@"
            ;;
        *)
            echo "error: unknown command: $cmd" >&2
            exit 1
            ;;
    esac
}

main "$@"
