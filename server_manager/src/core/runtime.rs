use anyhow::{Result, Context, bail};
use std::path::Path;
use std::process::Command;
use std::fs;
use std::io::Write;
use log::info;
use which::which;
use nix::unistd::{Uid, User};
use sysinfo::{System, SystemExt, DiskExt};

use crate::core::hardware::HardwareInfo;

pub trait SystemRuntime: Send + Sync {
    fn check_root(&self) -> Result<()>;
    fn install_dependencies(&self) -> Result<()>;
    fn apply_optimizations(&self, hw: &HardwareInfo) -> Result<()>;
    fn create_system_user(&self, username: &str, password: &str) -> Result<()>;
    fn delete_system_user(&self, username: &str) -> Result<()>;
    fn set_system_user_password(&self, username: &str, password: &str) -> Result<()>;
    fn set_system_quota(&self, username: &str, quota_gb: u64) -> Result<()>;
    fn configure_firewall(&self) -> Result<()>;
    fn get_uid(&self, username: &str) -> Result<u32>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    fn file_exists(&self, path: &Path) -> bool;
    fn stop_and_disable_service(&self, service_name: &str) -> Result<()>;
}

pub trait DockerRuntime: Send + Sync {
    fn is_installed(&self) -> bool;
    fn install(&self) -> Result<()>;
    fn run_compose_up(&self) -> Result<()>;
}

pub trait HardwareProbe: Send + Sync {
    fn detect_hardware(&self) -> (u64, u64, usize, u64); // ram, swap, cores, disk
    fn has_nvidia(&self) -> bool;
    fn has_intel_quicksync(&self) -> bool;
    fn detect_user_context(&self) -> (String, String);
}

pub struct LinuxRuntime;

impl SystemRuntime for LinuxRuntime {
    fn check_root(&self) -> Result<()> {
        if !Uid::effective().is_root() {
            bail!("This application must be run as root.");
        }
        Ok(())
    }

    fn install_dependencies(&self) -> Result<()> {
        info!("Checking system dependencies...");
        if which("apt-get").is_err() {
            bail!("apt-get not found. This tool supports Debian/Ubuntu based systems only.");
        }

        let pkgs = vec![
            "curl", "git", "ufw", "lsb-release", "ca-certificates", "gnupg",
            "htop", "iotop", "net-tools", "quota", "build-essential"
        ];

        Command::new("apt-get").arg("update").status()?;
        Command::new("apt-get").arg("install").arg("-y").args(&pkgs).status()?;
        Ok(())
    }

    fn apply_optimizations(&self, hw: &HardwareInfo) -> Result<()> {
        let swappiness = if hw.ram_gb > 16 { 1 } else { 10 };
        let config = format!("vm.swappiness={}\nfs.inotify.max_user_watches=524288\n", swappiness);
        let path = Path::new("/etc/sysctl.d/99-server-manager-optimization.conf");
        fs::write(path, config)?;
        Command::new("sysctl").arg("--system").status()?;
        Ok(())
    }

    fn create_system_user(&self, username: &str, password: &str) -> Result<()> {
        let status = Command::new("useradd").args(["-m", "-s", "/bin/bash", username]).status()?;
        if !status.success() { bail!("useradd failed"); }
        self.set_system_user_password(username, password)
    }

    fn delete_system_user(&self, username: &str) -> Result<()> {
        let status = Command::new("userdel").args(["-r", username]).status()?;
        if !status.success() { bail!("userdel failed"); }
        Ok(())
    }

