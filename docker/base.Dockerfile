# MetaSSR base image: the `metassr` CLI + MetaCall runtime + Node/npm.
#
# Built for linux/amd64. Multi-arch (linux/arm64) is tracked in #193.
#
# NOTE: the `builder` stage compiles `metassr` from source so the image is
# self-contained. Once `metassr` binaries are published via GitHub Releases,
# replace that stage with a download/copy of the prebuilt binary (tracked in #193).

ARG METACALL_VERSION=0.9.23

# MetaCall runtime, also the source of /usr/local for the builder.
FROM metacall/core:${METACALL_VERSION}-runtime AS metacall

# Build stage
FROM rust:slim-trixie AS builder

RUN apt-get update \
	&& DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		ca-certificates \
		git \
		wget \
		pkg-config \
		cmake \
		gcc \
		libclang-dev \
	&& rm -rf /var/lib/apt/lists/*

COPY --from=metacall /usr/local /usr/local

WORKDIR /src
COPY . .
RUN cargo build --release --locked

# Runtime stage
FROM metacall/core:${METACALL_VERSION}-runtime AS runtime

ARG METACALL_VERSION=0.9.23
ARG METASSR_VERSION=1.0.0-alpha

RUN apt-get update \
	&& DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		npm \
	&& rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/metassr /usr/local/bin/metassr

LABEL org.opencontainers.image.title="MetaSSR" \
	org.opencontainers.image.description="MetaSSR base image: CLI + MetaCall runtime + Node/npm" \
	org.opencontainers.image.source="https://github.com/metacall/metassr" \
	org.metacall.metassr.version="${METASSR_VERSION}" \
	org.metacall.metacall.version="${METACALL_VERSION}"

WORKDIR /app
ENTRYPOINT ["metassr"]
