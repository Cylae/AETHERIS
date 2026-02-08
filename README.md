# AETHERIS

### Next-Gen Hexagonal Server Orchestrator

AETHERIS is an industrial-grade, environment-agnostic server stack orchestrator built in Rust. It utilizes a **Hexagonal Architecture (Ports & Adapters)** to decouple core business logic from infrastructure details, enabling high-availability deployment and seamless CI/CD testing.

## 🚀 Key Features

- **Industrial Robustness:** Designed for high-availability environments.
- **Hexagonal Architecture:** Complete decoupling of domain logic from system calls.
- **Environment Agnostic:** Easily switch between `LiveAdapter` (Docker/Linux) and `MockRuntime` for testing.
- **Idempotent Installation:** Safe to run multiple times; ensures the desired state.
- **Resource Optimized:** Automatically tunes service resources based on hardware profiles (Low, Standard, High).
- **Security First:** Generates secure, random secrets and strictly manages system user quotas and permissions.

## 📦 Supported Services

### Media
- Plex
- Jellyfin
- Audiobookshelf
- Tautulli
- Overseerr
- Jellyseerr

### Arr Stack
- Sonarr
- Radarr
- Prowlarr
- Jackett
- Bazarr

### Downloads
- QBittorrent

### Infrastructure
- MariaDB
- Redis
- NginxProxy
- DNSCrypt
- Wireguard
- Portainer
- Netdata
- UptimeKuma

### Apps
- Vaultwarden
- Filebrowser
- Yourls
- GLPI
- Gitea
- Roundcube
- Nextcloud
- MailService
- Syncthing

## 🏗 Architecture

AETHERIS follows the Ports & Adapters pattern:

- **Domain (`src/domain/`):** Contains the `AetherisOrchestrator`, the heart of the system. It relies exclusively on Traits defined in the Ports layer.
- **Ports (`src/ports/`):** Defines `RuntimePort` and `SystemPort` traits. These specify *what* the system can do without prescribing *how*.
- **Adapters (`src/adapters/`):**
    - `LiveAdapter`: Real-world implementation using Docker CLI and Linux system calls.
    - `MockRuntime`: High-fidelity mock used for 100% test coverage in sandboxed or CI environments.

## 🛠 Usage

### Environment Variables

- `AETHERIS_HOME`: Base directory for configuration and data (defaults to current directory).
- `AETHERIS_CONFIG`: Explicit path to `config.yaml`.

### CLI Commands

```bash
# Install the full AETHERIS stack
aetheris install

# Check system and runtime status
aetheris status

# Manage users
aetheris user list
aetheris user add <username> --role Admin
```

## 🧪 Testing

AETHERIS is designed to be fully testable without requiring Docker or root privileges:

```bash
cargo test
```

The test suite automatically utilizes the `MockRuntime` to verify orchestrator logic and service configurations.

---
**Project AETHERIS** - *The Environment-Agnostic Orchestrator.*