    fn set_system_user_password(&self, username: &str, password: &str) -> Result<()> {
        let mut child = Command::new("chpasswd").stdin(std::process::Stdio::piped()).spawn()?;
        {
            let stdin = child.stdin.as_mut().context("Failed to open stdin")?;
            stdin.write_all(format!("{}:{}", username, password).as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }

    fn set_system_quota(&self, username: &str, quota_gb: u64) -> Result<()> {
        // Implementation from system.rs
        let blocks = quota_gb * 1024 * 1024;
        Command::new("setquota").args(["-u", username, &blocks.to_string(), &blocks.to_string(), "0", "0", "/home"]).status()?;
        Ok(())
    }

    fn configure_firewall(&self) -> Result<()> {
        if which("ufw").is_err() { return Ok(()); }
        Command::new("ufw").args(["default", "deny", "incoming"]).status()?;
        Command::new("ufw").args(["default", "allow", "outgoing"]).status()?;
        Command::new("ufw").args(["allow", "ssh"]).status()?;
        Command::new("ufw").args(["--force", "enable"]).status()?;
        Ok(())
    }

    fn get_uid(&self, username: &str) -> Result<u32> {
        match User::from_name(username)? {
            Some(user) => Ok(user.uid.as_raw()),
            None => bail!("User not found"),
        }
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).map_err(Into::into)
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        fs::write(path, content).map_err(Into::into)
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn stop_and_disable_service(&self, service_name: &str) -> Result<()> {
        let _ = Command::new("systemctl").args(["stop", service_name]).status();
        let _ = Command::new("systemctl").args(["disable", service_name]).status();
        Ok(())
    }
}

impl DockerRuntime for LinuxRuntime {
    fn is_installed(&self) -> bool {
        which("docker").is_ok()
    }

    fn install(&self) -> Result<()> {
        Command::new("curl").args(["-fsSL", "https://get.docker.com", "-o", "get-docker.sh"]).status()?;
        Command::new("sh").arg("get-docker.sh").status()?;
        Ok(())
    }

    fn run_compose_up(&self) -> Result<()> {
        let status = Command::new("docker").args(["compose", "up", "-d", "--remove-orphans"]).status()?;
        if !status.success() { bail!("Docker compose failed"); }
        Ok(())
    }
}

impl HardwareProbe for LinuxRuntime {
    fn detect_hardware(&self) -> (u64, u64, usize, u64) {
        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu();
        sys.refresh_disks_list();
        let ram = sys.total_memory() / 1024 / 1024 / 1024;
        let swap = sys.total_swap() / 1024 / 1024 / 1024;
        let cores = sys.cpus().len();
        let disk = sys.disks().iter().map(|d| d.total_space()).sum::<u64>() / 1024 / 1024 / 1024;
        (ram, swap, cores, disk)
    }

    fn has_nvidia(&self) -> bool {
        let has_smi = which("nvidia-smi").is_ok();
        let has_cli = which("nvidia-container-cli").is_ok();
        let has_runtime = which("nvidia-container-runtime").is_ok();
        has_smi && (has_cli || has_runtime)
    }

    fn has_intel_quicksync(&self) -> bool {
        Path::new("/dev/dri").exists()
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

pub struct MockRuntime {
    pub ram_gb: u64,
    pub swap_gb: u64,
    pub cpu_cores: usize,
    pub disk_gb: u64,
    pub has_nvidia: bool,
    pub has_intel_quicksync: bool,
    pub docker_installed: bool,
    pub docker_fail: bool,
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self {
            ram_gb: 8,
            swap_gb: 2,
            cpu_cores: 4,
            disk_gb: 512,
            has_nvidia: false,
            has_intel_quicksync: false,
            docker_installed: true,
            docker_fail: false,
        }
    }
}

impl SystemRuntime for MockRuntime {
    fn check_root(&self) -> Result<()> { Ok(()) }
    fn install_dependencies(&self) -> Result<()> { Ok(()) }
    fn apply_optimizations(&self, _hw: &HardwareInfo) -> Result<()> { Ok(()) }
    fn create_system_user(&self, _u: &str, _p: &str) -> Result<()> { Ok(()) }
    fn delete_system_user(&self, _u: &str) -> Result<()> { Ok(()) }
    fn set_system_user_password(&self, _u: &str, _p: &str) -> Result<()> { Ok(()) }
    fn set_system_quota(&self, _u: &str, _q: u64) -> Result<()> { Ok(()) }
    fn configure_firewall(&self) -> Result<()> { Ok(()) }
    fn get_uid(&self, _u: &str) -> Result<u32> { Ok(1000) }
    fn create_dir_all(&self, _path: &Path) -> Result<()> { Ok(()) }
    fn write_file(&self, _path: &Path, _content: &str) -> Result<()> { Ok(()) }
    fn file_exists(&self, _path: &Path) -> bool { false }
    fn stop_and_disable_service(&self, _service_name: &str) -> Result<()> { Ok(()) }
}

impl DockerRuntime for MockRuntime {
    fn is_installed(&self) -> bool { self.docker_installed }
    fn install(&self) -> Result<()> {
        if self.docker_fail { bail!("Mock Docker install failure"); }
        Ok(())
    }
    fn run_compose_up(&self) -> Result<()> {
        if self.docker_fail { bail!("Mock Docker compose failure"); }
        Ok(())
    }
}

impl HardwareProbe for MockRuntime {
    fn detect_hardware(&self) -> (u64, u64, usize, u64) {
        (self.ram_gb, self.swap_gb, self.cpu_cores, self.disk_gb)
    }
    fn has_nvidia(&self) -> bool { self.has_nvidia }
    fn has_intel_quicksync(&self) -> bool { self.has_intel_quicksync }
    fn detect_user_context(&self) -> (String, String) {
        ("1000".to_string(), "1000".to_string())
    }
}
