use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HardwareSpecs {
    pub ram_gb: u64,
    pub swap_gb: u64,
    pub cpu_cores: usize,
    pub disk_gb: u64,
    pub has_nvidia: bool,
    pub has_intel_quicksync: bool,
}

#[async_trait]
pub trait RuntimePort: Send + Sync {
    fn is_docker_installed(&self) -> bool;
    async fn install_docker(&self) -> Result<()>;
    async fn run_compose_up(&self, workdir: &Path) -> Result<()>;
    fn detect_hardware(&self) -> HardwareSpecs;
    fn detect_user_context(&self) -> (String, String);
}

#[async_trait]
pub trait SystemPort: Send + Sync {
    fn check_root(&self) -> Result<()>;
    async fn install_packages(&self, pkgs: Vec<String>) -> Result<()>;
    async fn apply_optimizations(&self, ram_gb: u64) -> Result<()>;
    async fn configure_firewall(&self) -> Result<()>;

    // User Management
    async fn create_user(&self, username: &str, password: &str) -> Result<()>;
    async fn delete_user(&self, username: &str) -> Result<()>;
    async fn set_password(&self, username: &str, password: &str) -> Result<()>;
    async fn set_quota(&self, username: &str, quota_gb: u64) -> Result<()>;
    fn get_uid(&self, username: &str) -> Result<u32>;

    // Filesystem
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn file_exists(&self, path: &Path) -> bool;

    // Service management
    async fn stop_and_disable_service(&self, name: &str) -> Result<()>;
}
