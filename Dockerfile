FROM rust:alpine AS builder
WORKDIR /home/rust/src
RUN apk --no-cache add musl-dev
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/rust/src/target \
    cargo build --release
RUN --mount=type=cache,target=/home/rust/src/target \
    cp target/release/cadre /usr/bin

FROM alpine
COPY --from=builder /usr/bin/cadre /usr/bin
USER 1000:1000
ENTRYPOINT ["cadre"]
