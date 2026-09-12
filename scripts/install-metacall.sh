#!/usr/bin/env bash
#
# Install a pinned MetaCall runtime for MetaSSR.
#
# MetaCall publishes prebuilt "distributable" tarballs per platform. This script
# wraps the official installer and pins the runtime version, so local
# development, containers and CI all resolve to the same build.
#
# Usage:
#   scripts/install-metacall.sh [--version <x.y.z|latest>] [--debug] [--force]
#
# Options:
#   --version <v>  MetaCall version to install. Defaults per platform (see below).
#   --debug        Install the debug build (libmetacalld) instead of the release build.
#   --force        Reinstall even if MetaCall is already present.
#   -h, --help     Show this help.
#
# Environment:
#   METACALL_VERSION      Overrides the default pinned version.
#   METACALL_INSTALL_REF  Git ref of metacall/install to run (default: master).
#
# Defaults:
#   linux  0.9.11  latest metacall/distributable-linux release
#   macos  0.1.6   latest metacall/distributable-macos release
#
# Note: MetaCall's prebuilt "distributable" tarballs lag behind metacall/core
# tags. 0.9.23 is not installable via install.sh: the latest distributable-linux
# release is v0.9.11, and the v0.9.23 core release assets are stale (0.9.22).
# The genuine 0.9.23 build only ships as the metacall/core:0.9.23-runtime image.
#
# TODO(#192): support installing 0.9.23 (Docker image extraction or source build)
# and bump DEFAULT_VERSION_LINUX back to 0.9.23.

set -euo pipefail

DEFAULT_VERSION_LINUX="0.9.11"
DEFAULT_VERSION_MACOS="0.1.6"
INSTALL_REF="${METACALL_INSTALL_REF:-master}"

DEBUG=0
FORCE=0
REQUESTED_VERSION="${METACALL_VERSION:-}"

info() { printf '[metassr] %s\n' "$*"; }
warn() { printf '[metassr] warning: %s\n' "$*" >&2; }
die() {
    printf '[metassr] error: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Install a pinned MetaCall runtime for MetaSSR.

Usage:
  scripts/install-metacall.sh [--version <x.y.z|latest>] [--debug] [--force]

Options:
  --version <v>  MetaCall version to install. Defaults per platform.
  --debug        Install the debug build (libmetacalld).
  --force        Reinstall even if MetaCall is already present.
  -h, --help     Show this help.

Environment:
  METACALL_VERSION      Overrides the default pinned version.
  METACALL_INSTALL_REF  Git ref of metacall/install to run (default: master).
EOF
}

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --version)
                [ $# -ge 2 ] || die "--version requires an argument"
                REQUESTED_VERSION="$2"
                shift 2
                ;;
            --debug)
                DEBUG=1
                shift
                ;;
            --force)
                FORCE=1
                shift
                ;;
            -h | --help)
                usage
                exit 0
                ;;
            *)
                die "unknown argument: $1"
                ;;
        esac
    done
}

detect_platform() {
    case "$(uname -s)" in
        Linux) PLATFORM_OS="linux" ;;
        Darwin) PLATFORM_OS="macos" ;;
        *) die "unsupported operating system: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64 | amd64) PLATFORM_ARCH="amd64" ;;
        aarch64 | arm64) PLATFORM_ARCH="arm64" ;;
        *) die "unsupported architecture: $(uname -m)" ;;
    esac

    if [ -z "$REQUESTED_VERSION" ]; then
        case "$PLATFORM_OS" in
            linux) REQUESTED_VERSION="$DEFAULT_VERSION_LINUX" ;;
            macos) REQUESTED_VERSION="$DEFAULT_VERSION_MACOS" ;;
        esac
    fi
}

lib_dirs() {
    case "$PLATFORM_OS" in
        linux) printf '%s\n' /usr/local/lib /gnu/lib ;;
        macos) printf '%s\n' /opt/homebrew/lib /usr/local/lib ;;
    esac
}

lib_names() {
    if [ "$DEBUG" -eq 1 ]; then
        case "$PLATFORM_OS" in
            linux) printf '%s\n' libmetacalld.so ;;
            macos) printf '%s\n' libmetacalld.dylib ;;
        esac
    else
        case "$PLATFORM_OS" in
            linux) printf '%s\n' libmetacall.so ;;
            macos) printf '%s\n' libmetacall.dylib ;;
        esac
    fi
}

find_library() {
    local dir name
    while IFS= read -r dir; do
        while IFS= read -r name; do
            if [ -e "$dir/$name" ]; then
                printf '%s\n' "$dir/$name"
                return 0
            fi
        done < <(lib_names)
    done < <(lib_dirs)
    return 1
}

installed_version() {
    local lib target
    lib="$(find_library)" || return 1
    target="$(readlink -f "$lib" 2>/dev/null || readlink "$lib" 2>/dev/null || printf '%s' "$lib")"
    printf '%s\n' "$target" | grep -oE 'metacall-[0-9]+\.[0-9]+\.[0-9]+' | head -n1 | sed 's/^metacall-//'
}

install_metacall() {
    local url args=()
    url="https://raw.githubusercontent.com/metacall/install/${INSTALL_REF}/install.sh"

    if [ "$REQUESTED_VERSION" != "latest" ]; then
        args+=(--version "$REQUESTED_VERSION")
    fi
    if [ "$DEBUG" -eq 1 ]; then
        args+=(--debug)
    fi

    info "installing MetaCall ${REQUESTED_VERSION} (${PLATFORM_OS}/${PLATFORM_ARCH})"
    info "installer: ${url}"
    curl -fsSL "$url" | sh -s -- "${args[@]}"
}

verify() {
    local lib version
    if ! lib="$(find_library)"; then
        die "MetaCall library not found after installation"
    fi

    if version="$(installed_version)" && [ -n "$version" ]; then
        info "MetaCall ${version} installed at ${lib}"
    else
        info "MetaCall installed at ${lib}"
    fi
}

main() {
    parse_args "$@"
    detect_platform

    local existing
    if [ "$FORCE" -eq 0 ] && find_library >/dev/null 2>&1; then
        if existing="$(installed_version)" && [ -n "$existing" ]; then
            info "MetaCall ${existing} already installed (use --force to reinstall)"
        else
            info "MetaCall already installed (use --force to reinstall)"
        fi
        return 0
    fi

    install_metacall
    verify
}

main "$@"
