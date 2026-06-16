FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Update Rust to latest stable (matches host toolchain)
RUN rustup update stable && rustup default stable \
    && rustup component add clippy rustfmt rust-src

# System deps for OpenSSL and SurrealDB CLI
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        clang \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

# Install Mailpit for email testing in development - arch-aware for ARM Macs
RUN ARCH=$(uname -m | sed 's/aarch64/arm64/;s/x86_64/amd64/') \
    && curl -sSfL "https://github.com/axllent/mailpit/releases/download/v1.30.0/mailpit-linux-${ARCH}.tar.gz" \
    | tar xz -C /usr/local/bin mailpit \
    && mailpit version 2>/dev/null || true

# Install SurrealDB - arch-aware for ARM Macs
RUN ARCH=$(uname -m | sed 's/aarch64/arm64/;s/x86_64/amd64/') \
    && curl -sSfL "https://github.com/surrealdb/surrealdb/releases/download/v3.1.3/surreal-linux-${ARCH}" \
    -o /usr/local/bin/surreal \
    && chmod +x /usr/local/bin/surreal \
    && surreal version 2>/dev/null || true

USER root
