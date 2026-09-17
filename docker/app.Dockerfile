# MetaSSR app image: builds a MetaSSR app and serves it with the base image.
#
# The build context is the app directory, for example:
#   docker build -f docker/app.Dockerfile -t my-app path/to/my-app
#
# The base image is built from docker/base.Dockerfile.
# See docs/getting-started/docker.md for details.
#
# BUILD_TYPE selects the build target and how the runtime serves it:
#   ssr (default) -> `metassr build -t ssr` + `metassr start`
#   ssg           -> `metassr build -t ssg` + `metassr start --serve`

ARG BASE_IMAGE=metacall/metassr:1.0.0-alpha
ARG BUILD_TYPE=ssr

FROM ${BASE_IMAGE} AS build

ARG BUILD_TYPE

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

FROM ${BASE_IMAGE} AS runtime

ARG BUILD_TYPE
ENV METASSR_BUILD_TYPE=${BUILD_TYPE}

WORKDIR /app

# Install Python dependencies for polyglot API routes, if any.
COPY --from=build /out/requirements.txt /tmp/requirements.txt
RUN if [ -s /tmp/requirements.txt ]; then \
		apt-get update \
		&& DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends python3-pip \
		&& python3 -m pip install --break-system-packages --no-cache-dir -r /tmp/requirements.txt \
		&& rm -rf /var/lib/apt/lists/* /tmp/requirements.txt; \
	fi

COPY --from=build /out/ ./

EXPOSE 8080

# The base image entrypoint is `metassr`; replace it with a tiny launcher so SSG
# builds serve the pre-rendered files (`metassr start --serve`) while SSR builds
# run the renderer (`metassr start`).
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
