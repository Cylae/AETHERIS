#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Running Infrastructure Tests ==="

# 1. Shell script syntax check
echo "[1/4] Validating shell script syntax..."
bash -n install.sh
echo "  ✓ install.sh syntax valid."

# 2. Test install.sh non-interactive setup
echo "[2/4] Testing non-interactive installation setup..."
TEST_DIR=$(mktemp -d /tmp/aetheris_test_XXXXXX)
cleanup() {
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

cp install.sh .env.template init-dbs.sql.template "$TEST_DIR/"
cd "$TEST_DIR"

echo 'n' | ./install.sh > /dev/null

if [ ! -f .env ]; then
    echo "  ✗ Error: .env file was not generated."
    return 1 2>/dev/null || exit 1
fi

if [ ! -f ./data/init-dbs.sql ]; then
    echo "  ✗ Error: init-dbs.sql was not generated in ./data/."
    return 1 2>/dev/null || exit 1
fi
echo "  ✓ install.sh successfully scaffolded .env and init-dbs.sql."

# 3. Test Docker Compose configuration validity
echo "[3/4] Validating Docker Compose schema and env resolution..."
cp "$PROJECT_ROOT/docker-compose.yml" "$TEST_DIR/"
docker compose config > /dev/null
echo "  ✓ docker-compose.yml is structurally valid with generated .env."

# 4. Verify init-dbs.sql password substitution
echo "[4/4] Verifying database credentials substitution..."
if grep -q '\${NPM_DB_PASSWORD}' ./data/init-dbs.sql; then
    echo "  ✗ Error: init-dbs.sql contains unreplaced placeholders."
    return 1 2>/dev/null || exit 1
fi
echo "  ✓ Password substitutions succeeded."

echo "=== All Infrastructure Tests Passed Successfully ==="
