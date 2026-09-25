#!/bin/sh
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${OUT:-$(pwd)/out}"
PAYLOAD="${PAYLOAD:-$OUT/payload/linux-x64-gnu}"
METACALL_IMAGE="${METACALL_IMAGE:-metacall/core:0.9.24-runtime}"
NODE_VERSION="${NODE_VERSION:-24.20.0}"
NODE_SONAME="${NODE_SONAME:-libnode.so.137}"
METASSR_BIN="${METASSR_BIN:-target/release/metassr}"
VENDOR_BUNDLER="${VENDOR_BUNDLER:-crates/metassr-bundler/vendor}"
LIB_NODE="${LIB_NODE:-$OUT/libnode.so}"
VERSION="${VERSION:-}"

if [ ! -f "$METASSR_BIN" ]; then
	echo "error: metassr binary not found at $METASSR_BIN (build with cargo build --release, or set METASSR_BIN)" >&2
	exit 1
fi

if [ ! -d "$VENDOR_BUNDLER/node_modules/esbuild" ]; then
	echo "error: vendored esbuild not found at $VENDOR_BUNDLER (run npm install there, or set VENDOR_BUNDLER)" >&2
	exit 1
fi

if [ ! -f "$LIB_NODE" ]; then
	echo "building libnode $NODE_VERSION (first run takes 20-40 minutes)..."
	OUT="$(dirname "$LIB_NODE")" NODE_VERSION="$NODE_VERSION" BASE_IMAGE="${LIB_NODE_BASE_IMAGE:-debian:bookworm-slim}" "$HERE/build-libnode.sh"
fi

echo "extracting MetaCall runtime from $METACALL_IMAGE..."
mkdir -p "$PAYLOAD"
docker run --rm \
	--user "$(id -u):$(id -g)" \
	-v "$PAYLOAD:/payload" \
	-v "$HERE/extract-runtime.sh:/extract-runtime.sh:ro" \
	--entrypoint sh \
	"$METACALL_IMAGE" /extract-runtime.sh

cp "$LIB_NODE" "$PAYLOAD/node/$NODE_SONAME"
cp "$METASSR_BIN" "$PAYLOAD/bin/metassr"
cp "$VENDOR_BUNDLER/node_modules/@esbuild/linux-x64/bin/esbuild" "$PAYLOAD/bin/esbuild"
chmod 755 "$PAYLOAD/bin/metassr" "$PAYLOAD/bin/esbuild"

mkdir -p "$PAYLOAD/vendor"
cp -a "$VENDOR_BUNDLER" "$PAYLOAD/vendor/bundler"
rm -f "$PAYLOAD/vendor/bundler/package-lock.json"

cp "$HERE/../linux-x64-gnu/package.json" "$PAYLOAD/package.json"
if [ -n "$VERSION" ]; then
	sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PAYLOAD/package.json" "$HERE/../cli/package.json"
	sed -i "s#\"@metassr/linux-x64-gnu\": \"[^\"]*\"#\"@metassr/linux-x64-gnu\": \"$VERSION\"#" "$HERE/../cli/package.json"
fi

name="metassr-runtime-linux-x64-gnu${VERSION:+-$VERSION}"
tar -cf "$OUT/$name.tar" -C "$PAYLOAD" .
gzip -9 -f "$OUT/$name.tar"
sha256sum "$OUT/$name.tar.gz" > "$OUT/$name.tar.gz.sha256"

echo "payload: $PAYLOAD"
echo "tarball: $OUT/$name.tar.gz ($(du -h "$OUT/$name.tar.gz" | cut -f1))"
cat "$OUT/$name.tar.gz.sha256"
