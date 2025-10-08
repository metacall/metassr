FROM rust:slim-bookworm

# Image descriptor
LABEL copyright.name="Vicente Eduardo Ferrer Garcia" \
	copyright.address="vic798@gmail.com" \
	maintainer.name="Vicente Eduardo Ferrer Garcia" \
	maintainer.address="vic798@gmail.com" \
	vendor="MetaCall Inc." \
	version="0.1"

# Install MetaCall dependencies
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
		ca-certificates \
		git

# Set working directory to root
WORKDIR /root

# Clone and build the project
RUN git clone --branch v0.9.14 https://github.com/metacall/core \
	&& mkdir core/build && cd core/build \
	&& ../tools/metacall-environment.sh release base nodejs python backtrace \
	&& ../tools/metacall-configure.sh release nodejs python ports install \
	&& ../tools/metacall-build.sh release nodejs python ports install \
	&& cd ../.. \
	&& rm -rf core

# Clone and build the project
RUN git clone https://github.com/metacall/metassr \
	&& cd metassr \
	&& cargo build --release
