# MetaSSR app image: builds a MetaSSR app and serves it.
#
# The build context is the app directory, for example:
#   docker build -f docker/app.Dockerfile -t my-app path/to/my-app
#
# The image installs the published `metassr` CLI from npm (no base image, no
# Rust build). It ships Node + npm. The npm runtime payload bundles MetaCall's
# Node and TypeScript loaders; Python is not included yet.
#
# BUILD_TYPE selects the build target and how the runtime serves it:
#   ssr (default) -> `metassr build -t ssr` + `metassr start`
#   ssg           -> `metassr build -t ssg` + `metassr start --serve`
#
# METASSR_VERSION pins the `metassr` npm package version. See
# docs/getting-started/docker.md for details.

ARG METASSR_VERSION=1.0.0-alpha.1
ARG BUILD_TYPE=ssr

FROM node:22-trixie-slim AS build

ARG BUILD_TYPE
ARG METASSR_VERSION

RUN npm install -g "metassr@${METASSR_VERSION}"

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .
RUN metassr build -t "${BUILD_TYPE}"

# Collect only what the runtime needs. `static/` and `src/api` are optional,
# so guard each one.
RUN mkdir -p /out \
	&& cp -r dist /out/ \
	&& if [ -d src/api ]; then mkdir -p /out/src && cp -r src/api /out/src/; fi \
	&& if [ -f metassr.toml ]; then cp metassr.toml /out/; fi \
	&& if [ -d static ]; then cp -r static /out/; fi

FROM node:22-trixie-slim AS runtime

ARG BUILD_TYPE
ARG METASSR_VERSION
ENV METASSR_BUILD_TYPE=${BUILD_TYPE}

RUN npm install -g "metassr@${METASSR_VERSION}"

WORKDIR /app

COPY --from=build /out/ ./

EXPOSE 8080

# Launcher: SSG builds serve the pre-rendered files (`metassr start --serve`)
# while SSR builds run the renderer (`metassr start`).
RUN printf '%s\n' \
	'#!/bin/sh' \
	'set -e' \
	'if [ "$METASSR_BUILD_TYPE" = "ssg" ]; then' \
	'	exec metassr start --serve' \
	'else' \
	'	exec metassr start' \
	'fi' \
	> /usr/local/bin/metassr-run \
	&& chmod +x /usr/local/bin/metassr-run

ENTRYPOINT ["/usr/local/bin/metassr-run"]