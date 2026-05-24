#!/usr/bin/env bash
# Dev seed: create test user + game for manual testing.
# Requires: API server running at localhost:3000.
# Usage: bash scripts/dev-seed.sh [base_url] [email] [password]

set -euo pipefail

BASE="${1:-http://localhost:3000}"
EMAIL="${2:-test@hangrier.dev}"
PASS="${3:-password123}"
COOKIE_JAR="/tmp/hangrier-cookies.txt"

echo "=== Seeding Dev Environment ==="
echo "API: $BASE"
echo "Email: $EMAIL"
echo ""

# 1. Get CSRF token by loading the auth page
echo "1. Getting CSRF token..."
curl -s -c "$COOKIE_JAR" -b "$COOKIE_JAR" "$BASE/auth" -o /dev/null
CSRF=$(grep -oP 'csrf_token=[^;]+' "$COOKIE_JAR" 2>/dev/null | head -1 | cut -d= -f2-)
if [ -z "$CSRF" ]; then
  echo "   WARNING: Could not extract CSRF token from cookie."
  echo "   Continuing with placeholder. If requests fail, try:"
  echo "   First visit http://localhost:3000/auth in your browser, then run this script."
  CSRF="dev"
fi
echo "   CSRF token: ${CSRF:0:20}..."

# 2. Register user (CSRF-secured)
echo "2. Registering user..."
REG_RESP=$(curl -s -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
  -X POST "$BASE/auth/register" \
  -d "display_name=TestUser&email=$EMAIL&password=$PASS&confirm_password=$PASS&csrf_token=$CSRF" \
  -o /tmp/hangrier-register.html \
  -w "%{http_code}")
echo "   HTTP: $REG_RESP (see /tmp/hangrier-register.html)"

# 3. Verify email via dev-only route (no CSRF needed)
echo "3. Verifying email (dev bypass)..."
VERIFY_RESP=$(curl -s -L \
  -X POST "$BASE/dev/verify-email" \
  -d "email=$EMAIL" \
  -o /tmp/hangrier-verify.html \
  -w "%{http_code}")
echo "   HTTP: $VERIFY_RESP"

# 4. Get fresh CSRF token for login
echo "4. Getting fresh CSRF token..."
curl -s -c "$COOKIE_JAR" -b "$COOKIE_JAR" "$BASE/auth" -o /dev/null
CSRF=$(grep -oP 'csrf_token=[^;]+' "$COOKIE_JAR" 2>/dev/null | head -1 | cut -d= -f2-)
[ -z "$CSRF" ] && CSRF="dev"

# 5. Login
echo "5. Logging in..."
LOGIN_RESP=$(curl -s -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
  -X POST "$BASE/auth/login" \
  -d "email=$EMAIL&password=$PASS&csrf_token=$CSRF" \
  -o /tmp/hangrier-login.html \
  -w "%{http_code}")
echo "   HTTP: $LOGIN_RESP"

# 6. Create a game (needs its own CSRF token)
echo "6. Getting CSRF for game creation..."
curl -s -c "$COOKIE_JAR" -b "$COOKIE_JAR" "$BASE/games/new" -o /dev/null
CSRF=$(grep -oP 'csrf_token=[^;]+' "$COOKIE_JAR" 2>/dev/null | head -1 | cut -d= -f2-)
[ -z "$CSRF" ] && CSRF="dev"

echo "7. Creating test game..."
GAME_RESP=$(curl -s -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
  -X POST "$BASE/games/new" \
  -d "name=Dev+Test+Game&description=Created+by+seed+script&csrf_token=$CSRF" \
  -D /tmp/hangrier-game-headers.txt \
  -o /tmp/hangrier-game.html \
  -w "%{http_code}")
echo "   HTTP: $GAME_RESP"

# Extract redirect location (game URL)
GAME_URL=$(grep -oP 'Location: [^\r\n]+' /tmp/hangrier-game-headers.txt 2>/dev/null | sed 's/Location: //' || echo "/games")
echo ""
echo "=== Dev seed complete ==="
echo "Login: $EMAIL / $PASS"
echo "Open: $BASE$GAME_URL"
echo ""
echo "If login fails, the CSRF token flow might need adjustment."
echo "Try: open $BASE/auth in browser first, then re-run this script."
