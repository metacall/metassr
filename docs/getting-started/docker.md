# Containers

MetaSSR can run in a container, so you don't have to install MetaCall, Node and the CLI by hand on every machine.

> **Status:** the base image and the app image are available for linux/amd64.

## The images

There are two kinds of images:

- **Base image** — a MetaSSR installation, packaged. MetaCall, Node + npm, and the `metassr` CLI. No app, no source.
- **App image** — your built app on top of the base image.

People building MetaSSR itself use `Dockerfile.dev` instead.

## What the base image is

Think of it as a machine that already has MetaSSR installed:

| | |
| --- | --- |
| Base OS | Debian 13 (trixie) |
| MetaCall | 0.9.24 |
| Node / npm | v20 / 9.x |
| MetaSSR CLI | `metassr` on `PATH` |
| Working dir | `/app` |
| Entrypoint | `metassr` |

It is the same set of dependencies the [installation guide](./installation.md) asks you to install by hand, just baked into an image.

It is **NOT**:

- an installer — it doesn't put `metassr` on your host;
- an app image — it has no `dist/`, `src/api` or `metassr.toml`;
- an image for building & developing MetaSSR itself (that's `Dockerfile.dev`).

## Using the base image

Run the CLI:

```sh
docker run --rm metacall/metassr:1.0.0-alpha --version
```

Build an app without installing anything on your host:

```sh
docker run --rm -v "$PWD":/app -w /app --entrypoint sh \
  metacall/metassr:1.0.0-alpha \
  -c "npm install && metassr build -t ssr"
```

The entrypoint is `metassr`, so use `--entrypoint sh` when you want a shell.

## Building your own app image

An app image is a two-stage build, and one generic Dockerfile covers it: `docker/app.Dockerfile`. The build context is your app directory.

```sh
docker build -f docker/app.Dockerfile -t my-app path/to/my-app
docker run --rm -p 8080:8080 my-app
```

The build stage runs `npm install` and `metassr build`. The runtime stage copies `dist/`, `src/api` and `metassr.toml` (plus `static/` if you have it) into a fresh base image.

`BUILD_TYPE` selects the build target and how the runtime serves it:

- `ssr` (default) — `metassr build -t ssr`, served with `metassr start`.
- `ssg` — `metassr build -t ssg`, served as static files with `metassr start --serve`.

```sh
docker build --build-arg BUILD_TYPE=ssg -f docker/app.Dockerfile -t my-app-ssg path/to/my-app
```

`BASE_IMAGE` overrides the base image tag if you are not using the default
`metacall/metassr:1.0.0-alpha`.

Two things to remember:

- `metassr start` scans `src/api` to register API routes, so the runtime image needs that directory too.
- Python API routes bring their own packages. List them in `requirements.txt` and the app image installs them; the base image keeps Python bare on purpose.

## Building the base image

The base image is built locally; nothing is pushed to a registry:

```sh
docker build --platform linux/amd64 -f docker/base.Dockerfile -t metacall/metassr:1.0.0-alpha .
```

## CI

The [`Docker` workflow](../../.github/workflows/docker.yml) builds the base image, then builds and smoke-tests the SSR app image, the SSG app image and the `sales-dashboard` polyglot example.

The base image build uses [`cargo-chef`](https://github.com/LukeMathWalker/cargo-chef) so dependencies are compiled in their own layer. CI caches that layer with the GitHub Actions cache backend (`mode=max`), which means a source-only change does not recompile every dependency.

## Notes

- **Architecture:** linux/amd64 only for now; linux/arm64 is tracked in [#193](https://github.com/metacall/metassr/issues/193).
- **Size:** large (~2.8 GB) because it bundles the MetaCall runtime.
- **Temporary:** the image currently compiles `metassr` from source in a builder stage. Once release binaries exist it will just download one ([#193](https://github.com/metacall/metassr/issues/193)).
