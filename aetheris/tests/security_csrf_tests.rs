use std::fs;
use std::time::Duration;
use aetheris_core::core::users::UserManager;
use aetheris_core::adapters::mock::MockRuntime;
use tokio::time::sleep;

const PORT: u16 = 3003;
const BASE_URL: &str = "http://127.0.0.1:3003";

#[tokio::test]
async fn test_csrf_protection_and_flow() {
    // 1. Setup isolated environment
    let temp_dir = std::env::temp_dir().join("aetheris_csrf_test_v2");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir_all(&temp_dir).unwrap();

    std::env::set_var("AETHERIS_HOME", &temp_dir);

    // 2. Create Admin User
    let _manager = UserManager::default();
    let _runtime = MockRuntime::default();

    // Hash for "password"
    let hash = "$2y$12$eX6lqHJetswerXHzelRy.e9SVJ8ebcwich0tJWCh8tIy9LF.1hV.S";

    let users_yaml_content = format!(
        r#"
users:
  admin:
    username: admin
    password_hash: "{}"
    role: Admin
    quota_gb: null
"#, hash);

    fs::write(temp_dir.join("users.yaml"), users_yaml_content).unwrap();

    // 3. Start Server in Background
    let server_handle = tokio::spawn(async move {
        aetheris_core::interface::web::start_server(PORT).await.unwrap();
    });

    sleep(Duration::from_secs(2)).await;

    let cookie_file = temp_dir.join("cookies.txt");

    // 4. Login
    let status = std::process::Command::new("curl")
        .arg("-c").arg(&cookie_file)
        .arg("-d").arg("username=admin")
        .arg("-d").arg("password=password")
        .arg("-L")
        .arg(format!("{}/login", BASE_URL))
        .output()
        .expect("Failed to execute curl for login");

    assert!(status.status.success());

    // 5. TEST CASE 1: Attack without token (should FAIL with 403)
    let attack_status = std::process::Command::new("curl")
        .arg("-b").arg(&cookie_file)
        .arg("-d").arg("username=hacker_user")
        .arg("-d").arg("password=hacked")
        .arg("-d").arg("role=Admin")
        .arg("-d").arg("quota=0")
        .arg("-w").arg("%{http_code}")
        .arg("-o").arg("/dev/null")
        .arg("-s")
        .arg(format!("{}/users/add", BASE_URL))
        .output()
        .expect("Failed to execute curl for attack");

    let http_code = String::from_utf8_lossy(&attack_status.stdout);

    // NOTE: If using Form extractor, missing field might cause 422 Unprocessable Entity or 400 Bad Request
    // depending on Axum version/config. Or if we added the field to struct but didn't send it,
    // it's a deserialization error (422/400).
    // If we send WRONG token, it's 403.
    // Let's check for any failure code (4xx)
    println!("Attack response code (No Token): {}", http_code);
    assert!(http_code.starts_with("4"), "Expected 4xx error for missing token, got {}", http_code);

    // 6. TEST CASE 2: Attack with WRONG token (should FAIL with 403)
     let attack_wrong_token = std::process::Command::new("curl")
        .arg("-b").arg(&cookie_file)
        .arg("-d").arg("username=hacker_user_2")
        .arg("-d").arg("password=hacked")
        .arg("-d").arg("role=Admin")
        .arg("-d").arg("quota=0")
        .arg("-d").arg("csrf_token=invalidtoken123")
        .arg("-w").arg("%{http_code}")
        .arg("-o").arg("/dev/null")
        .arg("-s")
        .arg(format!("{}/users/add", BASE_URL))
        .output()
        .expect("Failed to execute curl for attack");

    let http_code_wrong = String::from_utf8_lossy(&attack_wrong_token.stdout);
    println!("Attack response code (Wrong Token): {}", http_code_wrong);
    assert_eq!(http_code_wrong.trim(), "403");

    // 7. TEST CASE 3: Valid Request (Happy Path)
    // First, fetch the form page to get the token
    let form_page = std::process::Command::new("curl")
        .arg("-b").arg(&cookie_file)
        .arg("-s")
        .arg(format!("{}/users", BASE_URL))
        .output()
        .expect("Failed to fetch users page");

    let html = String::from_utf8_lossy(&form_page.stdout);

    // Extract token
    // <input type="hidden" name="csrf_token" value="...">
    let token_part = html.split("name=\"csrf_token\" value=\"").nth(1)
        .expect("Could not find csrf token in HTML");
    let token = token_part.split("\"").next().expect("Could not parse token");

    println!("Extracted CSRF Token: {}", token);

    // Send valid request
    let valid_req = std::process::Command::new("curl")
        .arg("-b").arg(&cookie_file)
        .arg("-d").arg("username=valid_user")
        .arg("-d").arg("password=securepass")
        .arg("-d").arg("role=Observer")
        .arg("-d").arg("quota=0")
        .arg("-d").arg(format!("csrf_token={}", token))
        .arg("-w").arg("%{http_code}")
        .arg("-o").arg("/dev/null")
        .arg("-s")
        .arg(format!("{}/users/add", BASE_URL))
        .output()
        .expect("Failed to execute valid request");

    let http_code_valid = String::from_utf8_lossy(&valid_req.stdout);
    println!("Valid request response code: {}", http_code_valid);

    // Should be 303 Redirect or 200 OK (if following redirects manually, but here we check immediate response)
    // The handler returns Redirect, which is 303.
    assert_eq!(http_code_valid.trim(), "303");

    // Verify persistence
    let content = fs::read_to_string(temp_dir.join("users.yaml")).unwrap();
    assert!(content.contains("valid_user"), "Valid user should be added");
    assert!(!content.contains("hacker_user"), "Hacker user should NOT be added");

    server_handle.abort();
    // fs::remove_dir_all(temp_dir).unwrap();
}
