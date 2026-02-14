use aetheris_core::ports::SystemPort;
use aetheris_core::services::{Service, apps::NextcloudService};
use aetheris_core::core::hardware::{HardwareInfo, HardwareProfile};
use aetheris_core::core::secrets::Secrets;
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Default)]
struct SpySystem {
    written_files: Arc<Mutex<HashMap<PathBuf, String>>>,
    created_dirs: Arc<Mutex<Vec<PathBuf>>>,
    existing_files: Arc<Mutex<Vec<PathBuf>>>,
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
        self.written_files.lock().unwrap().insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.created_dirs.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }

    fn file_exists(&self, path: &Path) -> bool {
        self.existing_files.lock().unwrap().contains(&path.to_path_buf())
    }

    async fn stop_and_disable_service(&self, _name: &str) -> Result<()> { Ok(()) }
}

#[tokio::test]
async fn test_nextcloud_php_injection_vulnerability() {
    let system = SpySystem::default();
    let service = NextcloudService;
    let hw = HardwareInfo {
        profile: HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 4,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };

    // Attempting to inject PHP code via variable interpolation or breaking out of quotes
    let malicious_pass = "password$foo".to_string();
    let malicious_admin = "{${system('ls')}}".to_string();

    let secrets = Secrets {
        nextcloud_db_password: Some(malicious_pass.clone()),
        nextcloud_admin_password: Some(malicious_admin.clone()),
        ..Secrets::default()
    };

    service.configure(&system, &hw, &secrets).await.expect("Configure failed");

    let written = system.written_files.lock().unwrap();
    let autoconfig_path = Path::new("./config/nextcloud/autoconfig.php");
    let content = written.get(autoconfig_path).unwrap();

    println!("Generated PHP config:\n{}", content);

    // Verify that single quotes are used for values
    assert!(content.contains("'dbpass'        => 'password$foo'"));
    assert!(content.contains("'adminpass'     => '{${system(\\'ls\\')}}'"));

    drop(written);

    // Test complex escaping
    let complex_secrets = Secrets {
        nextcloud_db_password: Some("a'b\\c".to_string()),
        nextcloud_admin_password: Some("admin' OR '1'='1".to_string()),
        ..Secrets::default()
    };

    service.configure(&system, &hw, &complex_secrets).await.expect("Configure failed");
    let written = system.written_files.lock().unwrap();
    let content = written.get(autoconfig_path).unwrap();

    println!("Complex PHP config:\n{}", content);
    assert!(content.contains("'dbpass'        => 'a\\'b\\\\c'"));
    assert!(content.contains("'adminpass'     => 'admin\\' OR \\'1\\'=\\'1'"));
}
