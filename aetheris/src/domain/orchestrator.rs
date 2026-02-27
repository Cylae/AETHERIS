use anyhow::{Result, Context};
use std::path::Path;
use log::info;
use crate::ports::{RuntimePort, SystemPort};
use crate::core::{hardware, secrets, config};
use crate::services;
use crate::build_compose_structure;

pub struct AetherisOrchestrator {
    runtime: Box<dyn RuntimePort>,
    system: Box<dyn SystemPort>,
}

impl AetherisOrchestrator {
    pub fn new(runtime: Box<dyn RuntimePort>, system: Box<dyn SystemPort>) -> Self {
        Self { runtime, system }
    }

    pub async fn install(&self) -> Result<()> {
        info!("Starting Project AETHERIS Installation...");

        self.system.check_root()?;

        let install_path_env = std::env::var("AETHERIS_HOME").unwrap_or_else(|_| "/opt/aetheris".to_string());
        let install_dir = Path::new(&install_path_env);

        if !self.system.file_exists(install_dir) {
            info!("Creating installation directory at {:?}...", install_dir);
            self.system.create_dir_all(install_dir).with_context(|| format!("Failed to create {:?}", install_dir))?;
        }
        std::env::set_current_dir(install_dir).with_context(|| format!("Failed to chdir to {:?}", install_dir))?;

        let specs = self.runtime.detect_hardware();
        let (uid, gid) = self.runtime.detect_user_context();
        let hw = hardware::HardwareInfo::from_specs(specs.clone(), uid, gid);

        let pkgs = vec![
            "curl".to_string(), "git".to_string(), "ufw".to_string(),
            "lsb-release".to_string(), "ca-certificates".to_string(), "gnupg".to_string(),
            "htop".to_string(), "iotop".to_string(), "net-tools".to_string(),
            "quota".to_string(), "build-essential".to_string()
        ];

        info!("Installing system dependencies...");
        self.system.install_packages(pkgs).await?;

        info!("Applying system optimizations...");
        self.system.apply_optimizations(specs.ram_gb).await?;

        info!("Configuring firewall...");
        self.system.configure_firewall().await?;

        if !self.runtime.is_docker_installed() {
            info!("Installing Docker...");
            self.runtime.install_docker().await?;
        }

        let secrets = secrets::Secrets::load_or_create("secrets.yaml")?;
        let config = config::Config::load()?;

        info!("Initializing services...");
        self.initialize_services(&hw, &secrets, &config).await?;

        info!("Configuring services...");
        self.configure_services(&hw, &secrets, &config).await?;

        info!("Generating docker-compose.yml...");
        self.generate_compose(&hw, &secrets, &config).await?;

        info!("Launching services via Docker Compose...");
        self.runtime.run_compose_up(Path::new(".")).await?;

        info!("AETHERIS Stack Deployed Successfully! 🚀");
        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        let specs = self.runtime.detect_hardware();
        let (uid, gid) = self.runtime.detect_user_context();
        let hw = hardware::HardwareInfo::from_specs(specs.clone(), uid, gid);

        println!("=== AETHERIS System Status ===");
        println!("RAM: {} GB", hw.ram_gb);
        println!("Swap: {} GB", hw.swap_gb);
        println!("Disk: {} GB", hw.disk_gb);
        println!("Cores: {}", hw.cpu_cores);
        println!("Profile: {:?}", hw.profile);
        println!("Nvidia GPU: {}", hw.has_nvidia);
        println!("Intel QuickSync: {}", hw.has_intel_quicksync);

        println!("\n=== Runtime Status ===");
        if self.runtime.is_docker_installed() {
             println!("Docker: Installed");
        } else {
             println!("Docker: NOT Installed");
        }
        Ok(())
    }

    async fn configure_services(&self, hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, config: &config::Config) -> Result<()> {
        let all_services = services::get_all_services();
        for service in all_services {
            if config.is_enabled(service.name()) {
                service.configure(self.system.as_ref(), hw, secrets).await?;
            }
        }
        Ok(())
    }

    async fn initialize_services(&self, hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, config: &config::Config) -> Result<()> {
        let all_services = services::get_all_services();
        for service in all_services {
            if config.is_enabled(service.name()) {
                service.initialize(self.system.as_ref(), hw, secrets).await?;
            }
        }
        Ok(())
    }

    async fn generate_compose(&self, hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, config: &config::Config) -> Result<()> {
        let top_level = build_compose_structure(hw, secrets, config)?;
        let yaml_output = serde_yaml_ng::to_string(&top_level)?;
        self.system.write_file(Path::new("docker-compose.yml"), &yaml_output)?;
        Ok(())
    }
}
