# MetaSSR base image: the `metassr` CLI + MetaCall runtime + Node/npm.
#
# Built for linux/amd64. Multi-arch (linux/arm64) is tracked in #193.
#
# NOTE: the `builder` stage compiles `metassr` from source so the image is
# self-contained. Once `metassr` binaries are published via GitHub Releases,
# replace that stage with a download/copy of the prebuilt binary (tracked in #193).
#
# Dependencies are compiled with cargo-chef so the expensive dependency layer
# survives source-only changes. `cargo chef cook` is a normal image layer on
# purpose: that is what lets CI restore it with a GitHub Actions `mode=max`
# layer cache. BuildKit `type=cache` mounts are intentionally not used for the
# Cargo registry/target here, because cache mounts are not exported to the GHA
# cache backend and would move the dependency artifacts out of the cacheable
# layer (see https://docs.docker.com/build/cache/backends/gha/#cache-mounts).

ARG METACALL_VERSION=0.9.23
ARG CARGO_CHEF_VERSION=0.1.78

# MetaCall runtime, also the source of /usr/local for the builder.
FROM metacall/core:${METACALL_VERSION}-runtime AS metacall

# Chef stage: Rust toolchain + cargo-chef + MetaCall libraries.
FROM rust:slim-trixie AS chef

ARG CARGO_CHEF_VERSION=0.1.78

RUN apt-get update \
	&& DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		ca-certificates \
		git \
		wget \
		curl \
		xz-utils \
		pkg-config \
		cmake \
		gcc \
		libclang-dev \
	&& rm -rf /var/lib/apt/lists/*

RUN curl -fsSL "https://github.com/LukeMathWalker/cargo-chef/releases/download/v${CARGO_CHEF_VERSION}/cargo-chef-x86_64-unknown-linux-gnu.tar.xz" \
	| tar -xJ -C /usr/local/bin --strip-components=1 cargo-chef-x86_64-unknown-linux-gnu/cargo-chef

COPY --from=metacall /usr/local /usr/local

WORKDIR /src

# Install the pinned Rust toolchain from rust-toolchain.toml once here, so the
# planner and builder stages reuse it instead of downloading it repeatedly.
COPY rust-toolchain.toml ./
RUN cargo --version

# Planner: turn the workspace manifests into a dependency recipe.
FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder: compile dependencies (cached), then the workspace.
FROM chef AS builder

COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
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
