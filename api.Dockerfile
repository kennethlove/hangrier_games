ARG RUST_VERSION=1.86.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl-dev \
        pkg-config \
        curl \
        ca-certificates \
        git \
        llvm \
        libclang-dev \
        build-essential && \
    rm -rf /var/lib/apt/lists/*

COPY announcers/ announcers/
COPY api/ api/
COPY game/ game/
COPY migrations/ migrations/
COPY schemas/ schemas/
COPY shared/ shared/

RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release --manifest-path api/Cargo.toml

FROM debian:bookworm-slim AS final

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libclang1 \
        curl && \
    rm -rf /var/lib/apt/lists/*

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    api_user

COPY --from=builder /app/api/target/release/api /usr/local/bin/hg_api

RUN chown -R api_user /usr/local/bin/hg_api && \
    chmod +x /usr/local/bin/hg_api

USER api_user
WORKDIR /opt/api

COPY --from=builder /app/schemas /opt/api/schemas
COPY --from=builder /app/migrations /opt/api/migrations

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl --fail http://localhost:3000/api/games/ || exit 1

EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/hg_api"]
