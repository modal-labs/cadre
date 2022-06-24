# syntax=docker/dockerfile:1.4

FROM rust:latest
WORKDIR /home/root/app
ENV CARGO_INCREMENTAL=0

# copy source code and dependency references
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src src

# make build and copy binaries
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/root/app/target \
    cargo build --release
RUN --mount=type=cache,target=/home/root/app/target \
    cp target/release/cadre /usr/bin

