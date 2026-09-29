#!/bin/sh
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
METACALL_IMAGE="${METACALL_IMAGE:-metacall/core:0.9.24-runtime}"
METASSR_BIN="${METASSR_BIN:-$ROOT/target/release/metassr}"
TAG="${TAG:-metassr-build}"

DOCKER_BUILDKIT="${DOCKER_BUILDKIT:-0}" \
docker build \
	--build-arg "METACALL_IMAGE=$METACALL_IMAGE" \
	-t "$TAG" \
	-f "$HERE/Dockerfile.metassr" \
	"$ROOT"

container="$(docker create "$TAG")"
trap 'docker rm -f "$container" >/dev/null 2>&1 || true' EXIT INT TERM

mkdir -p "$(dirname "$METASSR_BIN")"
docker cp "${container}:/src/target/release/metassr" "$METASSR_BIN"
chmod 755 "$METASSR_BIN"
ls -lh "$METASSR_BIN"
