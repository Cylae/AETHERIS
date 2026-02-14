use aetheris_core::domain::orchestrator::AetherisOrchestrator;
use aetheris_core::ports::SystemPort;
use aetheris_core::core::hardware::HardwareInfo;
use aetheris_core::core::secrets::Secrets;
use aetheris_core::core::config::Config;
use aetheris_core::adapters::mock::MockRuntime;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::time::{Duration, Instant};

struct DelayedSystem {
    delay: Duration,
}

#[async_trait]
impl SystemPort for DelayedSystem {
    fn check_root(&self) -> Result<()> { Ok(()) }
    async fn install_packages(&self, _pkgs: Vec<String>) -> Result<()> { Ok(()) }
    async fn apply_optimizations(&self, _ram_gb: u64) -> Result<()> { Ok(()) }
    async fn configure_firewall(&self) -> Result<()> { Ok(()) }
    async fn create_user(&self, _u: &str, _p: &str) -> Result<()> { Ok(()) }
    async fn delete_user(&self, _u: &str) -> Result<()> { Ok(()) }
    async fn set_password(&self, _u: &str, _p: &str) -> Result<()> { Ok(()) }
    async fn set_quota(&self, _u: &str, _q: u64) -> Result<()> { Ok(()) }
    fn get_uid(&self, _u: &str) -> Result<u32> { Ok(1000) }

    fn write_file(&self, _path: &Path, _content: &str) -> Result<()> {
        // Use block_in_place to simulate blocking I/O without stalling the executor
        // for other tasks if running on a multi-threaded runtime.
        let delay = self.delay;
        let _ = tokio::task::block_in_place(move || {
            std::thread::sleep(delay);
        });
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> Result<()> {
        let delay = self.delay;
        let _ = tokio::task::block_in_place(move || {
            std::thread::sleep(delay);
        });
        Ok(())
    }

    fn file_exists(&self, _path: &Path) -> bool { false }

    async fn stop_and_disable_service(&self, _name: &str) -> Result<()> {
        tokio::time::sleep(self.delay).await;
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_configure_services_performance() {
    let delay = Duration::from_millis(50);
    let system = DelayedSystem { delay };
    let runtime = MockRuntime::default();
    let orchestrator = AetherisOrchestrator::new(Box::new(runtime), Box::new(system));

    let hw = HardwareInfo {
        profile: aetheris_core::core::hardware::HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 4,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };
    let secrets = Secrets::default();
    let config = Config::default();

    let start = Instant::now();
    orchestrator.configure_services(&hw, &secrets, &config).await.unwrap();
    let duration = start.elapsed();

    println!("Configure services took: {:?}", duration);
    // MariaDB does 3-4 calls (create_dir_all, write_file x2)
    // Nextcloud does 2 calls (create_dir_all, write_file)
    // Total should be at least 5 * 50ms = 250ms sequentially.
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_initialize_services_performance() {
    let delay = Duration::from_millis(50);
    let system = DelayedSystem { delay };
    let runtime = MockRuntime::default();
    let orchestrator = AetherisOrchestrator::new(Box::new(runtime), Box::new(system));

    let hw = HardwareInfo {
        profile: aetheris_core::core::hardware::HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 4,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };
    let secrets = Secrets::default();
    let config = Config::default();

    let start = Instant::now();
    orchestrator.initialize_services(&hw, &secrets, &config).await.unwrap();
    let duration = start.elapsed();

    println!("Initialize services took: {:?}", duration);
    // NginxProxy does 3 calls to stop_and_disable_service
    // Total should be at least 3 * 50ms = 150ms sequentially.
}
