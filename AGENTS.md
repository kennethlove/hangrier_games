# Agent Instructions

## Repository Map

A full codemap is available at `codemap.md` in the project root.

Before working on any task, read `codemap.md` to understand:
- Project architecture and entry points
- Directory responsibilities and design patterns
- Data flow and integration points between modules

For deep work on a specific folder, also read that folder's `codemap.md`.

## Project Structure

Rust workspace with 4 crates:
- `game/` - Core simulation logic (pure Rust, no I/O)
- `api/` - Axum REST API (SurrealDB backend)
- `shared/` - Shared types
- `announcers/` - Commentary pipeline: BroadcastPackageBuilder → Commentator trait → persisted segments

## Quick Commands

This project uses [just](https://github.com/casey/just) for common development tasks. Run `just` to see all available recipes.

**Most useful commands:**
- `just dev` - Start SurrealDB and API in one command
- `just api` - Run API server only
- `just test` - Run game crate tests
- `just fmt` - Format all code
- `just setup` - Install all dependencies (Ollama model)
- `just quality` - Run all quality checks (format, check, clippy, test)

See the `justfile` in the repository root for all available commands.

## Development Commands

**Build & run API**:
```bash
just api
# OR: cargo run --package api
# Requires: SurrealDB running on SURREAL_HOST, .env file present
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
SURREAL_HOST=ws://localhost:8000       # API → SurrealDB
SURREAL_USER=root
SURREAL_PASS=root
```

## Critical Quirks

**SurrealDB migrations**: `schemas/*.surql` files + `migrations/definitions/_initial.json` define schema. Migrations run via `surrealdb-migrations` crate at API startup.

**Announcer Commentary**: The `announcers/` crate transforms structured game events into Capitol broadcast commentary.
  - `BroadcastPackageBuilder` classifies 55+ `MessagePayload` variants into typed `EventLine`s
  - `Commentator` trait abstracts over LLM backends (Ollama behind `features = ["ollama"]`)
  - `TributeHistories` tracks rolling per-tribute digests
  - API integration: background task after each `run_game_cycles` → persists to `commentary_segments` table → pushes via SSE/WebSocket

**Ollama setup** (optional, requires `features = ["ollama"]` on the `announcers` crate):
```bash
cd announcers/src
ollama create announcers -f Modelfile
```

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

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:970c3bf2 -->
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

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   bd dolt push
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->

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
- A branch is NOT ready for a PR until all CI gates pass locally. Use `just quality` to run the full gamut.
- Work is NOT complete until a PR is open on GitHub
- NEVER push directly to `main`; always go through a feature bookmark + PR
- NEVER merge PRs locally; let the GitHub UI (or maintainer) do the merge
- NEVER stop before opening the PR - that leaves work stranded on a local bookmark
- NEVER say "ready to push when you are" - YOU must push the bookmark and open the PR
- If push or `gh pr create` fails, resolve and retry until it succeeds

<!-- BEGIN BEADS CODEX SETUP: generated by bd setup codex -->
## Beads Issue Tracker

Use Beads (`bd`) for durable task tracking in repositories that include it. Use the `beads` skill at `.agents/skills/beads/SKILL.md` (project install) or `~/.agents/skills/beads/SKILL.md` (global install) for Beads workflow guidance, then use the `bd` CLI for issue operations.

### Quick Reference

```bash
bd ready                # Find available work
bd show <id>            # View issue details
bd update <id> --claim  # Claim work
bd close <id>           # Complete work
bd prime                # Refresh Beads context
```

### Rules

- Use `bd` for all task tracking; do not create markdown TODO lists.
- Run `bd prime` when Beads context is missing or stale. Codex 0.129.0+ can load Beads context automatically through native hooks; use `/hooks` to inspect or toggle them.
- Keep persistent project memory in Beads via `bd remember`; do not create ad hoc memory files.

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.
<!-- END BEADS CODEX SETUP -->
