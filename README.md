# Hangrier Games

> A browser-based Hunger Games simulation built with Rust, featuring autonomous AI tributes, procedurally-generated arenas, and AI-powered sports commentary.

[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

##  Features

- **Autonomous Tribute AI**: 24 tributes with individual personalities, decision-making, and survival strategies
- **Turn-Based Combat**: D20-based combat system with weapons, shields, and status effects
- **Procedural Arena**: 5-region arena (Cornucopia + 4 cardinal directions) with dynamic item spawning and area closures
- **AI Commentary**: Dual-commentator narration powered by Ollama LLM, transforming game events into Capitol-style sports commentary
- **Full-Stack Rust**: Backend API (Axum + SurrealDB), HTMX server-rendered UI, pure Rust game engine
- **Real-Time State Management**: SSE-driven live updates with HTMX
- **Multiple Themes**: 3 themeable colorschemes with CSS custom properties

##  Architecture

**4-Crate Rust Workspace:**

```

┌─────────────────────────────────────────────┐
│  Browser - HTMX + Maud Templates            │
│  • Server-rendered HTML (maud)              │
│  • Themeable UI with Tailwind CSS          │
│  • Real-time updates via SSE               │
└──────────────────┬──────────────────────────┘
                   │ HTML fragments + JSON
                   ▼
┌─────────────────────────────────────────────┐
│  API Server - Axum REST API                 │
│  • JWT authentication                       │
│  • SurrealDB graph database                 │
│  • Server-side rendering with Maud         │
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
1. Browser requests page via HTMX (or direct navigation)
2. API renders HTML with Maud templates, hydrating state from SurrealDB
3. Game engine runs pure simulation logic
4. API persists updated state to database
5. Announcers generate commentary from event log (optional)
6. HTMX swaps in updated HTML via SSE push or hx-trigger

##  Prerequisites

- **Rust** (edition 2024) - [Install rustup](https://rustup.rs/)
- **Node.js** & npm - For Tailwind CSS compilation
- **SurrealDB** - [Installation guide](https://surrealdb.com/docs/installation)
- **Ollama** - [Installation guide](https://ollama.ai) (optional, for commentary)
- **Just** - [Installation guide](https://github.com/casey/just) (recommended, or use cargo commands directly)

##  Quick Start

### Automatic Setup (Recommended)

```bash
# Clone the repository
git clone https://github.com/kennethlove/hangrier_games.git
cd hangrier_games

# Install all dependencies (Node packages, Ollama model)
just setup

# Build Tailwind CSS
just build-css

# Start the full development environment (SurrealDB + API)
just dev
```

The application will be available at:
- **Web UI + API**: http://localhost:3000
- **SurrealDB**: ws://localhost:8000
- **Mailpit (email testing)**: http://localhost:8025

### Manual Setup

If you don't have `just` installed:

```bash
# Install dependencies
cd api/assets && npm install && cd ../..

# Create Ollama model (optional)
cd announcers/src
ollama create announcers -f Modelfile.qwen
cd ../..

# Build Tailwind CSS
cd api/assets
npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css
cd ../..

# Start services (in separate terminals)
# Terminal 1: SurrealDB
surrealdb start --log info --user root --pass root memory

# Terminal 2: API server (serves both UI and REST API)
cargo run --package api
```

##  Development Workflow

### Common Commands

This project uses [`just`](https://github.com/casey/just) for task automation. Run `just` to see all available recipes.

**Most useful commands:**

```bash
just dev          # Start SurrealDB, API, Mailpit, and Tailwind watcher
just api          # Run API server only (serves both UI and REST)
just build-css    # Build Tailwind CSS
just test         # Run game crate tests (60+ unit tests)
just fmt          # Format all code
just quality      # Run all quality checks (format, check, clippy, test)
just seed         # Seed dev database with test user and game
just open         # Open http://localhost:3000 in browser
just clean        # Clean all build artifacts
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
├── api/           # Axum REST API + HTMX server-rendered UI
│   ├── routes/    # HTTP endpoints (HTML + JSON)
│   ├── templates/ # Maud HTML templates
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
- **Rendering**: Server-side HTML with [Maud](https://maud.lambda.xyz/)
- **Dynamic UI**: [HTMX 2.0](https://htmx.org/) + SSE for real-time updates
- **Styling**: Tailwind CSS with CSS custom properties
- **Storage**: Cookies (JWT + session)

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
SURREAL_HOST=ws://localhost:8000     # API → SurrealDB
SURREAL_USER=root
SURREAL_PASS=root
```

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
- **api**: REST API + HTMX UI (port 3000)

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

- Run `just dev` to start all services at once
- Build Tailwind CSS with `just build-css` before starting the API
- Game crate tests can be slow; workspace-wide tests may hang
- Use `just seed` to populate the dev database with test data
- See `AGENTS.md` for AI agent instructions

##  Further Reading

- **[codemap.md](codemap.md)**: Comprehensive architecture documentation
- **[AGENTS.md](AGENTS.md)**: Project-specific AI agent instructions
- **[justfile](justfile)**: All available development commands
- **[HTMX Documentation](https://htmx.org/)**
- **[Maud Template Documentation](https://maud.lambda.xyz/)**
- **[SurrealDB Documentation](https://surrealdb.com/docs)**

##  License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

##  Acknowledgments

- Game icons from [game-icons.net](https://game-icons.net)
- Inspired by Suzanne Collins' *The Hunger Games*
- Built with the amazing Rust ecosystem

---

**Current Status**: Active development  
**Last Updated**: May 2026

For questions or issues, please [open an issue](https://github.com/kennethlove/hangrier_games/issues).
