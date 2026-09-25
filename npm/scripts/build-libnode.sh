#!/bin/sh
set -e

NODE_VERSION="${NODE_VERSION:-24.20.0}"
BASE_IMAGE="${BASE_IMAGE:-debian:bookworm-slim}"
OUT="${OUT:-$(pwd)/out/libnode}"
WORK="${WORK:-$(dirname "$OUT")/libnode-work}"
JOBS="${JOBS:-8}"

mkdir -p "$OUT" "$WORK"

docker run --rm --name "metassr-libnode-$$" \
	-v "$WORK:/build" \
	-w /build \
	--entrypoint sh \
	"$BASE_IMAGE" -c "
set -e
apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
	build-essential ca-certificates curl python3 xz-utils
curl -fsSL https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}.tar.xz -o node.tar.xz
tar -xJf node.tar.xz
cd node-v${NODE_VERSION}
./configure --shared --without-npm --without-corepack || {
	apt-get install -y -qq --no-install-recommends python3.13
	python3.13 ./configure --shared --without-npm --without-corepack
}
make -j${JOBS}
cp out/Release/libnode.so.* /build/libnode.so
strip -s /build/libnode.so
"

cp "$WORK/libnode.so" "$OUT/libnode.so"
ls -lh "$OUT/libnode.so"
