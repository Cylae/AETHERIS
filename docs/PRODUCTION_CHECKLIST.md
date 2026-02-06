# 🚀 Production Deployment Checklist

## Pre-Deployment Security Audit

### Critical Security Fixes (MUST DO BEFORE PRODUCTION)

- [ ] **P1: Password Input Security**
  - Apply `SECURITY_PATCH_password_input.rs`
  - Add `rpassword = "7.3"` to Cargo.toml
  - Test password input in terminal (verify no echo)
  - Verify password not in shell history

- [ ] **P1: Change Default Credentials**
  ```bash
  sudo server_manager user passwd admin
  # Set strong password (20+ chars, mixed case, numbers, symbols)
  ```

- [ ] **P2: Secrets Backup**
  ```bash
  sudo cp /opt/server_manager/secrets.yaml ~/secure_backup/
  sudo chmod 600 ~/secure_backup/secrets.yaml
  ```

- [ ] **P2: Enable CSRF Protection**
  - Add CSRF tokens to web forms (future enhancement)
  - For now: Use reverse proxy with rate limiting

## System Hardening

### Firewall Configuration

- [ ] **UFW Status Verification**
  ```bash
  sudo ufw status verbose
  # Should show:
  # Status: active
  # 22/tcp ALLOW IN
  # 80/tcp ALLOW IN  
  # 443/tcp ALLOW IN
  ```

- [ ] **Close Unnecessary Ports**
  ```bash
  # List listening ports
  sudo netstat -tulpn
  
  # Block any unexpected services
  sudo ufw deny <PORT>
  ```

- [ ] **Configure Fail2Ban**
  ```bash
  sudo systemctl status fail2ban
  # Check SSH jail is active
  sudo fail2ban-client status sshd
  ```

### SSL/TLS Configuration

- [ ] **Nginx Proxy Manager Setup**
  1. Access http://YOUR-IP:81
  2. Login with: `admin@example.com` / `changeme`
  3. Change admin password immediately
  4. Add SSL certificates for domains
  5. Force HTTPS redirects
  6. Enable HSTS headers

- [ ] **Let's Encrypt Certificate**
  ```
  For each service:
  1. Add Proxy Host in NPM
  2. Forward Scheme: http
  3. Forward Hostname/IP: localhost
  4. Forward Port: <service_port>
  5. Enable SSL, Request Let's Encrypt cert
  6. Force SSL
  ```

- [ ] **Verify SSL Configuration**
  ```bash
  # Test with SSLLabs
  https://www.ssllabs.com/ssltest/
  # Should achieve A+ rating
  ```

## Service Configuration

### Media Services

- [ ] **Plex Configuration**
  1. Complete setup wizard at http://YOUR-IP:32400/web
  2. Sign in with Plex account
  3. Add media libraries:
     - TV Shows → `/tv`
     - Movies → `/movies`
  4. Configure remote access (Settings → Remote Access)
  5. Enable hardware transcoding (Settings → Transcoder)
  6. Set up library auto-updates

- [ ] **Jellyfin Configuration** (if using)
  1. Access http://YOUR-IP:8096
  2. Complete initial setup
  3. Add libraries (same paths as Plex)
  4. Configure hardware acceleration
  5. Set up remote access

- [ ] **ArrStack Setup**
  ```
  Prowlarr (9696):
  1. Add indexers
  2. Configure authentication
  
  Sonarr (8989):
  1. Link to Prowlarr
  2. Add root folder: /media/tv
  3. Connect download client (qBittorrent)
  4. Set quality profiles
  
  Radarr (7878):
  1. Link to Prowlarr  
  2. Add root folder: /media/movies
  3. Connect download client
  4. Set quality profiles
  
  Bazarr (6767):
  1. Connect to Sonarr/Radarr
  2. Configure subtitle providers
  ```

- [ ] **qBittorrent Configuration**
  1. Access http://localhost:8080 (via SSH tunnel)
  2. Default credentials: `admin` / `adminadmin`
  3. Change password immediately
  4. Set download directory: `/downloads`
  5. Enable WebUI authentication
  6. Configure connection limits

### Cloud Services

- [ ] **Nextcloud Setup**
  1. Access https://YOUR-IP:4443
  2. Login: `admin` / (check secrets.yaml)
  3. Install recommended apps:
     - Calendar
     - Contacts
     - Notes
     - Tasks
  4. Configure external storage (optional)
  5. Set up email notifications
  6. Enable 2FA for admin account
  7. Create user accounts

