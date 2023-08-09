FROM rust

RUN set -ex && export DEBIAN_FRONTEND=noninteractive && \
    apt update && apt install -y libudev-dev && \
    rustup target add thumbv6m-none-eabi &&  \
    cargo install --locked flip-link probe-run

WORKDIR /src
