# Docker

Dockerfiles for MetaSSR containers.

- [`base.Dockerfile`](./base.Dockerfile) — the MetaSSR base image: MetaCall + Node/npm + the `metassr` CLI.
- [`app.Dockerfile`](./app.Dockerfile) — generic app image. Pass `--build-arg BUILD_TYPE=ssr` (default) or `ssg` to pick how the app is built and served.

See the [containers guide](../docs/getting-started/docker.md) for what these images are and how to use them.
