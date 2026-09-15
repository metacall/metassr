# Docker

Dockerfiles for MetaSSR containers.

- [`base.Dockerfile`](./base.Dockerfile) — the MetaSSR base image: MetaCall + Node/npm + the `metassr` CLI.
- [`app.Dockerfile`](./app.Dockerfile) — generic app image; build it with your app directory as context.

See the [containers guide](../docs/getting-started/docker.md) for what these images are and how to use them.
