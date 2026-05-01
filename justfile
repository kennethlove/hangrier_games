# Hangrier Games - Rust Workspace Justfile
# List all available recipes
default:
    @just --list

# Development recipes
# ==================

# Run the API server (requires SurrealDB running)
api:
    cargo run --package api

# Run the frontend dev server (requires Dioxus CLI and Tailwind CSS built)
web:
    cd web && dx serve

# Start SurrealDB with persistent on-disk storage for local development
db:
    surreal start --log trace --user root --pass root --bind 0.0.0.0:8000 surrealkv://.surrealdb

# Start full development environment (DB, API, and web frontend)
dev:
    #!/usr/bin/env bash
    echo "Starting SurrealDB..."
    surreal start --log info --user root --pass root --bind 0.0.0.0:8000 surrealkv://.surrealdb &
    DB_PID=$!
    sleep 2
    echo "Starting API server..."
    cargo run --package api &
    API_PID=$!
    sleep 2
    echo "Starting web frontend..."
    cd web && dx serve &
    WEB_PID=$!
    echo ""
    echo "Development environment running:"
    echo "  - SurrealDB: ws://localhost:8000"
    echo "  - API: http://localhost:3000"
    echo "  - Web: http://localhost:8080"
    echo ""
    echo "Press Ctrl+C to stop all services"
    trap "kill $DB_PID $API_PID $WEB_PID 2>/dev/null" EXIT
    wait

# Building recipes
# ================

# Build Tailwind CSS for the frontend
build-css:
    #!/usr/bin/env bash
    cd web/assets
    npm install
    npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css

# Build the web frontend (debug mode - faster)
build-web-dev:
    cd web
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' dx build

# Build the web frontend (CSS + Dioxus)
build-web: build-css
    #!/usr/bin/env bash
    cd web
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' dx build --release

# Build the entire workspace
build-all:
    cargo build --workspace

# Build for production (optimized)
build-prod: build-css
    #!/usr/bin/env bash
    cargo build --workspace --release
    cd web
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' dx build --release

# Quality recipes
# ==============

# Fast check - just verify compilation without building
check-fast:
    cargo check --workspace --all-targets

# Check web crate only (faster for frontend-only changes)
check-web:
    cargo check --package web --target wasm32-unknown-unknown

# Check api crate only (faster for backend-only changes)
check-api:
    cargo check --package api

# Run tests for the game crate (workspace-wide tests may hang)
test:
    cargo test --package game

# Run all workspace tests (WARNING: may be slow or hang)
test-all:
    cargo test --workspace

# Format all code with custom rustfmt settings
fmt:
    cargo fmt --all

# Check all crates for compilation errors
check:
    cargo check --workspace

# Run clippy linter on all crates
clippy:
    cargo clippy --workspace -- -D warnings

# Run full quality gate (format, check, clippy, test)
quality: fmt check clippy test
    @echo "✓ All quality checks passed!"

# Setup recipes
# ============

# Install Dioxus CLI
setup-dx:
    cargo install dioxus-cli@0.7.7 --locked

# Create Ollama announcer model
setup-ollama:
    #!/usr/bin/env bash
    cd announcers/src
    ollama create announcers -f Modelfile.qwen

# Install wasm32 target for frontend builds
setup-wasm:
    rustup target add wasm32-unknown-unknown

# Install Node dependencies for Tailwind CSS
setup-node:
    cd web/assets && npm install

# Run all setup tasks
setup: setup-wasm setup-dx setup-node setup-ollama
    @echo "✓ Setup complete!"
    @echo ""
    @echo "Next steps:"
    @echo "  1. Ensure .env file exists with required variables"
    @echo "  2. Run 'just build-css' to build Tailwind CSS"
    @echo "  3. Run 'just dev' to start the development environment"

# Utility recipes
# ==============

# Clean all build artifacts
clean:
    cargo clean
    rm -rf web/dist
    rm -rf web/assets/dist
    rm -rf web/assets/node_modules

# Clean and rebuild everything
rebuild: clean build-all

# Watch and rebuild on file changes (requires cargo-watch)
watch:
    cargo watch -x "check --workspace"

# Docker recipes
# =============

# Build Docker images for production
docker-build:
    docker-compose build

# Start production environment with Docker Compose
docker-up:
    docker-compose up -d

# Stop Docker Compose services
docker-down:
    docker-compose down

# View Docker Compose logs
docker-logs:
    docker-compose logs -f

# Database recipes
# ===============

# Run SurrealDB migrations (migrations run automatically on API startup)
migrate:
    @echo "Migrations run automatically when the API starts."
    @echo "To manually run migrations, use the surrealdb-migrations CLI."

# Information recipes
# ==================

# Show environment configuration
env:
    @echo "Required environment variables:"
    @cat .env 2>/dev/null || echo ".env file not found!"

# Show project structure
tree:
    @echo "Workspace structure:"
    @echo "  game/       - Core simulation logic (pure Rust)"
    @echo "  api/        - Axum REST API (SurrealDB backend)"
    @echo "  web/        - Dioxus frontend (WASM)"
    @echo "  shared/     - Shared types"
    @echo "  announcers/ - Ollama LLM integration"

# Show helpful development tips
tips:
    @echo "Development tips:"
    @echo "  - Frontend requires RUSTFLAGS='--cfg getrandom_backend=\"wasm_js\"'"
    @echo "  - Build Tailwind CSS before building web frontend"
    @echo "  - .env file must exist with APP_API_HOST, SURREAL_HOST, etc."
    @echo "  - Game crate tests can be slow; workspace tests may hang"
    @echo "  - Use 'just dev' to start all services at once"
    @echo ""
    @echo "For more information, see AGENTS.md"
