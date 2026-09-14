# MetaSSR app image: builds a MetaSSR app and serves it with the base image.
#
# The build context is the app directory, for example:
#   docker build -f docker/app.Dockerfile -t my-app path/to/my-app
#
# The base image is built from docker/base.Dockerfile.
# See docs/getting-started/docker.md for details.

FROM metacall/metassr:1.0.0-alpha AS build

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .
RUN metassr build -t ssr

# Collect only what the runtime needs. `static/`, `src/api` and `requirements.txt`
# are optional, so guard each one.
RUN mkdir -p /out \
	&& cp -r dist /out/ \
	&& if [ -d src/api ]; then mkdir -p /out/src && cp -r src/api /out/src/; fi \
	&& if [ -f metassr.toml ]; then cp metassr.toml /out/; fi \
	&& if [ -d static ]; then cp -r static /out/; fi \
	&& if [ -f requirements.txt ]; then cp requirements.txt /out/; else : > /out/requirements.txt; fi

FROM metacall/metassr:1.0.0-alpha AS runtime

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
CMD ["start"]
