use crate::ports::{HardwareSpecs, RuntimePort, SystemPort};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

pub struct MockRuntime {
    pub specs: HardwareSpecs,
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self {
            specs: HardwareSpecs {
                ram_gb: 8,
                swap_gb: 2,
                cpu_cores: 4,
                disk_gb: 512,
                has_nvidia: false,
                has_intel_quicksync: false,
            },
        }
    }
}

#[async_trait]
impl RuntimePort for MockRuntime {
    fn is_docker_installed(&self) -> bool {
        true
    }
    async fn install_docker(&self) -> Result<()> {
        Ok(())
    }
    async fn run_compose_up(&self, _workdir: &Path) -> Result<()> {
        Ok(())
    }
    fn detect_hardware(&self) -> HardwareSpecs {
        self.specs.clone()
    }
    fn detect_user_context(&self) -> (String, String) {
        ("1000".to_string(), "1000".to_string())
    }
}

#[async_trait]
impl SystemPort for MockRuntime {
    fn check_root(&self) -> Result<()> {
        Ok(())
    }
    async fn install_packages(&self, _pkgs: Vec<String>) -> Result<()> {
        Ok(())
    }
    async fn apply_optimizations(&self, _ram_gb: u64) -> Result<()> {
        Ok(())
    }
    async fn configure_firewall(&self) -> Result<()> {
        Ok(())
    }
    async fn create_user(&self, _u: &str, _p: &str) -> Result<()> {
        Ok(())
    }
    async fn delete_user(&self, _u: &str) -> Result<()> {
        Ok(())
    }
    async fn set_password(&self, _u: &str, _p: &str) -> Result<()> {
        Ok(())
    }
    async fn set_quota(&self, _u: &str, _q: u64) -> Result<()> {
        Ok(())
    }
    fn get_uid(&self, _u: &str) -> Result<u32> {
        Ok(1000)
    }
    fn write_file(&self, _path: &Path, _content: &str) -> Result<()> {
        Ok(())
    }
    fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
    fn file_exists(&self, _path: &Path) -> bool {
        true
    }
    async fn stop_and_disable_service(&self, _name: &str) -> Result<()> {
        Ok(())
    }
}
