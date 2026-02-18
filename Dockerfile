# Multi-arch Build Dockerfile for host-snapshot-agent-rs
# Using cargo-zigbuild for robust cross-compilation

FROM rust:1.93.1-alpine AS builder
RUN apk update
RUN apk add build-base
RUN cargo install --locked cargo-zigbuild

# Pre-install Rust targets
RUN rustup target add x86_64-unknown-linux-gnu && \
    rustup target add aarch64-unknown-linux-gnu && \
    rustup target add x86_64-unknown-linux-musl && \
    rustup target add aarch64-unknown-linux-musl

RUN apk add zig

WORKDIR /app

# Copy the source code
COPY hsnap hsnap
COPY hsnap-purl-plugin hsnap-purl-plugin
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

# Build Argument for the target (default to x86_64-gnu)
ARG TARGET=x86_64-unknown-linux-gnu

# Build the project using zigbuild
# This automatically handles the missing cross-compilers like 'aarch64-linux-gnu-gcc'
RUN cargo zigbuild --release --target $TARGET

# Output the binary to a standard location
RUN mkdir -p /output && cp target/$TARGET/release/hsnap /output/hsnap

FROM scratch AS export
COPY --from=builder /output/hsnap .