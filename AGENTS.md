# Agent Instructions

## Project Structure

Rust workspace with 5 crates:
- `game/` - Core simulation logic (pure Rust, no I/O)
- `api/` - Axum REST API (SurrealDB backend)
- `web/` - Dioxus frontend (compiles to WASM)
- `shared/` - Shared types
- `announcers/` - Ollama LLM integration for game commentary

## Development Commands

**Build & run API**:
```bash
cargo run --package api
# Requires: SurrealDB running on SURREAL_HOST, .env file present
```

**Build & run frontend** (requires Dioxus CLI):
```bash
# Install dx first: cargo install dioxus-cli@0.6.2 --locked
cd web && dx serve
# Requires: APP_API_HOST in .env, Tailwind CSS built
```

**Build frontend CSS**:
```bash
cd web/assets
npm install
npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css
```

**Run tests** (game crate has ~60 inline tests using rstest):
```bash
cargo test --package game
# WARNING: Tests may be slow; workspace-wide `cargo test` can hang
```

**Format code** (custom edition=2024, fn_single_line=true):
```bash
cargo fmt
```

## Environment Setup

Required `.env` at repo root (already exists):
```bash
ENV=development
APP_API_HOST=http://127.0.0.1:3000    # Frontend → API
SURREAL_HOST=ws://localhost:8000       # API → SurrealDB
SURREAL_USER=root
SURREAL_PASS=root
```

**Frontend build.rs codegen**: Web crate reads `APP_*` env vars at build time and generates `src/env.rs`. Change requires rebuild.

## Critical Quirks

**WASM build**: Frontend requires `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'` and `wasm32-unknown-unknown` target.

**SurrealDB migrations**: `schemas/*.surql` files + `migrations/definitions/_initial.json` define schema. Migrations run via `surrealdb-migrations` crate at API startup.

**Announcer LLM**: Expects Ollama running locally with model named `announcers`. Create from `announcers/src/Modelfile.qwen`:
```bash
cd announcers/src
ollama create announcers -f Modelfile.qwen
```

**Docker build order matters**: Frontend Dockerfile builds Tailwind first, then Dioxus. API Dockerfile uses build cache for faster rebuilds.

## Non-Interactive Shell Commands

**ALWAYS use `-f` flag** with file operations to avoid hanging on prompts:
```bash
cp -f source dest           # NOT: cp source dest
mv -f source dest           # NOT: mv source dest
rm -f file                  # NOT: rm file
rm -rf directory            # NOT: rm -r directory
```

System aliases may add `-i` (interactive) flag, causing indefinite hangs waiting for y/n input.

## Version Control

This project uses **jj (Jujutsu)** for version control with git coexistence (`.jj/` directory + `.git/` for GitHub integration).

**Basic workflow**:
```bash
jj status                    # Show working copy changes
jj diff                      # Show uncommitted changes
jj commit -m "message"       # Create new commit
jj git push                  # Push to GitHub (uses git backend)
```

**Branch operations**:
```bash
jj new                       # Create new change on top of current
jj new main                  # Create change based on main
jj rebase -d main            # Rebase current change onto main
jj bookmark set feature-x    # Create/move bookmark (like git branch)
```

**Working with GitHub**:
```bash
jj git fetch                 # Fetch from origin
jj git push                  # Push bookmarks to origin
jj rebase -d main@origin     # Rebase onto remote main
```

**Key differences from git**:
- Every change has a unique ID (not just commits)
- `jj commit` creates immutable snapshot but keeps working copy
- Use `jj new` to start fresh change (like `git commit && git checkout -b`)
- Conflicts tracked explicitly; can defer resolution

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   jj git fetch                # Sync with remote
   jj rebase -d main@origin    # Rebase if needed
   bd dolt push                # Push beads data
   jj git push                 # Push commits to GitHub
   jj log -r 'remote_bookmarks()'  # Verify push succeeded
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
