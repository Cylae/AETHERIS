pub mod media;
pub mod arr;
pub mod infra;
pub mod download;
pub mod apps;

use crate::core::hardware::HardwareInfo;
use crate::core::secrets::Secrets;
use crate::ports::SystemPort;
use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub memory_limit: Option<String>,
    pub memory_reservation: Option<String>,
    pub cpu_limit: Option<String>,
    pub cpu_reservation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub driver: String,
    pub options: HashMap<String, String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let mut options = HashMap::new();
        options.insert("max-size".to_string(), "10m".to_string());
        options.insert("max-file".to_string(), "3".to_string());
        Self {
            driver: "json-file".to_string(),
            options,
        }
    }
}

#[async_trait]
pub trait Service: Send + Sync {
    fn name(&self) -> &'static str;
    fn image(&self) -> &'static str;

    /// Generates configuration files (safe to run without side effects on system services)
    async fn configure(&self, _runtime: &dyn SystemPort, _hw: &HardwareInfo, _secrets: &Secrets) -> Result<()> { Ok(()) }

    /// Performs system initialization (e.g., stopping conflicting services). May require root.
    async fn initialize(&self, _runtime: &dyn SystemPort, _hw: &HardwareInfo, _secrets: &Secrets) -> Result<()> { Ok(()) }

    fn ports(&self) -> Vec<String> { vec![] }
    fn env_vars(&self, _hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> { HashMap::new() }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> { vec![] }
    fn networks(&self) -> Vec<String> { vec!["aetheris_net".to_string()] }
    fn devices(&self, _hw: &HardwareInfo) -> Vec<String> { vec![] }
    fn healthcheck(&self) -> Option<String> { None }
    fn depends_on(&self) -> Vec<String> { vec![] }
    fn security_opts(&self) -> Vec<String> { vec![] }
    fn labels(&self) -> HashMap<String, String> { HashMap::new() }
    fn cap_add(&self) -> Vec<String> { vec![] }
    fn sysctls(&self) -> Vec<String> { vec![] }

    /// Returns resource limits/reservations based on hardware
    fn resources(&self, _hw: &HardwareInfo) -> Option<ResourceConfig> { None }

    /// Returns logging configuration
    fn logging(&self) -> LoggingConfig { LoggingConfig::default() }
}

pub fn get_all_services() -> &'static [&'static dyn Service] {
    &[
        &media::PlexService,
        &media::TautulliService,
        &media::OverseerrService,
        &media::JellyfinService,
        &media::JellyseerrService,
        &media::AudiobookshelfService,
        &arr::SonarrService,
        &arr::RadarrService,
        &arr::ProwlarrService,
        &arr::JackettService,
        &arr::BazarrService,
        &download::QBittorrentService,
        &infra::MariaDBService,
        &infra::RedisService,
        &infra::NginxProxyService,
        &infra::DNSCryptService,
        &infra::WireguardService,
        &infra::PortainerService,
        &infra::NetdataService,
        &infra::HomepageService,
        &infra::UptimeKumaService,
        &apps::VaultwardenService,
        &apps::FilebrowserService,
        &apps::YourlsService,
        &apps::GLPIService,
        &apps::GiteaService,
        &apps::RoundcubeService,
        &apps::NextcloudService,
        &apps::MailService,
        &apps::SyncthingService,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_registry() {
        let services = get_all_services();
        assert!(!services.is_empty());

        let names: Vec<&str> = services.iter().map(|s| s.name()).collect();
        assert!(names.contains(&"plex"));
        assert!(names.contains(&"sonarr"));
        assert!(names.contains(&"mariadb"));
    }

}
