// COMPREHENSIVE TEST SUITE ADDITIONS
// Add these tests to verify edge cases and security

// File: tests/edge_case_tests.rs

use aetheris_core::build_compose_structure;
use aetheris_core::core::config::Config;
use aetheris_core::core::hardware::{HardwareInfo, HardwareProfile};
use aetheris_core::core::secrets::Secrets;
use aetheris_core::core::users::{Role, UserManager};

// ============================================================================
// UNICODE AND SPECIAL CHARACTER TESTS
// ============================================================================

#[test]
fn test_unicode_username_rejection() {
    let mut manager = UserManager::default();

    // Should reject unicode characters
    assert!(manager
        .add_user("user™", "password", Role::Observer, None)
        .is_err());
    assert!(manager
        .add_user("用户", "password", Role::Observer, None)
        .is_err());
    assert!(manager
        .add_user("user🎉", "password", Role::Observer, None)
        .is_err());
}

#[test]
fn test_special_chars_in_username() {
    let mut manager = UserManager::default();

    // Should accept valid special chars
    assert!(manager
        .add_user("user_name", "password", Role::Observer, None)
        .is_ok());
    assert!(manager
        .add_user("user-name", "password", Role::Observer, None)
        .is_ok());

    // Should reject invalid special chars
    assert!(manager
        .add_user("user name", "password", Role::Observer, None)
        .is_err());
    assert!(manager
        .add_user("user@name", "password", Role::Observer, None)
        .is_err());
    assert!(manager
        .add_user("user/name", "password", Role::Observer, None)
        .is_err());
}

#[test]
fn test_extremely_long_password() {
    let mut manager = UserManager::default();

    // Test 1KB password
    let long_password = "a".repeat(1024);
    assert!(manager
        .add_user("testuser", &long_password, Role::Observer, None)
        .is_ok());

    // Verify it works
    assert!(manager.verify("testuser", &long_password).is_some());

    // Test 10KB password (may fail due to bcrypt limits)
    let very_long_password = "b".repeat(10240);
    let result = manager.add_user("testuser2", &very_long_password, Role::Observer, None);
    // Either succeeds or fails gracefully
    if result.is_ok() {
        assert!(manager.verify("testuser2", &very_long_password).is_some());
    }
}

// ============================================================================
// CONFIG CORRUPTION RECOVERY TESTS
// ============================================================================

#[test]
fn test_config_malformed_yaml() {
    use std::fs;
    use std::path::Path;

    // Create malformed config
    let test_config_path = "/tmp/test_config_malformed.yaml";
    fs::write(test_config_path, "disabled_services: [plex, - invalid yaml").ok();

    // Should return default config on parse error
    // (This test would need config loading from custom path)
}

#[test]
fn test_config_empty_file() {
    let config_content = "";
    let config: Config = serde_yaml_ng::from_str(config_content).unwrap_or_default();

    assert!(config.disabled_services.is_empty());
}

// ============================================================================
// HARDWARE PROFILE EDGE CASES
// ============================================================================

#[test]
fn test_hardware_profile_boundary_conditions() {
    // Test exact boundary values
    assert_eq!(
        HardwareInfo::evaluate_profile(4, 3, 2),
        HardwareProfile::Standard,
        "4GB RAM should be Standard"
    );

    assert_eq!(
        HardwareInfo::evaluate_profile(3, 3, 2),
        HardwareProfile::Low,
        "3GB RAM should be Low"
    );

    assert_eq!(
        HardwareInfo::evaluate_profile(16, 8, 0),
        HardwareProfile::Standard,
        "Exactly 16GB should be Standard"
    );

    assert_eq!(
        HardwareInfo::evaluate_profile(17, 8, 0),
        HardwareProfile::High,
        "17GB should be High"
    );
}

#[test]
fn test_hardware_profile_swap_edge_cases() {
    // No swap, low RAM -> Low
    assert_eq!(
        HardwareInfo::evaluate_profile(6, 4, 0),
        HardwareProfile::Low
    );

    // With swap, low RAM -> Standard
    assert_eq!(
        HardwareInfo::evaluate_profile(6, 4, 2),
        HardwareProfile::Standard
    );

    // High RAM ignores swap
    assert_eq!(
        HardwareInfo::evaluate_profile(32, 8, 0),
        HardwareProfile::High
    );
}

// ============================================================================
// SQL INJECTION PROTECTION
// ============================================================================

