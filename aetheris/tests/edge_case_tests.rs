use aetheris_core::core::hardware::{HardwareInfo, HardwareProfile};
use aetheris_core::core::secrets::Secrets;
use aetheris_core::core::config::Config;
use aetheris_core::core::users::{UserManager, Role};
use aetheris_core::adapters::mock::MockRuntime;
use aetheris_core::build_compose_structure;

// ============================================================================
// UNICODE AND SPECIAL CHARACTER TESTS
// ============================================================================

#[tokio::test]
async fn test_unicode_username_rejection() {
    let mut manager = UserManager::default();
    let runtime = MockRuntime::default();

    // Should reject unicode characters
    assert!(manager.add_user(&runtime, "user™", "password", Role::Observer, None).await.is_err());
    assert!(manager.add_user(&runtime, "用户", "password", Role::Observer, None).await.is_err());
    assert!(manager.add_user(&runtime, "user🎉", "password", Role::Observer, None).await.is_err());
}

#[tokio::test]
async fn test_special_chars_in_username() {
    let mut manager = UserManager::default();
    let runtime = MockRuntime::default();

    // Should accept valid special chars
    assert!(manager.add_user(&runtime, "user_name", "password", Role::Observer, None).await.is_ok());
    assert!(manager.add_user(&runtime, "user-name", "password", Role::Observer, None).await.is_ok());

    // Should reject invalid special chars
    assert!(manager.add_user(&runtime, "user name", "password", Role::Observer, None).await.is_err());
    assert!(manager.add_user(&runtime, "user@name", "password", Role::Observer, None).await.is_err());
    assert!(manager.add_user(&runtime, "user/name", "password", Role::Observer, None).await.is_err());
}

#[tokio::test]
async fn test_extremely_long_password() {
    let mut manager = UserManager::default();
    let runtime = MockRuntime::default();

    // Test 1KB password
    let long_password = "a".repeat(1024);
    assert!(manager.add_user(&runtime, "testuser", &long_password, Role::Observer, None).await.is_ok());

    // Verify it works
    assert!(manager.verify("testuser", &long_password).is_some());

    // Test 10KB password
    let very_long_password = "b".repeat(10240);
    let result = manager.add_user(&runtime, "testuser2", &very_long_password, Role::Observer, None).await;
    if result.is_ok() {
        assert!(manager.verify("testuser2", &very_long_password).is_some());
    }
}

// ============================================================================
// HARDWARE PROFILE EDGE CASES
// ============================================================================

#[test]
fn test_hardware_profile_boundary_conditions() {
    assert_eq!(HardwareInfo::evaluate_profile(4, 3, 2), HardwareProfile::Standard);
    assert_eq!(HardwareInfo::evaluate_profile(3, 3, 2), HardwareProfile::Low);
    assert_eq!(HardwareInfo::evaluate_profile(16, 8, 0), HardwareProfile::Standard);
    assert_eq!(HardwareInfo::evaluate_profile(17, 8, 0), HardwareProfile::High);
}

// ============================================================================
// COMPOSE GENERATION EDGE CASES
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
    let services = aetheris_core::services::get_all_services();
    for service in services {
        config.disable_service(service.name());
    }

    let secrets = Secrets::default();
    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();

    assert_eq!(compose.services.len(), 0);
    assert!(compose.networks.contains_key("aetheris_net"));
}

// ============================================================================
// USER MANAGEMENT EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_user_deletion_protections() {
    let mut manager = UserManager::default();
    let runtime = MockRuntime::default();
    manager.add_user(&runtime, "admin", "password", Role::Admin, None).await.unwrap();

    // Cannot delete last admin
    assert!(manager.delete_user(&runtime, "admin").await.is_err());

    // Add another admin
    manager.add_user(&runtime, "admin2", "password", Role::Admin, None).await.unwrap();

    // Now can delete first admin
    assert!(manager.delete_user(&runtime, "admin").await.is_ok());

    // But still can't delete last one
    assert!(manager.delete_user(&runtime, "admin2").await.is_err());
}

#[tokio::test]
async fn test_user_password_update() {
    let mut manager = UserManager::default();
    let runtime = MockRuntime::default();
    manager.add_user(&runtime, "testuser", "initial_pass", Role::Observer, None).await.unwrap();

    assert!(manager.verify("testuser", "initial_pass").is_some());

    manager.update_password(&runtime, "testuser", "new_pass").await.unwrap();

    assert!(manager.verify("testuser", "initial_pass").is_none());
    assert!(manager.verify("testuser", "new_pass").is_some());
}

#[tokio::test]
async fn test_user_listing() {
    let mut manager = UserManager::default();
    let runtime = MockRuntime::default();
    manager.add_user(&runtime, "user1", "pass", Role::Observer, None).await.unwrap();
    manager.add_user(&runtime, "user2", "pass", Role::Admin, Some(10)).await.unwrap();

    let users = manager.list_users();
    assert_eq!(users.len(), 2);

    let u1 = users.iter().find(|u| u.username == "user1").unwrap();
    assert_eq!(u1.role, Role::Observer);
    assert!(u1.quota_gb.is_none());

    let u2 = users.iter().find(|u| u.username == "user2").unwrap();
    assert_eq!(u2.role, Role::Admin);
    assert_eq!(u2.quota_gb, Some(10));
}

#[tokio::test]
async fn test_user_verify_async() {
    let mut manager = UserManager::default();
    let runtime = MockRuntime::default();
    manager.add_user(&runtime, "testuser", "correct_pass", Role::Observer, None).await.unwrap();

    // Verify correct password returns Some(User)
    let user = manager.verify_async("testuser", "correct_pass").await;
    assert!(user.is_some());
    assert_eq!(user.unwrap().username, "testuser");

    // Verify incorrect password returns None
    let bad_user = manager.verify_async("testuser", "wrong_pass").await;
    assert!(bad_user.is_none());

    // Verify non-existent user returns None
    let no_user = manager.verify_async("missinguser", "correct_pass").await;
    assert!(no_user.is_none());
}
