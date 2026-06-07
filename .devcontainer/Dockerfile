FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

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

# Add clippy and rustfmt as the vscode user
USER vscode
RUN rustup component add clippy rustfmt rust-src

USER root
