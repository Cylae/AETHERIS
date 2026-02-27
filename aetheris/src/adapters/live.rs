use anyhow::{Result, Context, bail};
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use std::fs;
use std::io::Write;
use which::which;
use nix::unistd::{Uid, User};
use sysinfo::{System, SystemExt, DiskExt};
use crate::ports::{RuntimePort, SystemPort, HardwareSpecs};

#[derive(Clone)]
pub struct LiveAdapter;

#[async_trait]
impl RuntimePort for LiveAdapter {
    fn is_docker_installed(&self) -> bool {
        which("docker").is_ok()
    }

    async fn install_docker(&self) -> Result<()> {
        let status = Command::new("curl").args(["-fsSL", "https://get.docker.com", "-o", "get-docker.sh"]).status()?;
        if !status.success() { bail!("Failed to download docker install script"); }
        let status = Command::new("sh").arg("get-docker.sh").status()?;
        if !status.success() { bail!("Failed to execute docker install script"); }
        Ok(())
    }

    async fn run_compose_up(&self, workdir: &Path) -> Result<()> {
        let status = Command::new("docker")
            .args(["compose", "up", "-d", "--remove-orphans"])
            .current_dir(workdir)
            .status()?;
        if !status.success() { bail!("Docker compose failed"); }
        Ok(())
    }

    fn detect_hardware(&self) -> HardwareSpecs {
        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu();
        sys.refresh_disks_list();
        let ram = sys.total_memory() / 1024 / 1024 / 1024;
        let swap = sys.total_swap() / 1024 / 1024 / 1024;
        let cores = sys.cpus().len();
        let disk = sys.disks().iter().map(|d| d.total_space()).sum::<u64>() / 1024 / 1024 / 1024;

        let has_nvidia = which("nvidia-smi").is_ok() &&
            (which("nvidia-container-cli").is_ok() || which("nvidia-container-runtime").is_ok());

        let has_intel_quicksync = Path::new("/dev/dri").exists();

        HardwareSpecs {
            ram_gb: ram,
            swap_gb: swap,
            cpu_cores: cores,
            disk_gb: disk,
            has_nvidia,
            has_intel_quicksync,
        }
    }

    fn detect_user_context(&self) -> (String, String) {
        if let (Ok(uid), Ok(gid)) = (std::env::var("SUDO_UID"), std::env::var("SUDO_GID")) {
            return (uid, gid);
        }

        if let Ok(username) = std::env::var("SUDO_USER") {
            if let Ok(Some(user)) = User::from_name(&username) {
                return (user.uid.to_string(), user.gid.to_string());
            }
        }
        ("1000".to_string(), "1000".to_string())
    }
}

#[async_trait]
impl SystemPort for LiveAdapter {
    fn check_root(&self) -> Result<()> {
        if !Uid::effective().is_root() {
            bail!("This application must be run as root.");
        }
        Ok(())
    }

    async fn install_packages(&self, pkgs: Vec<String>) -> Result<()> {
        if which("apt-get").is_err() {
            bail!("apt-get not found. This tool supports Debian/Ubuntu based systems only.");
        }
        Command::new("apt-get").arg("update").status()?;
        let status = Command::new("apt-get").arg("install").arg("-y").args(&pkgs).status()?;
        if !status.success() { bail!("apt-get install failed"); }
        Ok(())
    }

    async fn apply_optimizations(&self, ram_gb: u64) -> Result<()> {
        let swappiness = if ram_gb > 16 { 1 } else { 10 };
        let config = format!("vm.swappiness={}\nfs.inotify.max_user_watches=524288\n", swappiness);
        let path = Path::new("/etc/sysctl.d/99-aetheris-optimization.conf");
        tokio::fs::write(path, config).await?;
        tokio::process::Command::new("sysctl").arg("--system").status().await?;
        Ok(())
    }

    async fn configure_firewall(&self) -> Result<()> {
        if which("ufw").is_err() { return Ok(()); }
        Command::new("ufw").args(["default", "deny", "incoming"]).status()?;
        Command::new("ufw").args(["default", "allow", "outgoing"]).status()?;
        Command::new("ufw").args(["allow", "ssh"]).status()?;
        let status = Command::new("ufw").args(["--force", "enable"]).status()?;
        if !status.success() { bail!("Failed to enable ufw"); }
        Ok(())
    }

    async fn create_user(&self, username: &str, password: &str) -> Result<()> {
        let status = Command::new("useradd").args(["-m", "-s", "/bin/bash", username]).status()?;
        if !status.success() { bail!("useradd failed"); }
        self.set_password(username, password).await
    }

    async fn delete_user(&self, username: &str) -> Result<()> {
        let status = Command::new("userdel").args(["-r", username]).status()?;
        if !status.success() { bail!("userdel failed"); }
        Ok(())
    }

    async fn set_password(&self, username: &str, password: &str) -> Result<()> {
        let mut child = Command::new("chpasswd").stdin(std::process::Stdio::piped()).spawn()?;
        {
            let stdin = child.stdin.as_mut().context("Failed to open stdin")?;
            stdin.write_all(format!("{}:{}", username, password).as_bytes())?;
        }
        let status = child.wait()?;
        if !status.success() { bail!("chpasswd failed"); }
        Ok(())
    }

    async fn set_quota(&self, username: &str, quota_gb: u64) -> Result<()> {
        let blocks = quota_gb * 1024 * 1024;
        let status = Command::new("setquota").args(["-u", username, &blocks.to_string(), &blocks.to_string(), "0", "0", "/home"]).status()?;
        if !status.success() { bail!("setquota failed"); }
        Ok(())
    }

    fn get_uid(&self, username: &str) -> Result<u32> {
        match User::from_name(username)? {
            Some(user) => Ok(user.uid.as_raw()),
            None => bail!("User not found"),
        }
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        fs::write(path, content).map_err(Into::into)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).map_err(Into::into)
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    async fn stop_and_disable_service(&self, name: &str) -> Result<()> {
        let _ = Command::new("systemctl").args(["stop", name]).status();
        let _ = Command::new("systemctl").args(["disable", name]).status();
        Ok(())
    }
}
