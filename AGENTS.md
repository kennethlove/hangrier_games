# Agent Instructions

## Repository Map

A full codemap is available at `codemap.md` in the project root.

Before working on any task, read `codemap.md` to understand:
- Project architecture and entry points
- Directory responsibilities and design patterns
- Data flow and integration points between modules

For deep work on a specific folder, also read that folder's `codemap.md`.

## Project Structure

Rust workspace with 5 crates:
- `game/` - Core simulation logic (pure Rust, no I/O)
- `api/` - Axum REST API (SurrealDB backend)
- `web/` - Dioxus frontend (compiles to WASM)
- `shared/` - Shared types
- `announcers/` - Ollama LLM integration for game commentary

## Quick Commands

This project uses [just](https://github.com/casey/just) for common development tasks. Run `just` to see all available recipes.

**Most useful commands:**
- `just dev` - Start SurrealDB, API, and web frontend in one command
- `just api` - Run API server only
- `just web` - Run frontend dev server only
- `just build-css` - Build Tailwind CSS
- `just test` - Run game crate tests
- `just fmt` - Format all code
- `just setup` - Install all dependencies (Dioxus CLI, Node packages, Ollama model)
- `just quality` - Run all quality checks (format, check, clippy, test)

See the `justfile` in the repository root for all available commands.

## Development Commands

**Build & run API**:
```bash
just api
# OR: cargo run --package api
# Requires: SurrealDB running on SURREAL_HOST, .env file present
```

**Build & run frontend** (requires Dioxus CLI):
```bash
just web
# OR: cd web && dx serve
# Install dx first: just setup-dx
# Requires: APP_API_HOST in .env, Tailwind CSS built
```

**Build frontend CSS**:
```bash
just build-css
# OR: cd web/assets && npm install && npx @tailwindcss/cli -i ./src/main.css -o ./dist/main.css
```

**Run tests** (game crate has ~60 inline tests using rstest):
```bash
just test
# OR: cargo test --package game
# WARNING: Tests may be slow; workspace-wide `cargo test` can hang
```

**Format code** (custom edition=2024, fn_single_line=true):
```bash
just fmt
# OR: cargo fmt
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

## Pull Request Workflow

**All changes land on `main` via GitHub Pull Requests.** Never merge to `main` locally and never push directly to `main`.

**Standard flow**:
```bash
# 1. Make commits on a descriptive bookmark (not main)
jj bookmark create my-feature -r @-

# 2. Push the bookmark to origin
jj git push --bookmark my-feature

# 3. Open a PR with gh
gh pr create --base main --head my-feature \
  --title "type(scope): summary" \
  --body "..."
```

**Rules**:
- One bookmark per logical change; name it after the work (e.g. `fix-ws-hook`, `feat-sponsorship`)
- Use conventional commit style for both commit messages and PR titles
- PR body should include a Summary, Changes, Verification (commands run), and Follow-ups (beads IDs) section
- Do not use `jj git push` to push directly to `main` — only push feature bookmarks
- Do not run `jj bookmark set main` to advance main locally; let the GitHub merge do it, then `jj git fetch` to sync

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

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until a PR exists on GitHub for every code change.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create beads issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds (`just quality`)
3. **Update issue status** - Close finished work, update in-progress items
4. **OPEN A PULL REQUEST** - This is MANDATORY for any code change:
   ```bash
   jj git fetch                                    # Sync with remote
   jj rebase -d main@origin                        # Rebase if needed
   bd backup export-git --branch beads-backup      # Push beads JSONL snapshot to beads-backup branch
   jj bookmark create <branch-name> -r @-          # Create feature bookmark
   jj git push --bookmark <branch-name>            # Push the bookmark (NOT main)
   gh pr create --base main --head <branch-name> \
     --title "type(scope): summary" --body "..."   # Open the PR
   ```
5. **Clean up** - Clear stashes, prune stale local bookmarks
6. **Verify** - All changes committed AND a PR URL is in hand
7. **Hand off** - Provide the PR URL plus context for next session

**CRITICAL RULES:**
- Work is NOT complete until a PR is open on GitHub
- NEVER push directly to `main`; always go through a feature bookmark + PR
- NEVER merge PRs locally; let the GitHub UI (or maintainer) do the merge
- NEVER stop before opening the PR - that leaves work stranded on a local bookmark
- NEVER say "ready to push when you are" - YOU must push the bookmark and open the PR
- If push or `gh pr create` fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
