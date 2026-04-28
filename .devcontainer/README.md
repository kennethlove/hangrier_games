# Devcontainer

Reproducible dev environment for Hangrier Games. Opens in VS Code or any editor that speaks the [devcontainer spec](https://containers.dev/).

## What's inside

- **Rust 1.x** (Debian bookworm base) with `wasm32-unknown-unknown`, `clippy`, `rustfmt`, `rust-src`.
- **Dioxus CLI 0.6.2** preinstalled (matches `justfile`).
- **Node LTS** + npm for Tailwind CSS.
- **just**, **gh**, **SurrealDB CLI**.
- **SurrealDB sidecar** (in-memory) reachable at `ws://surrealdb:8000`.
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
just api    # in one terminal
just web    # in another
```

Or use `just dev` if you have a multiplexer set up.

## Caches

`cargo registry` and the workspace `target/` directory are mounted as named Docker volumes so rebuilds survive container recreation.
