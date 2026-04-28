#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Installing web/assets npm deps"
if [ -d web/assets ]; then
  (cd web/assets && npm install --no-audit --no-fund)
fi

echo "==> Building Tailwind CSS"
just build-css || echo "build-css skipped (will run on first dev start)"

echo "==> Ensuring .env exists"
if [ ! -f .env ]; then
  cat > .env <<'EOF'
ENV=development
APP_API_HOST=http://127.0.0.1:3000
SURREAL_HOST=ws://surrealdb:8000
SURREAL_USER=root
SURREAL_PASS=root
EOF
  echo "Created default .env (SurrealDB reachable at ws://surrealdb:8000 inside the devcontainer network)"
fi

echo "==> Done. Run 'just api' and 'just web' (or 'just dev' if you have tmux)."
