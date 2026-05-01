#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Fixing volume ownership (named volumes mount as root by default)"
sudo chown -R vscode:vscode /usr/local/cargo/registry /workspaces/hangrier_games/target 2>/dev/null || true

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
SURREAL_HOST=ws://localhost:8000
SURREAL_USER=root
SURREAL_PASS=root
EOF
  echo "Created default .env (SurrealDB runs in-container at ws://localhost:8000)"
fi

echo "==> Done. Run 'just dev' (starts SurrealDB + API + web in one shell)."
