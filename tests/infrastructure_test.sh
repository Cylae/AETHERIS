#!/bin/bash
# ==============================================================================
# AETHERIS INFRASTRUCTURE VALIDATION SUITE
# Automated end-to-end sanity check for scaffolded files, permissions,
# secret generation, SQL privilege compliance, and Compose syntax.
# ==============================================================================
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Running AETHERIS Infrastructure Test Suite ==="

# Clean test environment
rm -rf ./data ./media .env

# Run installation non-interactively
echo "n" | ./install.sh

# 1. Verify directory scaffolding and permissions
echo "Checking directory scaffolding and permissions..."
test -d ./data/mailserver/mail-data
test -d ./data/mailserver/mail-state
test -d ./data/mailserver/mail-logs
test -d ./data/mailserver/config
test -d ./data/roundcube/db
test -d ./data/roundcube/config
test -d ./media

MAIL_PERM=$(stat -c "%a" ./data/mailserver)
if [ "$MAIL_PERM" != "750" ]; then
    echo "ERROR: Mailserver permissions are $MAIL_PERM (expected 750)"
    false
fi

# 2. Verify .env file creation & random password generation
echo "Checking .env file generation..."
test -f .env

if grep -F -q "generate_secure_password_here" .env; then
    echo "ERROR: Default password placeholders still found in .env"
    false
fi

# 3. Verify init-dbs.sql generation and permissions
echo "Checking database initialization script..."
test -f ./data/init-dbs.sql

if grep -F -q '${' ./data/init-dbs.sql; then
    echo "ERROR: Unsubstituted template variables in init-dbs.sql"
    false
fi

if grep -F -q "ALL PRIVILEGES" ./data/init-dbs.sql; then
    echo "ERROR: Found ALL PRIVILEGES in init-dbs.sql (least privilege violation)"
    false
fi

# 4. Validate Docker Compose configuration with generated .env
echo "Validating Docker Compose configuration..."
docker compose config > /dev/null

echo "=== All Infrastructure Tests Passed Successfully! ==="
