#!/bin/bash
set -euo pipefail

# Tests for the database initialization script generation logic in install.sh

echo "Running init-dbs.sql generation tests..."

# Resolve project root directory dynamically
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Setup temporary directory
TEMP_DIR=$(mktemp -d)
trap 'rm -rf -- "$TEMP_DIR"' EXIT

# Copy the template
cp "$PROJECT_ROOT/init-dbs.sql.template" "$TEMP_DIR/"

# Setup mock environment variables
NPM_DB_PASS="mock_npm_pass_123!"
NEXTCLOUD_DB_PASS="mock_nc_pass_456@"
GITEA_DB_PASS="mock_gitea_pass_789#"
YOURLS_DB_PASS="mock_yourls_pass_012$"

# Run the sed command as it appears in install.sh, adjusting paths
mkdir -p "$TEMP_DIR/data"
sed -e "s/\${NPM_DB_PASSWORD}/${NPM_DB_PASS}/g" \
    -e "s/\${NEXTCLOUD_DB_PASSWORD}/${NEXTCLOUD_DB_PASS}/g" \
    -e "s/\${GITEA_DB_PASSWORD}/${GITEA_DB_PASS}/g" \
    -e "s/\${YOURLS_DB_PASSWORD}/${YOURLS_DB_PASS}/g" \
    "$TEMP_DIR/init-dbs.sql.template" > "$TEMP_DIR/data/init-dbs.sql"

# Verification
TARGET_FILE="$TEMP_DIR/data/init-dbs.sql"

if [ ! -f "$TARGET_FILE" ]; then
    echo "ERROR: Generated file does not exist at $TARGET_FILE"
    exit 1
fi

echo "Verifying NPM password..."
if ! grep -Fq "${NPM_DB_PASS}" "$TARGET_FILE"; then
    echo "ERROR: NPM password was not correctly interpolated."
    exit 1
fi

echo "Verifying Nextcloud password..."
if ! grep -Fq "${NEXTCLOUD_DB_PASS}" "$TARGET_FILE"; then
    echo "ERROR: Nextcloud password was not correctly interpolated."
    exit 1
fi

echo "Verifying Gitea password..."
if ! grep -Fq "${GITEA_DB_PASS}" "$TARGET_FILE"; then
    echo "ERROR: Gitea password was not correctly interpolated."
    exit 1
fi

echo "Verifying YOURLS password..."
if ! grep -Fq "${YOURLS_DB_PASS}" "$TARGET_FILE"; then
    echo "ERROR: YOURLS password was not correctly interpolated."
    exit 1
fi

echo "Verifying no template variables remain..."
if grep -q "\${.*_DB_PASSWORD}" "$TARGET_FILE"; then
    echo "ERROR: Some template variables were not replaced."
    grep "\${.*_DB_PASSWORD}" "$TARGET_FILE"
    exit 1
fi

echo "All tests passed successfully!"
exit 0
