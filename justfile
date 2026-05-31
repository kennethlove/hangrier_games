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

# Build Tailwind CSS for HTMX pages
build-css:
    #!/usr/bin/env bash
    cd api/assets && npm install --silent && npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css

# Watch Tailwind CSS for changes and auto-rebuild
watch-css:
    cd api/assets && npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css --watch

# Start full development environment (DB + API + CSS)
dev:
    #!/usr/bin/env bash
    echo "==> Starting SurrealDB..."
    surreal start --log info --user root --pass root --bind 0.0.0.0:8000 surrealkv://.surrealdb &
    DB_PID=$!
    sleep 2

    echo "==> Starting Mailpit (email testing)..."
    mailpit --smtp 0.0.0.0:1025 --listen 0.0.0.0:8025 &
    MAILPIT_PID=$!

    echo "==> Building Tailwind CSS..."
    cd api/assets && npm install --silent && npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css
    echo "==> Starting Tailwind watcher..."
    cd api/assets && npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css --watch &
    CSS_PID=$!

    echo "==> Starting API server..."
    cargo run --package api &
    API_PID=$!
    sleep 3

    echo ""
    echo "Development environment running:"
    echo "  - Mailpit UI: http://localhost:8025"
    echo "  - SurrealDB: ws://localhost:8000"
    echo "  - API + HTMX pages: http://localhost:3000"
    echo "  - Tailwind: watching api/assets/src/main.css"
    echo ""
    echo "Press Ctrl+C to stop all services"
    trap "kill $DB_PID $MAILPIT_PID $API_PID $CSS_PID 2>/dev/null; exit" INT
    wait

# Start Mailpit email testing UI
mailpit:
    mailpit --smtp 0.0.0.0:1025 --listen 0.0.0.0:8025

# Seed dev database with test user and game
seed:
    #!/usr/bin/env bash
    if [ ! -f scripts/dev-seed.sh ]; then
        echo "Error: scripts/dev-seed.sh not found."
        exit 1
    fi
    bash scripts/dev-seed.sh "$@"

# Open the app in browser
open:
    open http://localhost:3000

# Install dependencies for development
install-deps:
    #!/usr/bin/env bash
    echo "==> Installing Node dependencies..."
    cd api/assets && npm install
    echo "==> Installing Mailpit..."
    if command -v brew &> /dev/null; then
        brew install mailpit 2>/dev/null || echo "   Mailpit already installed"
    fi
    echo "==> Installing cargo-watch..."
    cargo install cargo-watch 2>/dev/null || echo "   cargo-watch already installed"
    echo ""
    echo "✓ Dev dependencies installed."
    echo "   Run 'just build-css' then 'just dev' to start."

# Building recipes
# ================

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

# Pull Ollama model for commentary (optional; requires `features = ["ollama"]`)
setup-ollama:
    ollama pull llama3.2:3b
    cd announcers/src && ollama create announcers -f Modelfile
    @echo "✓ Model ready. Enable with: cargo build --features announcers/ollama"

# Install Node dependencies for Tailwind CSS
setup-node:
    cd api/assets && npm install

# Run all setup tasks
setup: setup-node setup-ollama
    @echo "✓ Setup complete!"
    @echo ""
    @echo "Quick start:"
    @echo "  1. just build-css"
    @echo "  2. just dev"
    @echo "  3. just seed"
    @echo "  4. just open"

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
    @echo "  announcers/ - Commentary pipeline (BroadcastPackageBuilder + Commentator trait)"

# Show helpful development tips
tips:
    @echo "Quick start:"
    @echo "  Terminal 1: just db"
    @echo "  Terminal 2: just build-css && just api"
    @echo "  Terminal 3: just seed"
    @echo "  Browser:    http://localhost:3000"
    @echo ""
    @echo "   OR"
    @echo ""
    @echo "  Single terminal: just dev"
    @echo "  (then in another: just seed && just open)"
    @echo ""
    @echo "For more information, see AGENTS.md"
