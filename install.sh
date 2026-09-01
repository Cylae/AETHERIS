#!/bin/bash
set -euo pipefail

# ==============================================================================
# AETHERIS INSTALLATION & INITIALIZATION SCRIPT
# Pure Container-First Architecture (Tabula Rasa)
# ==============================================================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() { echo -e "${GREEN}[INFO]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
warn() { echo -e "${YELLOW}[WARNING]${NC} $1"; }

banner() {
    cat << 'EOF'
    ___    __________________  ____________  _________
   /   |  / ____/_  __/ __/ / / / ____/ __ \/  _/ ___/
  / /| | / __/   / / / /_/ /_/ / __/ / /_/ // / \__ \
 / ___ |/ /___  / / / __/ __  / /___/ _, _// / ___/ /
/_/  |_/_____/ /_/ /_/ /_/ /_/_____/_/ |_/___//____/

   Pure Container-First Infrastructure Orchestrator
EOF
}

# ------------------------------------------------------------------------------
# 1. Pre-flight Checks
# ------------------------------------------------------------------------------
banner

if ! command -v docker &> /dev/null; then
    error "Docker is not installed. Please install Docker first: https://get.docker.com/"
fi

if ! docker compose version &> /dev/null; then
    error "Docker Compose (V2) is not installed. Please install it."
fi

# ------------------------------------------------------------------------------
# 2. Directory Scaffolding
# ------------------------------------------------------------------------------
log "Scaffolding data directories..."

# Define base paths (matches .env.template)
DATA_DIR="./data"
MEDIA_DIR="./media"

mkdir -p "${DATA_DIR}/mailserver/mail-data"
mkdir -p "${DATA_DIR}/mailserver/mail-state"
mkdir -p "${DATA_DIR}/mailserver/mail-logs"
mkdir -p "${DATA_DIR}/mailserver/config"
mkdir -p "${DATA_DIR}/roundcube/db"
mkdir -p "${DATA_DIR}/roundcube/config"
mkdir -p "${MEDIA_DIR}"

# Secure permissions for Mailserver data directories
chmod -R 750 "${DATA_DIR}/mailserver" || true

log "Directories created successfully."

# ------------------------------------------------------------------------------
# 3. Environment Variable Generation
# ------------------------------------------------------------------------------
log "Configuring environment variables..."

if [ -f ".env" ]; then
    warn ".env file already exists. Skipping secret generation to preserve existing configuration."
else
    log "Generating secure .env file..."
    cp .env.template .env

    # Generate cryptographically secure random passwords
    MYSQL_ROOT_PASS=$(openssl rand -hex 24)
    NPM_DB_PASS=$(openssl rand -hex 24)
    NEXTCLOUD_DB_PASS=$(openssl rand -hex 24)
    GITEA_DB_PASS=$(openssl rand -hex 24)
    YOURLS_DB_PASS=$(openssl rand -hex 24)

    REDIS_PASS=$(openssl rand -hex 24)
    ADMIN_TOKEN=$(openssl rand -base64 48 | tr -d '\n')
    NEXTCLOUD_ADMIN_PASS=$(openssl rand -hex 16)
    YOURLS_ADMIN_PASS=$(openssl rand -hex 16)

    # Replace placeholders in .env
    sed -i "s/MYSQL_ROOT_PASSWORD=generate_secure_password_here/MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASS}/" .env
    sed -i "s/NPM_DB_PASSWORD=generate_secure_password_here/NPM_DB_PASSWORD=${NPM_DB_PASS}/" .env
    sed -i "s/NEXTCLOUD_DB_PASSWORD=generate_secure_password_here/NEXTCLOUD_DB_PASSWORD=${NEXTCLOUD_DB_PASS}/" .env
    sed -i "s/GITEA_DB_PASSWORD=generate_secure_password_here/GITEA_DB_PASSWORD=${GITEA_DB_PASS}/" .env
    sed -i "s/YOURLS_DB_PASSWORD=generate_secure_password_here/YOURLS_DB_PASSWORD=${YOURLS_DB_PASS}/" .env

    sed -i "s/REDIS_PASSWORD=generate_secure_password_here/REDIS_PASSWORD=${REDIS_PASS}/" .env
    sed -i "s|ADMIN_TOKEN=generate_secure_token_here|ADMIN_TOKEN=${ADMIN_TOKEN}|" .env
    sed -i "s/NEXTCLOUD_ADMIN_PASSWORD=generate_secure_password_here/NEXTCLOUD_ADMIN_PASSWORD=${NEXTCLOUD_ADMIN_PASS}/" .env
    sed -i "s/YOURLS_ADMIN_PASSWORD=generate_secure_password_here/YOURLS_ADMIN_PASSWORD=${YOURLS_ADMIN_PASS}/" .env

    # Create the init-dbs.sql file from template
    log "Generating database initialization script..."
    sed -e "s/\${NPM_DB_PASSWORD}/${NPM_DB_PASS}/g" \
        -e "s/\${NEXTCLOUD_DB_PASSWORD}/${NEXTCLOUD_DB_PASS}/g" \
        -e "s/\${GITEA_DB_PASSWORD}/${GITEA_DB_PASS}/g" \
        -e "s/\${YOURLS_DB_PASSWORD}/${YOURLS_DB_PASS}/g" \
        init-dbs.sql.template > ./data/init-dbs.sql

    # Set Host UID/GID dynamically
    PUID=$(id -u)
    PGID=$(id -g)
    sed -i "s/PUID=1000/PUID=${PUID}/" .env
    sed -i "s/PGID=1000/PGID=${PGID}/" .env

    log "Secure .env file generated."
    warn "Please review the .env file to configure your DOMAIN_NAME and TZ (Timezone) before proceeding."
fi

# ------------------------------------------------------------------------------
# 4. Container Deployment
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}Ready to deploy AETHERIS stack.${NC}"
read -p "Do you want to start the containers now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log "Pulling latest Docker images..."
    docker compose pull

    log "Starting AETHERIS infrastructure..."
    docker compose up -d --remove-orphans

    log "Deployment complete! Check container status with: docker compose ps"
    echo -e "\n${GREEN}Access your services:${NC}"
    echo -e " - Nginx Proxy Manager: http://localhost:81 (Default login: admin@example.com / changeme)"
    echo -e " - Map your domains in Nginx Proxy Manager to access Roundcube, Portainer, etc."
else
    log "Deployment skipped. Run 'docker compose up -d' when you are ready."
fi
