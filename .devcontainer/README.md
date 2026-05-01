# Devcontainer

Reproducible dev environment for Hangrier Games. Opens in VS Code or any editor that speaks the [devcontainer spec](https://containers.dev/).

## What's inside

- **Rust 1.x** (Debian bookworm base) with `wasm32-unknown-unknown`, `clippy`, `rustfmt`, `rust-src`.
- **Dioxus CLI 0.6.2** preinstalled (matches `justfile`).
- **Node LTS** + npm for Tailwind CSS.
- **just**, **gh**.
- **SurrealDB CLI** (`surreal` binary, v2.1.4) in `/usr/local/bin`. Started in-container by `just dev` / `just db` with persistent on-disk storage at `.surrealdb/` in the workspace root (gitignored). Listens on `ws://localhost:8000`.
- System libs for Dioxus desktop builds (`libwebkit2gtk-4.1`, `libgtk-3`, `libsoup-3.0`, `libxdo`, `clang`).

Ollama is **not** included; install on the host and point at it if you want announcer commentary.

## Forwarded ports

| Port | Service        |
|------|----------------|
| 3000 | API (axum)     |
| 8000 | SurrealDB      |
| 8080 | Dioxus web dev |

## First-run

`postCreateCommand` runs `.devcontainer/post-create.sh` which installs `web/assets` npm deps, builds Tailwind, and writes a default `.env` if one is missing.

After it finishes:

```bash
just dev    # starts SurrealDB + API + web together
```

Or run them individually in separate terminals:

```bash
just db     # SurrealDB (memory mode, ws://localhost:8000)
just api    # API server
just web    # Dioxus dev server
```

## Caches

`cargo registry` and the workspace `target/` directory are mounted as named Docker volumes so rebuilds survive container recreation.