- [ ] **Vaultwarden Configuration**
  1. Access http://localhost:8001 (via SSH tunnel)
  2. Access admin panel: /admin
  3. Enter admin token from secrets.yaml
  4. Disable user registration (if desired)
  5. Configure SMTP for invitations
  6. Enable 2FA requirement

- [ ] **Gitea Setup**
  1. Access http://localhost:3000
  2. Complete installation wizard
  3. Database: MySQL (pre-configured)
  4. Create admin account
  5. Configure SSH access (port 2222)
  6. Set up webhooks (if needed)

### Mail Services

- [ ] **Mailserver Configuration**
  ```bash
  # Add email accounts
  docker exec -it mailserver setup email add user@yourdomain.com
  docker exec -it mailserver setup email add postmaster@yourdomain.com
  
  # Configure DKIM
  docker exec -it mailserver setup config dkim
  
  # Test mail delivery
  docker exec -it mailserver setup debug mail
  ```

- [ ] **DNS Records**
  ```
  Add these to your domain DNS:
  
  MX    @  yourdomain.com  (priority 10)
  A     @  YOUR-SERVER-IP
  TXT   @  v=spf1 mx ~all
  TXT   _dmarc  v=DMARC1; p=quarantine; rua=mailto:postmaster@yourdomain.com
  TXT   mail._domainkey  (DKIM public key from setup)
  ```

- [ ] **Roundcube Webmail**
  1. Access http://localhost:8090
  2. Login with email credentials
  3. Configure identity
  4. Test send/receive

## Monitoring Setup

- [ ] **Netdata Configuration**
  ```bash
  # Access via SSH tunnel
  ssh -L 19999:localhost:19999 user@YOUR-SERVER
  
  # Configure alarms
  # Edit /opt/server_manager/config/netdata/health.d/
  ```

