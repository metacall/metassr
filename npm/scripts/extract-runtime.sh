#!/bin/sh
set -e

P=/payload
mkdir -p "$P"
find "$P" -mindepth 1 -exec rm -rf {} +
mkdir -p "$P/bin" "$P/lib/node_modules" "$P/share/metacall/configurations" "$P/node" "$P/lib/python3.14/dist-packages"

cp -a /usr/local/lib/libmetacall.so \
	/usr/local/lib/libnode_loader.so \
	/usr/local/lib/libts_loader.so \
	/usr/local/lib/libpy_loader.so \
	/usr/local/lib/librapid_json_serial.so \
	/usr/local/lib/libplugin_extension.so \
	/usr/local/lib/libplthook_detour.so \
	/usr/local/lib/libext_loader.so \
	"$P/lib/"
cp -a /usr/local/lib/bootstrap.js /usr/local/lib/bootstrap.ts "$P/lib/"

for m in acorn acorn-jsx eslint-visitor-keys espree metacall; do
	cp -a "/usr/local/lib/node_modules/$m" "$P/lib/node_modules/"
done

cp -a /usr/local/share/metacall/configurations/global.json \
	/usr/local/share/metacall/configurations/node_loader.json \
	"$P/share/metacall/configurations/"

# Python runtime: libpython + stdlib (pyc stripped) + the metacall bindings.
cp -a /usr/lib/x86_64-linux-gnu/libpython3.14.so.1.0 "$P/lib/"
ln -s libpython3.14.so.1.0 "$P/lib/libpython3.14.so.1"
ln -s libpython3.14.so.1 "$P/lib/libpython3.14.so"
cp -a /usr/lib/python3.14/. "$P/lib/python3.14/"
find "$P/lib/python3.14" -name __pycache__ -type d -prune -exec rm -rf {} +
cp -a /usr/local/lib/python3.14/dist-packages/metacall \
	/usr/local/lib/python3.14/dist-packages/metacall-0.5.4.dist-info \
	"$P/lib/python3.14/dist-packages/"

cp -aL /usr/lib/x86_64-linux-gnu/libstdc++.so.6 \
	/usr/lib/x86_64-linux-gnu/libgcc_s.so.1 \
	"$P/node/"

echo "extracted runtime: $(du -sh "$P" | cut -f1)"
