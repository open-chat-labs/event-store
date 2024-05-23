# To build run 'docker build . -t openchat'
FROM ubuntu:22.04 as builder
SHELL ["bash", "-c"]

ARG git_commit_id
ARG rustflags
ARG rust_version=1.78.0

ENV GIT_COMMIT_ID=$git_commit_id
ENV TZ=UTC
ENV DFX_VERSION=0.20.0
ENV PATH="/root/.local/share/dfx/bin:$PATH"
ENV RUSTFLAGS=$rustflags

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone && \
    apt -yq update && \
    apt -yqq install --no-install-recommends curl ca-certificates build-essential

# Install Rust and Cargo in /opt
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/cargo/bin:/opt/cargo/bin:$PATH

RUN curl --fail https://sh.rustup.rs -sSf \
        | sh -s -- -y --default-toolchain ${rust_version}-x86_64-unknown-linux-gnu --no-modify-path && \
    rustup default ${rust_version}-x86_64-unknown-linux-gnu && \
    rustup target add wasm32-unknown-unknown

RUN DFXVM_INIT_YES=true sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

COPY . /build
WORKDIR /build

RUN dfx build --ic --check
