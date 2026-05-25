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

# Install SurrealDB into /usr/local/bin (must run as root for write access).
# No '|| true' — fail the build if install fails so we don't end up with a
# broken devcontainer that silently misses the binary.
RUN curl -sSf https://install.surrealdb.com | sh -s -- --version v2.3.2 \
    && surreal version

# Add clippy and rustfmt as the vscode user
USER vscode
RUN rustup component add clippy rustfmt rust-src

# Install Mailpit for email testing in development
RUN curl -sSfL https://github.com/axllent/mailpit/releases/download/v1.30.0/mailpit-linux-amd64.tar.gz \
    | tar xz -C /usr/local/bin mailpit \
    && mailpit version

USER root
