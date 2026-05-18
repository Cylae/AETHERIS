#!/bin/bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo "Starting Infrastructure Tests..."

# Test 1: Verify install.sh syntax
if bash -n install.sh; then
    echo -e "${GREEN}[PASS]${NC} install.sh syntax is valid."
else
    echo -e "${RED}[FAIL]${NC} install.sh syntax is invalid."
    exit 1
fi

# Test 2: Validate docker-compose.yml configuration
if docker compose config -q; then
    echo -e "${GREEN}[PASS]${NC} docker-compose.yml configuration is valid."
else
    echo -e "${RED}[FAIL]${NC} docker-compose.yml configuration is invalid."
    exit 1
fi

# Test 3: Check for required environment variables in .env.template
REQUIRED_VARS=(
    "MYSQL_ROOT_PASSWORD"
    "REDIS_PASSWORD"
    "NPM_DB_PASSWORD"
    "ADMIN_TOKEN"
    "NEXTCLOUD_DB_PASSWORD"
    "GITEA_DB_PASSWORD"
    "YOURLS_DB_PASSWORD"
    "GRAFANA_ADMIN_PASSWORD"
)

for VAR in "${REQUIRED_VARS[@]}"; do
    if grep -q "^$VAR=" .env.template; then
        echo -e "${GREEN}[PASS]${NC} Required variable $VAR found in .env.template."
    else
        echo -e "${RED}[FAIL]${NC} Required variable $VAR missing from .env.template."
        exit 1
    fi
done

# Test 4: Check if init-dbs.sql.template is properly formatted for substitution
if grep -q "\${NPM_DB_PASSWORD}" init-dbs.sql.template; then
     echo -e "${GREEN}[PASS]${NC} init-dbs.sql.template contains substitution variables."
else
     echo -e "${RED}[FAIL]${NC} init-dbs.sql.template is missing substitution variables."
     exit 1
fi

echo -e "\n${GREEN}All infrastructure tests passed successfully!${NC}"
