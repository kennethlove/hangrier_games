# Hangrier Games

> A browser-based Hunger Games simulation built with Rust, featuring autonomous AI tributes, procedurally-generated arenas, and AI-powered sports commentary.

[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

##  Features

- **Autonomous Tribute AI**: 24 tributes with individual personalities, decision-making, and survival strategies
- **Turn-Based Combat**: D20-based combat system with weapons, shields, and status effects
- **Procedural Arena**: 5-region arena (Cornucopia + 4 cardinal directions) with dynamic item spawning and area closures
- **AI Commentary**: Dual-commentator narration powered by Ollama LLM, transforming game events into Capitol-style sports commentary
- **Full-Stack Rust**: Backend API (Axum + SurrealDB), frontend (Dioxus WASM), pure Rust game engine
- **Real-Time State Management**: Query-driven frontend with automatic cache invalidation and optimistic updates
- **Multiple Themes**: 3 themeable colorschemes with LocalStorage persistence

##  Architecture

**5-Crate Rust Workspace:**

```
┌─────────────────────────────────────────────┐
│  Browser (WASM) - Dioxus Frontend           │
│  • Query-driven state (dioxus-query)        │
│  • Themeable UI with Tailwind CSS          │
└──────────────────┬──────────────────────────┘
                   │ HTTP/JSON
                   ▼
┌─────────────────────────────────────────────┐
│  API Server - Axum REST API                 │
│  • JWT authentication                       │
│  • SurrealDB graph database                 │
└──────────────────┬──────────────────────────┘
                   │ Pure functions
                   ▼
┌─────────────────────────────────────────────┐
│  Game Engine - Pure Rust Logic              │
│  • Stateless, deterministic                 │
│  • Event sourcing via message queue        │
└──────────────────┬──────────────────────────┘
                   │ Event log
                   ▼
┌─────────────────────────────────────────────┐
│  Announcers - Ollama LLM Integration        │
│  • Streaming commentary generation          │
│  • Dual-commentator narrative               │
└─────────────────────────────────────────────┘
```

**Data Flow:**
1. Frontend sends HTTP request with JWT authentication
2. API validates request and hydrates game state from SurrealDB
3. Game engine runs pure simulation logic
4. API persists updated state to database
5. Announcers generate commentary from event log (optional)
6. Frontend refetches via query invalidation

##  Prerequisites

- **Rust** (edition 2024) - [Install rustup](https://rustup.rs/)
- **Node.js** & npm - For Tailwind CSS compilation
- **SurrealDB** - [Installation guide](https://surrealdb.com/docs/installation)
- **Dioxus CLI** - Installed via `cargo install dioxus-cli@0.6.2 --locked`
- **Ollama** - [Installation guide](https://ollama.ai) (optional, for commentary)
- **Just** - [Installation guide](https://github.com/casey/just) (recommended, or use cargo commands directly)

##  Quick Start

### Automatic Setup (Recommended)

```bash
# Clone the repository
git clone https://github.com/kennethlove/hangrier_games.git
cd hangrier_games

# Install all dependencies (Dioxus CLI, Node packages, Ollama model)
just setup

# Build Tailwind CSS
just build-css

# Start the full development environment (SurrealDB + API + web frontend)
just dev
```

The application will be available at:
- **Frontend**: http://localhost:8080
- **API**: http://localhost:3000
- **SurrealDB**: ws://localhost:8000

### Manual Setup

If you don't have `just` installed:

```bash
# Install dependencies
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli@0.6.2 --locked
cd web/assets && npm install && cd ../..

# Create Ollama model (optional)
cd announcers/src
ollama create announcers -f Modelfile.qwen
cd ../..

# Build Tailwind CSS
cd web/assets
npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css
cd ../..

# Start services (in separate terminals)
# Terminal 1: SurrealDB
surrealdb start --log info --user root --pass root memory

# Terminal 2: API server
cargo run --package api

# Terminal 3: Web frontend
cd web && dx serve
```

##  Development Workflow

### Common Commands

This project uses [`just`](https://github.com/casey/just) for task automation. Run `just` to see all available recipes.

**Most useful commands:**

```bash
just dev          # Start SurrealDB, API, and web frontend
just api          # Run API server only
just web          # Run frontend dev server only
just build-css    # Build Tailwind CSS
just test         # Run game crate tests (60+ unit tests)
just fmt          # Format all code
just quality      # Run all quality checks (format, check, clippy, test)
just clean        # Clean all build artifacts
```

### Frontend-Only Development

```bash
# Terminal 1: Start backend services
just db &
just api &

# Terminal 2: Start frontend with hot reload
just web

# Rebuild CSS after changing Tailwind classes
just build-css
```

### Running Tests

```bash
# Run game crate tests only (recommended)
just test

# Run all workspace tests (WARNING: may be slow)
just test-all
```

##  Project Structure

```
hangrier_games/
├── game/          # Pure Rust simulation engine (no I/O)
│   ├── areas/     # Arena topology and item management
│   ├── items/     # Weapons, shields, consumables
│   ├── tributes/  # Autonomous AI with decision-making
│   └── threats/   # Environmental hazards
├── api/           # Axum REST API + SurrealDB integration
│   ├── db/        # Database client and queries
│   ├── routes/    # HTTP endpoints
│   └── middleware/ # JWT authentication
├── web/           # Dioxus WASM frontend
│   ├── components/ # UI components (42+ files)
│   ├── queries/   # API integration layer
│   └── assets/    # Tailwind CSS and static files
├── shared/        # Shared data types (DTOs, enums)
├── announcers/    # Ollama LLM integration
├── schemas/       # SurrealDB schema definitions
└── migrations/    # Database migration tracking
```

See [`codemap.md`](codemap.md) for detailed architecture documentation.

##  Technology Stack

### Backend
- **Web Framework**: Axum 0.8.4
- **Database**: SurrealDB 2.3.2 (graph database)
- **Authentication**: JWT with Argon2 password hashing
- **Runtime**: Tokio 1.45.0 (async)

### Frontend
- **Framework**: Dioxus 0.6.3 (React-like WASM)
- **State Management**: dioxus-query 0.7.0 (async query caching)
- **Styling**: Tailwind CSS
- **Storage**: LocalStorage (JWT + theme persistence)

### Game Logic
- **Language**: Pure Rust (edition 2024)
- **Testing**: rstest (60+ parameterized unit tests)
- **RNG**: rand crate (procedural generation)

### AI/LLM
- **LLM**: Ollama with custom `announcers` model (qwen2.5:1.5b)
- **Client**: ollama-rs
- **Streaming**: async_stream for progressive commentary

##  Configuration

Create a `.env` file in the project root:

```bash
ENV=development
APP_API_HOST=http://127.0.0.1:3000  # Frontend → API
SURREAL_HOST=ws://localhost:8000     # API → SurrealDB
SURREAL_USER=root
SURREAL_PASS=root
```

**Note**: Frontend reads `APP_*` environment variables at **build time** via `build.rs`. Changing these requires rebuilding the frontend.

##  Docker Deployment

```bash
# Build Docker images
just docker-build

# Start all services
just docker-up

# View logs
just docker-logs

# Stop services
just docker-down
```

Services:
- **surrealdb**: Database (port 8000)
- **api**: REST API (port 3000)
- **web**: Frontend static files (port 8080)

##  Testing

The game crate has 60+ unit tests using `rstest` for parameterized testing:

```bash
# Run game crate tests only (recommended)
just test

# Run all workspace tests (WARNING: may hang)
just test-all
```

**Coverage includes**:
- Game lifecycle and state transitions
- Combat mechanics and d20 rolls
- Tribute AI decision-making
- Item generation and effects
- Area topology and closures

##  Contributing

Contributions are welcome! Please follow these guidelines:

1. **Code Style**: Run `just fmt` before committing (custom edition=2024, `fn_single_line=true`)
2. **Quality Checks**: Run `just quality` to verify format, compilation, clippy, and tests
3. **Commit Messages**: Use conventional commits (e.g., `feat:`, `fix:`, `docs:`)
4. **Testing**: Add tests for new features in the game crate
5. **Documentation**: Update relevant codemaps when changing architecture

### Development Tips

- Frontend requires `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'` for WASM builds
- Build Tailwind CSS before building the web frontend
- Game crate tests can be slow; workspace-wide tests may hang
- Use `just dev` to start all services at once
- See `AGENTS.md` for AI agent instructions

##  Further Reading

- **[codemap.md](codemap.md)**: Comprehensive architecture documentation
- **[AGENTS.md](AGENTS.md)**: Project-specific AI agent instructions
- **[justfile](justfile)**: All available development commands
- **[Dioxus Documentation](https://dioxuslabs.com/)**
- **[SurrealDB Documentation](https://surrealdb.com/docs)**

##  License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

##  Acknowledgments

- Game icons from [game-icons.net](https://game-icons.net)
- Inspired by Suzanne Collins' *The Hunger Games*
- Built with the amazing Rust ecosystem

---

**Current Status**: Active development  
**Last Updated**: April 2026

For questions or issues, please [open an issue](https://github.com/kennethlove/hangrier_games/issues).
