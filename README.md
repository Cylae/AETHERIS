# Server Manager - Next-Gen Media Server Orchestrator 🚀

![Server Manager Banner](https://img.shields.io/badge/Status-Production--Ready-brightgreen) ![Version](https://img.shields.io/badge/Version-1.0.5-blue) ![Rust](https://img.shields.io/badge/Built%20With-Rust-orange) ![Docker](https://img.shields.io/badge/Powered%20By-Docker-blue) ![Security](https://img.shields.io/badge/Security-Hardened-red)

**Server Manager** is an intelligent, production-grade orchestrator written in Rust that deploys, manages, and optimizes a complete personal media and cloud server stack. It automatically detects your hardware and configures 28 Docker services for optimal performance.

---

## 🌟 Key Features

### Core Capabilities
- **28 Integrated Services**: Plex, Jellyfin, Sonarr, Radarr, Nextcloud, Mailserver, and more
- **Smart Hardware Detection**: Automatically adapts configuration based on RAM, CPU, GPU, and available swap
- **Three-Tier Optimization**: Low/Standard/High profiles ensure optimal performance on any hardware
- **Secure by Default**: UFW firewall, password generation, isolated Docker networks, localhost bindings
- **GPU Acceleration**: Automatic detection and configuration for Nvidia NVENC & Intel QuickSync
- **Web Administration**: Secure dashboard for service management (default port 8099)
- **User Management**: System-integrated user accounts with storage quotas
- **Idempotent Operations**: Safe to run installation multiple times

### Hardware Profiles

| Profile | Criteria | RAM Transcoding | ArrStack GC | Mail Antivirus | Database Buffer |
|---------|----------|-----------------|-------------|----------------|-----------------|
| **Low** | <4GB RAM or ≤2 cores or <8GB RAM with no swap | Disk | Disabled | Disabled | 256MB |
| **Standard** | 4-16GB RAM with adequate cores | Disk | Enabled | Enabled | 1GB |
| **High** | >16GB RAM | RAM (/dev/shm) | Enabled | Enabled | 4GB |

---

## 🚀 Quick Start

### One-Click Installation (Recommended)

The easiest way to deploy Server Manager on a fresh Linux system:

```bash
curl -sL https://raw.githubusercontent.com/Cylae/server_script/server-setup-script/setup.sh | sudo bash
```

**What this does:**
1. Hardens system security (UFW, Fail2Ban)
2. Configures filesystem quotas
3. Installs Docker and Rust
4. Compiles and deploys Server Manager
5. Launches all 28 services

**Time to complete**: 10-20 minutes (depending on hardware)

---

## 📋 Prerequisites

### System Requirements

**Minimum (Low Profile)**
- 2 GB RAM + 2 GB Swap
- 2 CPU cores
- 50 GB disk space
- Debian 11/12 or Ubuntu 22.04+

**Recommended (Standard Profile)**
- 8 GB RAM + 4 GB Swap
- 4 CPU cores
- 200 GB disk space
- Dedicated HDD/SSD for media

**Optimal (High Profile)**
- 16+ GB RAM
- 8+ CPU cores
- 500+ GB disk space
- GPU (Nvidia or Intel QuickSync) for transcoding

### Network
- Static IP or DDNS recommended
- Ports 80, 443 open for web services
- Port 22 for SSH (firewall-protected)

---

## 🛠️ Advanced Installation

### Manual Build from Source

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Clone repository
git clone https://github.com/Cylae/server_script
cd server_script
git checkout 721b5456fa417b5711fd55cf5ddb0d8bebb9597e  # Verify integrity
cd server_manager

# 3. Build release binary
cargo build --release

# 4. Install system-wide
sudo cp target/release/server_manager /usr/local/bin/

# 5. Run installation
sudo server_manager install
```

### Testing Before Deployment

```bash
# Run comprehensive test suite
cd server_manager
cargo test --verbose

# Run benchmarks
cargo bench

# Generate compose file only (no deployment)
sudo server_manager generate
cat docker-compose.yml  # Inspect configuration
```

---

## 📚 CLI Commands

### Installation & Management

```bash
# Full idempotent installation
sudo server_manager install

# Display detected hardware and profile
server_manager status

# Generate docker-compose.yml without deploying
sudo server_manager generate

# Enable/disable specific services
sudo server_manager enable nextcloud
sudo server_manager disable jellyfin

# Start web administration interface
sudo server_manager web --port 8099
```

### User Management

```bash
# Add new user with 50GB quota
sudo server_manager user add john --quota 50 --role Admin

# List all users
sudo server_manager user list

# Change user password
sudo server_manager user passwd john

# Delete user (removes system account and data)
sudo server_manager user delete john
```

**User Roles:**
- `Admin`: Full access to service management
- `Observer`: Read-only dashboard access

---

## 🌐 Web Administration

### Accessing the Dashboard

1. Start the web server:
   ```bash
   sudo server_manager web
   ```

2. Open browser to `http://YOUR-SERVER-IP:8099`

3. Login with default credentials:
   - **Username**: `admin`
   - **Password**: `admin`
   - **⚠️ CHANGE IMMEDIATELY AFTER FIRST LOGIN**

### Features

- **Real-time System Stats**: CPU, RAM, Swap, Disk usage
- **Service Toggle**: Enable/disable services with one click (Admin only)
- **User Management**: Add/remove users, set quotas (Admin only)
- **Session-based Auth**: 24-hour login sessions

---

## 📦 Deployed Services

### Service Matrix

| Category | Service | Port | Access | Description |
|----------|---------|------|--------|-------------|
| **Proxy** | Nginx Proxy Manager | 80, 81, 443 | `http://IP:81` | Reverse proxy & SSL manager |
| **Infra** | Portainer | 9000 | Localhost | Docker management UI |
| | MariaDB | - | Internal | SQL database for apps |
| | Redis | - | Internal | Cache layer |
| | Netdata | 19999 | Localhost | Real-time monitoring |
| | Uptime Kuma | 3001 | Localhost | Uptime monitoring |
| | DNSCrypt | 5300 | - | Encrypted DNS (DoH) |
| | Wireguard | 51820 UDP | - | VPN server |
| **Media** | Plex | 32400 | `http://IP:32400` | Media streaming server |
| | Jellyfin | 8096 | `http://IP:8096` | Open-source media server |
| | Tautulli | 8181 | Localhost | Plex statistics |
| | Overseerr | 5055 | Localhost | Plex request management |
| | Jellyseerr | 5056 | Localhost | Jellyfin request management |
| **Arrs** | Sonarr | 8989 | Localhost | TV show automation |
| | Radarr | 7878 | Localhost | Movie automation |
| | Bazarr | 6767 | Localhost | Subtitle automation |
| | Prowlarr | 9696 | Localhost | Indexer manager |
| | Jackett | 9117 | Localhost | Torrent proxy |
| **Download** | qBittorrent | 8080 | Localhost | Torrent client |
| **Cloud** | Nextcloud | 4443 | `https://IP:4443` | Personal cloud storage |
| | Vaultwarden | 8001 | Localhost | Password manager |
| | Filebrowser | 8002 | Localhost | Web file manager |
| | Syncthing | 8384, 22000 | Mixed | File synchronization |
| **Tools** | Yourls | 8003 | Localhost | URL shortener |
| | GLPI | 8088 | Localhost | IT asset management |
| | Gitea | 3000 | Localhost | Self-hosted Git |
| **Mail** | Mailserver | 25, 143, 587, 993 | Public | Full SMTP/IMAP server |
| | Roundcube | 8090 | Localhost | Webmail interface |

**Legend:**
- `Localhost` = Bound to 127.0.0.1 (requires reverse proxy or SSH tunnel)
- `Public` = Directly accessible from server IP
- `Internal` = Docker network only

---

## 🔒 Security Features

### Built-in Security

1. **Firewall (UFW)**
   - Default deny incoming
   - Whitelisted: SSH (22), HTTP (80), HTTPS (443)
   - Service-specific ports blocked by default

2. **Password Management**
   - Bcrypt hashing (cost factor 12)
   - Auto-generated secrets (32-character hex)
   - Stored in `/opt/server_manager/secrets.yaml` (600 permissions)

3. **Container Security**
   - `no-new-privileges` on sensitive services
   - Isolated Docker networks
   - Resource limits enforced
   - Non-root user execution where possible

4. **Service Isolation**
   - Admin panels bound to localhost
   - Database ports not exposed
   - Reverse proxy required for external access

5. **System Hardening**
   - Fail2Ban enabled
   - Kernel optimizations applied
   - Swap reduced (swappiness=1 on High profile)

### Security Recommendations

```bash
# 1. Change default admin password immediately
sudo server_manager user passwd admin

# 2. Configure SSL in Nginx Proxy Manager
# Access http://YOUR-IP:81
# Add SSL certificates for your domains

# 3. Set up reverse proxy for services
# Map internal ports through Nginx with SSL

# 4. Backup secrets.yaml
sudo cp /opt/server_manager/secrets.yaml ~/backup/

# 5. Enable 2FA where supported (Nextcloud, Vaultwarden)
```

---

## 🎯 Post-Installation Setup

### Essential Configuration

1. **Nginx Proxy Manager** (`http://YOUR-IP:81`)
   - Default: `admin@example.com` / `changeme`
   - Add proxy hosts for services
   - Request Let's Encrypt SSL certificates

2. **Plex** (`http://YOUR-IP:32400/web`)
   - Complete initial setup wizard
   - Add media libraries (`/tv`, `/movies`)
   - Configure remote access

3. **Nextcloud** (`https://YOUR-IP:4443`)
   - Login: `admin` / (see `secrets.yaml`)
   - Complete setup wizard
   - Install recommended apps

4. **Mailserver**
   ```bash
   # Access container CLI
   docker exec -it mailserver setup help
   
   # Add email account
   docker exec -it mailserver setup email add user@yourdomain.com
   ```

5. **ArrStack Integration**
   - Configure Prowlarr indexers
   - Link Sonarr/Radarr to Prowlarr
   - Connect to qBittorrent download client
   - Link Overseerr to Plex

---

## 🔧 Configuration Files

### Key Files

```
/opt/server_manager/
├── docker-compose.yml      # Generated service definitions
├── secrets.yaml            # Auto-generated passwords
├── config.yaml            # Enabled/disabled services
├── users.yaml             # Web dashboard users
└── config/                # Service-specific configs
    ├── plex/
    ├── sonarr/
    ├── nextcloud/
    └── ...
```

### Secrets Management

```yaml
# /opt/server_manager/secrets.yaml
mysql_root_password: a1b2c3d4...
mysql_user_password: e5f6g7h8...
nextcloud_admin_password: i9j0k1l2...
# ... (auto-generated on first install)
```

**To regenerate secrets:**
```bash
sudo rm /opt/server_manager/secrets.yaml
sudo server_manager generate
```

---

## 📊 Monitoring & Maintenance

### Built-in Monitoring

1. **Netdata** (Real-time metrics)
   ```bash
   # Access via SSH tunnel
   ssh -L 19999:localhost:19999 user@YOUR-SERVER-IP
   # Open http://localhost:19999
   ```

2. **Uptime Kuma** (Service health)
   ```bash
   ssh -L 3001:localhost:3001 user@YOUR-SERVER-IP
   # Open http://localhost:3001
   ```

3. **Portainer** (Container management)
   ```bash
   ssh -L 9000:localhost:9000 user@YOUR-SERVER-IP
   # Open http://localhost:9000
   ```

### Logs

```bash
# View all service logs
docker compose logs -f

# Specific service
docker compose logs -f plex

# Check server_manager logs
journalctl -u server_manager -f
```

### Backups

```bash
# Backup configuration
sudo tar -czf server_manager_backup.tar.gz \
  /opt/server_manager/config.yaml \
  /opt/server_manager/secrets.yaml \
  /opt/server_manager/users.yaml \
  /opt/server_manager/config/

# Backup media metadata
sudo tar -czf plex_metadata.tar.gz \
  /opt/server_manager/config/plex/
```

---

## 🐛 Troubleshooting

### Common Issues

**1. Services won't start**
```bash
# Check Docker daemon
sudo systemctl status docker

# Check logs
docker compose logs

# Restart specific service
docker compose restart plex
```

**2. Out of disk space**
```bash
# Check disk usage
df -h
du -sh /opt/server_manager/*

# Clean Docker
docker system prune -af
```

**3. Quota errors**
```bash
# Verify quotas enabled
sudo quotaon -p /home

# Initialize quotas
sudo quotacheck -ugm /home
sudo quotaon -v /home
```

**4. Port conflicts**
```bash
# Find process using port
sudo lsof -i :80

# Stop conflicting service
sudo systemctl stop apache2
sudo systemctl disable apache2
```

**5. GPU not detected**
```bash
# Verify nvidia-smi
nvidia-smi

# Check nvidia-container-toolkit
docker run --rm --gpus all nvidia/cuda:11.0-base nvidia-smi
```

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=debug sudo server_manager install
```

---

## 🔄 Updating

### Update Server Manager

```bash
cd /opt/server_manager_source
git fetch
git checkout 721b5456fa417b5711fd55cf5ddb0d8bebb9597e  # Verify integrity
cd server_manager
cargo build --release
sudo systemctl restart server_manager
```

### Update Services

```bash
cd /opt/server_manager
docker compose pull
docker compose up -d
```

---

## 🧪 Development

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Benchmarks
cargo bench

# Code coverage
cargo tarpaulin --out Html
```

### Project Structure

```
server_manager/
├── src/
│   ├── core/              # System-level operations
│   │   ├── hardware.rs    # Hardware detection
│   │   ├── docker.rs      # Docker management
│   │   ├── config.rs      # Configuration handling
│   │   ├── users.rs       # User management
│   │   └── secrets.rs     # Password generation
│   ├── services/          # Service definitions
│   │   ├── media.rs       # Plex, Jellyfin, etc.
│   │   ├── arr.rs         # Sonarr, Radarr, etc.
│   │   ├── infra.rs       # MariaDB, Redis, etc.
│   │   ├── apps.rs        # Nextcloud, Gitea, etc.
│   │   └── download.rs    # qBittorrent
│   ├── interface/         # User interfaces
│   │   ├── cli.rs         # Command-line interface
│   │   └── web.rs         # Web dashboard
│   ├── lib.rs            # Library root
│   └── main.rs           # Binary entry point
├── tests/                # Integration tests
└── benches/              # Performance benchmarks
```

### Adding a New Service

1. Define service in appropriate module:
   ```rust
   // src/services/apps.rs
   pub struct MyService;
   impl Service for MyService {
       fn name(&self) -> &'static str { "myservice" }
       fn image(&self) -> &'static str { "myimage:latest" }
       fn ports(&self) -> Vec<String> {
           vec!["8080:8080".to_string()]
       }
       // ... implement other traits
   }
   ```

2. Register in service list:
   ```rust
   // src/services/mod.rs
   pub fn get_all_services() -> &'static [Box<dyn Service>] {
       // ...
       Box::new(apps::MyService),
       // ...
   }
   ```

3. Test and rebuild:
   ```bash
   cargo test
   cargo build --release
   ```

---

## 🤝 Contributing

### Guidelines

1. All code must pass `cargo test`
2. Follow Rust style guidelines (`cargo fmt`)
3. Run `cargo clippy` and fix warnings
4. Add tests for new features
5. Update documentation

### Pull Request Process

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

---

## 📜 License

This project is licensed under the MIT License.

---

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Powered by [Docker](https://www.docker.com/)
- LinuxServer.io for excellent container images
- Community contributors

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/Cylae/server_script/issues)
- **Documentation**: This README
- **Logs**: `/var/log/my-server-install.log`

---

## 🗺️ Roadmap

### Version 1.1 (Planned)
- [ ] Automatic backup scheduling
- [ ] Email notifications for system events
- [ ] Mobile app for dashboard
- [ ] Kubernetes deployment option

### Version 1.2 (Future)
- [ ] Multi-server management
- [ ] Advanced metrics (Prometheus/Grafana)
- [ ] Automated SSL renewal
- [ ] Plugin system for custom services

---

**Built with ❤️ by the Server Manager Team**

*Last Updated: 2026-02-06 - Version 1.0.5*