- [ ] **Uptime Kuma Setup**
  1. Access http://localhost:3001 (SSH tunnel)
  2. Create admin account
  3. Add monitors for critical services:
     - Plex (http://localhost:32400)
     - Nginx (http://localhost:80)
     - Nextcloud (https://localhost:4443)
  4. Configure notifications (email, Discord, etc.)

- [ ] **Portainer Dashboard**
  1. Access http://localhost:9000
  2. Create admin account
  3. Connect local Docker environment
  4. Review container resource usage

## Backup Strategy

- [ ] **Configuration Backup Script**
  ```bash
  cat > /root/backup_server_manager.sh << 'EOF'
  #!/bin/bash
  BACKUP_DIR="/root/backups/server_manager_$(date +%Y%m%d)"
  mkdir -p "$BACKUP_DIR"
  
  # Backup configs
  cp /opt/server_manager/secrets.yaml "$BACKUP_DIR/"
  cp /opt/server_manager/config.yaml "$BACKUP_DIR/"
  cp /opt/server_manager/users.yaml "$BACKUP_DIR/"
  
  # Backup service configs
  tar -czf "$BACKUP_DIR/service_configs.tar.gz" /opt/server_manager/config/
  
  # Backup Plex metadata
  tar -czf "$BACKUP_DIR/plex_metadata.tar.gz" /opt/server_manager/config/plex/
  
  # Encrypt backup
  tar -czf - "$BACKUP_DIR" | openssl enc -aes-256-cbc -salt -out "$BACKUP_DIR.tar.gz.enc"
  rm -rf "$BACKUP_DIR"
  
  # Keep last 7 days
  find /root/backups/ -name "server_manager_*.enc" -mtime +7 -delete
  EOF
  
  chmod +x /root/backup_server_manager.sh
  ```

- [ ] **Schedule Daily Backups**
  ```bash
  # Add to root crontab
  crontab -e
  
  # Add line:
  0 2 * * * /root/backup_server_manager.sh
  ```

- [ ] **Test Backup Restoration**
  ```bash
  # Decrypt test
  openssl enc -d -aes-256-cbc -in /root/backups/server_manager_YYYYMMDD.tar.gz.enc -out test.tar.gz
  
  # Extract and verify
  tar -xzf test.tar.gz
  ```

## Performance Optimization

- [ ] **Verify Hardware Profile**
  ```bash
  server_manager status
  # Confirm correct profile detected
  ```

- [ ] **Check Disk Space**
  ```bash
  df -h
  # Ensure at least 20% free on /opt partition
  ```

- [ ] **Monitor Resource Usage**
  ```bash
  docker stats
  # Verify no services constantly at limit
  ```

- [ ] **Optimize MariaDB** (if needed)
  ```bash
  # Edit /opt/server_manager/config/mariadb/custom.cnf
  # Adjust based on available RAM
  ```

- [ ] **Enable Transcoding Optimization**
  - If GPU detected, verify passthrough working
  - Test Plex/Jellyfin transcoding
  - Monitor GPU usage with `nvidia-smi` or `intel_gpu_top`

## User Management

- [ ] **Create User Accounts**
  ```bash
  # For each user:
  sudo server_manager user add username --quota 100 --role Observer
  
  # Save credentials securely
  # Users stored in /opt/server_manager/users.yaml
  ```

- [ ] **Set Quotas**
  ```bash
  # Verify quotas enabled
  sudo quotaon -p /home
  
  # Check user quotas
  sudo quota -v username
  ```

- [ ] **Test User Access**
  ```bash
  # SSH as user
  ssh username@YOUR-SERVER
  
  # Verify quota
  quota -s
  
  # Test file creation
  dd if=/dev/zero of=~/testfile bs=1M count=100
  ```

## Network Configuration

- [ ] **Configure Reverse Proxy**
  ```
  For each service in Nginx Proxy Manager:
  
  Service      | Domain                    | Internal Port
  -------------|---------------------------|---------------
  Plex         | plex.yourdomain.com       | localhost:32400
  Nextcloud    | cloud.yourdomain.com      | localhost:4443
  Overseerr    | requests.yourdomain.com   | localhost:5055
  Sonarr       | sonarr.yourdomain.com     | localhost:8989
  Radarr       | radarr.yourdomain.com     | localhost:7878
  ```

- [ ] **Test External Access**
  ```bash
  # From external network:
  curl https://plex.yourdomain.com
  curl https://cloud.yourdomain.com
  
  # Should return 200 OK (or redirect to login)
  ```

- [ ] **Configure DDNS** (if no static IP)
  - Choose provider (DuckDNS, No-IP, etc.)
  - Set up auto-update script
  - Test domain resolution

## Documentation

- [ ] **Create Admin Documentation**
  ```
  Document in /root/ADMIN_NOTES.md:
  - All changed default passwords
  - Custom port mappings
  - Service-specific configurations
  - Backup encryption password
  - Domain/DNS settings
  ```

- [ ] **Create User Guide**
  ```
  For end users:
  - How to access services
  - Login credentials (their own)
  - Basic troubleshooting
  - Support contact
  ```

## Final Validation

- [ ] **Run Full Test Suite**
  ```bash
  cd /opt/server_manager_source/server_manager
  cargo test --verbose
  # All tests should pass
  ```

- [ ] **Security Scan**
  ```bash
  # Check for open ports
  nmap -sV YOUR-SERVER-IP
  
  # Should only show 22, 80, 443 open
  ```

- [ ] **Load Test** (optional)
  ```bash
  # Test concurrent logins
  # Test simultaneous transcodes
  # Monitor system resources
  ```

- [ ] **Disaster Recovery Test**
  1. Stop all services
  2. Delete config directory
  3. Restore from backup
  4. Verify all services start correctly

## Post-Deployment

- [ ] **Monitor for 24 Hours**
  - Check logs: `docker compose logs -f`
  - Monitor resource usage
  - Test all critical services
  - Verify backups running

- [ ] **Update README with Custom Settings**
  ```bash
  # Document any deviations from default setup
  echo "# Custom Configuration" >> /opt/server_manager/CUSTOM_README.md
  ```

- [ ] **Schedule Maintenance Window**
  ```
  Monthly:
  - Update Docker images
  - Review logs for errors
  - Test backups
  - Update OS packages
  
  Quarterly:
  - Review user quotas
  - Audit user accounts
  - Update SSL certificates (if not auto-renew)
  - Review firewall rules
  ```

## Emergency Contacts

- [ ] **Document Support Channels**
  ```
  Technical Issues:
  - GitHub Issues: https://github.com/Cylae/server_script/issues
  - System Logs: /var/log/my-server-install.log
  
  Service-Specific:
  - Plex Forums: https://forums.plex.tv/
  - Nextcloud Help: https://help.nextcloud.com/
  ```

## Sign-Off

```
Deployed by: _______________________
Date: _______________________
Server IP: _______________________
Domain(s): _______________________

Production Checklist Complete: [ ]
All Critical Services Running: [ ]
Backups Verified: [ ]
Security Audit Passed: [ ]

Signature: _______________________
```

---

**This checklist should be completed in order. Do not skip critical security items.**

**Estimated time to complete: 4-8 hours for first deployment**