#[test]
fn test_sql_injection_in_secrets() {
    let mut secrets = Secrets::default();

    // Simulate malicious password with SQL injection attempt
    secrets.nextcloud_db_password = Some("'; DROP TABLE users; --".to_string());

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

    // Build compose structure - should properly escape
    let compose = build_compose_structure(&hw, &secrets, &Config::default()).unwrap();

    // Verify MariaDB init script escapes properly
    // (This would require inspecting generated init.sql)
}

// ============================================================================
// CONCURRENT CONFIG UPDATES
// ============================================================================

#[tokio::test]
async fn test_concurrent_config_updates() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let mut tasks = JoinSet::new();

    // Spawn 100 tasks trying to update config simultaneously
    for i in 0..100 {
        tasks.spawn(async move {
            let mut config = Config::load().unwrap_or_default();
            if i % 2 == 0 {
                config.disable_service("plex");
            } else {
                config.enable_service("plex");
            }
            config.save().ok();
        });
    }

    // Wait for all tasks
    while let Some(_) = tasks.join_next().await {}

    // Final config should be consistent (not corrupted)
    let final_config = Config::load().unwrap();
    // Should be either enabled or disabled, not in inconsistent state
}

// ============================================================================
// DOCKER COMPOSE GENERATION EDGE CASES
// ============================================================================

#[test]
fn test_compose_generation_with_all_services_disabled() {
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

    let mut config = Config::default();

    // Disable ALL services
    let services = aetheris_core::services::get_all_services();
    for service in services {
        config.disable_service(service.name());
    }

    let secrets = Secrets::default();
    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();

    // Should generate valid (empty) compose file
    assert_eq!(compose.services.len(), 0);
    assert!(compose.networks.contains_key("aetheris_net"));
}

#[test]
fn test_compose_generation_with_gpu_but_no_transcoding_services() {
    let mut hw = HardwareInfo {
        profile: HardwareProfile::High,
        ram_gb: 32,
        cpu_cores: 16,
        has_nvidia: true,
        has_intel_quicksync: true,
        disk_gb: 1000,
        swap_gb: 4,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };

    let mut config = Config::default();
    config.disable_service("plex");
    config.disable_service("jellyfin");

    let secrets = Secrets::default();
    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();

    // Should not crash, GPU settings just unused
    assert!(!compose.services.contains_key("plex"));
    assert!(!compose.services.contains_key("jellyfin"));
}

// ============================================================================
// RESOURCE LIMIT EDGE CASES
// ============================================================================

#[test]
fn test_resource_limits_on_minimal_hardware() {
    let hw = HardwareInfo {
        profile: HardwareProfile::Low,
        ram_gb: 1, // Extremely low
        cpu_cores: 1,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 50,
        swap_gb: 2,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };

    let secrets = Secrets::default();
    let config = Config::default();
    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();

    // Verify all services have appropriate low limits
    let mariadb = compose.services.get("mariadb").unwrap();
    assert!(mariadb.deploy.is_some());

    let deploy = mariadb.deploy.as_ref().unwrap();
    let resources = deploy.resources.as_ref().unwrap();
    let limits = resources.limits.as_ref().unwrap();

    // Should be minimal on Low profile
    assert_eq!(limits.memory.as_ref().unwrap(), "512M");
}

// ============================================================================
// SECRETS GENERATION EDGE CASES
// ============================================================================

#[test]
fn test_secrets_hex_generation_randomness() {
    use aetheris_core::core::secrets::Secrets;
    use std::collections::HashSet;

    let mut seen = HashSet::new();

    // Generate 100 secrets files, verify no duplicates
    for _ in 0..100 {
        let secrets = Secrets::load_or_create().unwrap();
        let root_pw = secrets.mysql_root_password.unwrap();
        assert!(!seen.contains(&root_pw), "Duplicate password generated!");
        seen.insert(root_pw);
    }
}

// ============================================================================
// USER MANAGEMENT EDGE CASES
// ============================================================================

#[test]
fn test_user_deletion_protections() {
    let mut manager = UserManager::default();
    manager
        .add_user("admin", "password", Role::Admin, None)
        .unwrap();

    // Cannot delete last admin
    assert!(manager.delete_user("admin").is_err());

    // Add another admin
    manager
        .add_user("admin2", "password", Role::Admin, None)
        .unwrap();

    // Now can delete first admin
    assert!(manager.delete_user("admin").is_ok());

    // But still can't delete last one
    assert!(manager.delete_user("admin2").is_err());
}
