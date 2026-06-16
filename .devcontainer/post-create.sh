#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Fixing volume ownership (named volumes mount as root by default)"
sudo chown -R vscode:vscode /usr/local/cargo/registry /workspaces/hangrier_games/target 2>/dev/null || true

echo "==> Cleaning stale build cache (host toolchain may differ)"
cargo clean 2>/dev/null || true

echo "==> Installing api/assets npm deps"
if [ -d api/assets ]; then
  (cd api/assets && npm install --no-audit --no-fund)
fi

echo "==> Building Tailwind CSS"
just build-css || echo "build-css skipped (will run on first dev start)"

echo "==> Ensuring .env exists"
if [ ! -f .env ]; then
  cat > .env <<'EOF'
ENV=development
SURREAL_HOST=ws://localhost:8000
SURREAL_USER=root
SURREAL_PASS=root
EOF
  echo "Created default .env"
fi

echo "==> Done. Run 'just dev' to start everything, then 'just seed' + 'just open'."
