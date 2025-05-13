ARG RUST_VERSION=1.86.0

# Build stage for Tailwind
FROM node:23-slim AS css-builder
WORKDIR /app

# Install packages
COPY web/tailwind.config.js ./tailwind.config.js
COPY web/assets/package*.json ./
RUN npm install tailwindcss @tailwindcss/cli

# Build Tailwind CSS
COPY web/src/ ./src/
COPY web/assets/src/ ./assets/src/
RUN npx @tailwindcss/cli -i ./assets/src/main.css -o ./assets/dist/main.css

# Build stage for Dioxus
FROM rust:${RUST_VERSION}-slim-bookworm AS rust-builder

ARG APP_API_HOST

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
    rm -rf /var/lib/apt/lists/* && \
    rustup target add wasm32-unknown-unknown

RUN cargo install dioxus-cli --locked

# Build a dummy app with real dependencies
COPY web/Cargo.toml ./web/Cargo.toml
COPY web/Dioxus.toml ./web/Dioxus.toml
COPY web/build.rs ./web/build.rs
COPY shared/ ./shared/
COPY game ./game/
ENV APP_API_HOST=${APP_API_HOST}
ENV RUSTFLAGS="--cfg getrandom_backend='wasm_js'"
RUN mkdir -p web/src && \
    echo "fn main() {}" > web/src/main.rs && \
    cd web && \
    cargo build --release && \
    rm -f target/release/deps/web*

# Build real app with cached dependencies
COPY web/src/ ./web/src/
COPY --from=css-builder /app/assets/dist/ ./web/assets/dist/
COPY web/assets/images/ ./web/assets/images/
COPY web/assets/favicons/ ./web/assets/favicons/

WORKDIR /app/web
RUN dx build --release

# Final state with Nginx to serve static files
FROM nginx:alpine
COPY --from=rust-builder /app/web/target/dx/web/release/web/public/ /usr/share/nginx/html/
COPY web/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
