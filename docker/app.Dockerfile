# MetaSSR app image: builds a MetaSSR app and serves it.
#
# The build context is the app directory, for example:
#   docker build -f docker/app.Dockerfile -t my-app path/to/my-app
#
# The image installs the published `metassr` CLI from npm (no base image, no
# Rust build). It ships Node + npm and Python 3 (for installing the app's
# requirements.txt into the bundled Python runtime).
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

# Collect only what the runtime needs. `static/`, `src/api` and `requirements.txt`
# are optional, so guard each one.
RUN mkdir -p /out \
	&& cp -r dist /out/ \
	&& if [ -d src/api ]; then mkdir -p /out/src && cp -r src/api /out/src/; fi \
	&& if [ -f metassr.toml ]; then cp metassr.toml /out/; fi \
	&& if [ -d static ]; then cp -r static /out/; fi \
	&& if [ -f requirements.txt ]; then cp requirements.txt /out/; else : > /out/requirements.txt; fi

FROM node:22-trixie-slim AS runtime

ARG BUILD_TYPE
ARG METASSR_VERSION
ENV METASSR_BUILD_TYPE=${BUILD_TYPE}

RUN apt-get update \
	&& DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		python3 \
		python3-pip \
	&& rm -rf /var/lib/apt/lists/*

RUN npm install -g "metassr@${METASSR_VERSION}"

WORKDIR /app

# Install Python dependencies for polyglot API routes, if any. The loader runs
# the bundled Python 3.14 (from the npm payload), so the system pip (3.13)
# installs cp314 wheels straight into the payload's site-packages.
COPY --from=build /out/requirements.txt /tmp/requirements.txt
RUN if [ -s /tmp/requirements.txt ]; then \
		PAYLOAD="$(dirname "$(find "$(npm root -g)" -path "*/@metassr/linux-x64-gnu/package.json" | head -1)")" \
		&& python3 -m pip install --break-system-packages --no-cache-dir \
			--target "$PAYLOAD/lib/python3.14/site-packages" \
			--python-version 3.14 --implementation cp --only-binary=:all: \
			--platform manylinux_2_28_x86_64 \
			-r /tmp/requirements.txt \
		&& rm -f /tmp/requirements.txt; \
	fi

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