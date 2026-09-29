# Containers

MetaSSR can run in a container, so you don't have to install Node and the CLI by hand on every machine.

> **Status:** the app image is available for linux/amd64.

## The image

There is one kind of image:

- **App image** — your built app, packaged with Node + npm and the `metassr` CLI (installed from npm). No source, no Rust toolchain.

People building MetaSSR itself use `Dockerfile.dev` instead.

## What the app image is

Think of it as a machine that already has MetaSSR installed, ready to serve your app:

| | |
| --- | --- |
| Base OS | Debian 13 (trixie) |
| Node / npm | v22 |
| MetaSSR CLI | `metassr` from npm, on `PATH` |
| Working dir | `/app` |

It is the same set of dependencies the [installation guide](./installation.md) asks you to install by hand, just baked into an image. It installs the published `metassr` package with `npm install -g metassr`, which bundles the MetaCall runtime, Node loader and esbuild (`@metassr/linux-x64-gnu`).

The npm runtime payload ships MetaCall's Node, TypeScript and Python loaders; Python API routes run on the bundled Python 3.14 runtime.

It is **NOT**:

- an installer — it doesn't put `metassr` on your host;
- an image for building & developing MetaSSR itself (that's `Dockerfile.dev`).

## Building your app image

One generic Dockerfile covers any app: `docker/app.Dockerfile`. The build context is your app directory.

```sh
docker build -f docker/app.Dockerfile -t my-app path/to/my-app
docker run --rm -p 8080:8080 my-app
```

The build stage installs the app's npm dependencies and runs `metassr build`. The runtime stage copies `dist/`, `src/api` and `metassr.toml` (plus `static/` if you have it) into a fresh image.

`BUILD_TYPE` selects the build target and how the runtime serves it:

- `ssr` (default) — `metassr build -t ssr`, served with `metassr start`.
- `ssg` — `metassr build -t ssg`, served as static files with `metassr start --serve`.

```sh
docker build --build-arg BUILD_TYPE=ssg -f docker/app.Dockerfile -t my-app-ssg path/to/my-app
```

`METASSR_VERSION` pins the `metassr` npm package version. It defaults to a published release; pass it explicitly for reproducible builds:

```sh
docker build --build-arg METASSR_VERSION=1.0.0-alpha.1 -f docker/app.Dockerfile -t my-app path/to/my-app
```

Two things to remember:

- `metassr start` scans `src/api` to register API routes, so the runtime image needs that directory too.
- JavaScript (and TypeScript) API routes work in containers; Python API routes run on the bundled Python 3.14 — list their packages in `requirements.txt` and the app image installs them.

## CI

The [`Docker` workflow](../../.github/workflows/docker.yml) builds and smoke-tests the SSR app image, the SSG app image and the `sales-dashboard` example. The [`Deploy Site` workflow](../../.github/workflows/pages.yml) builds the SSG output for this site's landing page. Both pin `METASSR_VERSION` to the version in `npm/cli/package.json`.

## Notes

- **Architecture:** linux/amd64 only for now because the npm runtime payload is linux-x64 (glibc); linux/arm64 is tracked in [#193](https://github.com/metacall/metassr/issues/193).