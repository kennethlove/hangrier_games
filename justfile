# Hangrier Games - Rust Workspace Justfile
default:
    @just --list

# Development recipes
# ==================

# Run the API server (serves HTMX pages + REST API)
api:
    cargo run --package api

# Start SurrealDB with persistent on-disk storage for local development
db:
    surreal start --log trace --user root --pass root --bind 0.0.0.0:8000 surrealkv://.surrealdb

# Start full development environment (DB, API, and Tailwind watcher)
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
    echo "Building Tailwind CSS..."
    (cd api/assets && npm install --silent && npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css)
    echo "Starting Tailwind watcher..."
    (cd api/assets && npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css --watch) &
    CSS_PID=$!
    echo ""
    echo "Development environment running:"
    echo "  - SurrealDB: ws://localhost:8000"
    echo "  - API + HTMX pages: http://localhost:3000"
    echo "  - Tailwind: watching api/assets/src/main.css"
    echo ""
    echo "Press Ctrl+C to stop all services"
    trap "kill $DB_PID $API_PID $CSS_PID 2>/dev/null" EXIT
    wait

# Building recipes
# ================

# Build Tailwind CSS for HTMX pages
build-css:
    #!/usr/bin/env bash
    cd api/assets
    npm install
    npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css

# Build the entire workspace
build-all:
    cargo build --workspace

# Build for production (optimized)
build-prod: build-css
    cargo build --workspace --release

# Quality recipes
# ==============

# Fast check - just verify compilation without building
check-fast:
    cargo check --workspace --all-targets

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
    cargo clippy --workspace --tests -- -D warnings

# Run full quality gate (format, check, clippy, test)
quality: fmt check clippy test
    @echo "✓ All quality checks passed!"

# Setup recipes
# ============

# Create Ollama announcer model
setup-ollama:
    #!/usr/bin/env bash
    cd announcers/src
    ollama create announcers -f Modelfile.qwen

# Install Node dependencies for Tailwind CSS
setup-node:
    cd api/assets && npm install

# Run all setup tasks
setup: setup-node setup-ollama
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
    rm -rf api/assets/dist
    rm -rf api/assets/node_modules

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
    @echo "  api/        - Axum REST API + HTMX pages (SurrealDB backend)"
    @echo "  shared/     - Shared types"
    @echo "  announcers/ - Ollama LLM integration"

# Show helpful development tips
tips:
    @echo "Development tips:"
    @echo "  - API serves HTMX pages directly at http://localhost:3000"
    @echo "  - Build Tailwind CSS before running the API: just build-css"
    @echo "  - .env file must exist with SURREAL_HOST, SURREAL_USER, SURREAL_PASS"
    @echo "  - Game crate tests can be slow; workspace tests may hang"
    @echo "  - Use 'just dev' to start all services at once"
    @echo ""
    @echo "For more information, see AGENTS.md"
