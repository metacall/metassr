# Docker

Dockerfiles for MetaSSR containers.

- [`app.Dockerfile`](./app.Dockerfile) — generic app image. It installs the `metassr` CLI from npm and ships Node + npm. Pass `--build-arg BUILD_TYPE=ssr` (default) or `ssg` to pick how the app is built and served.

See the [containers guide](../docs/getting-started/docker.md) for what this image is and how to use it.