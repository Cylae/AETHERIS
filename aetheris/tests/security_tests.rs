use aetheris_core::ports::SystemPort;
use aetheris_core::core::hardware::{HardwareInfo, HardwareProfile};
use aetheris_core::core::secrets::Secrets;
use aetheris_core::services::apps::NextcloudService;
use aetheris_core::services::Service;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;

// Mock System Implementation that captures writes
struct SpySystem {
    files: Arc<Mutex<HashMap<PathBuf, String>>>,
}

impl SpySystem {
    fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SystemPort for SpySystem {
    fn check_root(&self) -> Result<()> { Ok(()) }
    async fn install_packages(&self, _pkgs: Vec<String>) -> Result<()> { Ok(()) }
    async fn apply_optimizations(&self, _ram_gb: u64) -> Result<()> { Ok(()) }
    async fn configure_firewall(&self) -> Result<()> { Ok(()) }
    async fn create_user(&self, _u: &str, _p: &str) -> Result<()> { Ok(()) }
    async fn delete_user(&self, _u: &str) -> Result<()> { Ok(()) }
    async fn set_password(&self, _u: &str, _p: &str) -> Result<()> { Ok(()) }
    async fn set_quota(&self, _u: &str, _q: u64) -> Result<()> { Ok(()) }
    fn get_uid(&self, _u: &str) -> Result<u32> { Ok(1000) }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        self.files.lock().unwrap().insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> Result<()> { Ok(()) }
    // Important: file_exists must return false so that it tries to write the config
    fn file_exists(&self, _path: &Path) -> bool { false }

    async fn stop_and_disable_service(&self, _name: &str) -> Result<()> { Ok(()) }
}

#[tokio::test]
async fn test_nextcloud_php_injection() {
    let spy = SpySystem::new();
    let service = NextcloudService;

    // Setup secrets with a PHP variable that should NOT be interpolated
    let mut secrets = Secrets::default();
    // In PHP double quotes: "password$variable" -> $variable is replaced.
    // In PHP single quotes: 'password$variable' -> literal string.
    let malicious_password = "password$variable";
    secrets.nextcloud_db_password = Some(malicious_password.to_string());
    secrets.nextcloud_admin_password = Some(malicious_password.to_string());

    let hw = HardwareInfo {
        profile: HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 2,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };

    // Run configure
    service.configure(&spy, &hw, &secrets).await.unwrap();

    // Check the written file
    let files = spy.files.lock().unwrap();
    let config_path = Path::new("./config/nextcloud/autoconfig.php");

    assert!(files.contains_key(config_path), "autoconfig.php was not created");

    let content = files.get(config_path).unwrap();
    println!("Generated content:\n{}", content);

    // With the VULNERABILITY:
    // "dbpass"        => "password$variable",
    // This allows $variable to be executed/interpolated.

    // We expect the fix to use single quotes:
    // 'dbpass'        => 'password$variable',

    // For now, this test asserts the current behavior (double quotes) to confirm reproduction.
    // Wait, usually I should write a test that FAILS if the vulnerability is present if I want to TDD it properly?
    // Or I write a test that passes if the fix is implemented.
    // Let's write the test asserting the SECURE behavior, so it fails now.

    // Note: The key "dbpass" is in double quotes in the template, only the value needs single quotes
    assert!(content.contains("\"dbpass\"        => 'password$variable'"), "Should use single quotes for dbpass VALUE to prevent injection");
    assert!(!content.contains("\"dbpass\"        => \"password$variable\""), "Should NOT use double quotes for dbpass VALUE");
}
