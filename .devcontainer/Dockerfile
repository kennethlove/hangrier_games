FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# System deps for Dioxus desktop targets, OpenSSL, and SurrealDB CLI
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        libgtk-3-dev \
        libwebkit2gtk-4.1-dev \
        libsoup-3.0-dev \
        libjavascriptcoregtk-4.1-dev \
        libxdo-dev \
        clang \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

# Add wasm32 target as the vscode user
USER vscode
RUN rustup target add wasm32-unknown-unknown \
    && rustup component add clippy rustfmt rust-src

# Pre-install Dioxus CLI (matches justfile pin)
RUN cargo install dioxus-cli@0.6.2 --locked

# Install SurrealDB CLI for migrations / debugging
RUN curl -sSf https://install.surrealdb.com | sh -s -- --version v2.1.4 \
    || true
ENV PATH="/home/vscode/.surrealdb:${PATH}"

USER root
