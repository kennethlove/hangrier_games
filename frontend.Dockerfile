ARG RUST_VERSION=1.86.0

# Build stage for Tailwind
FROM node:18-slim AS css-builder
WORKDIR /app

COPY web/assets/src/ ./assets/src/
COPY web/tailwind.config.js ./tailwind.config.js
COPY web/assets/package*.json ./

RUN npm install tailwindcss @tailwindcss/cli
RUN npx @tailwindcss/cli -i ./assets/src/main.css -o ./assets/dist/main.css

# Build stage for Dioxus
FROM rust:${RUST_VERSION}-slim-bookworm AS rust-builder
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

RUN cargo install dioxus-cli --locked

COPY web/Cargo.toml ./web/Cargo.toml
COPY web/Dioxus.toml ./web/Dioxus.toml
COPY web/src/ ./web/src/
COPY web/assets/images/ ./web/assets/images/
COPY shared/ ./shared/
COPY game ./game/
COPY --from=css-builder /app/assets/dist/ ./web/assets/dist/

WORKDIR /app/web
RUN dx build --release

# Final state with Nginx to serve static files
FROM nginx:alpine
COPY --from=rust-builder /app/web/target/dx/web/release/web/public/ /usr/share/nginx/html/
COPY web/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
