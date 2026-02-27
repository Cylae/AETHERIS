use aetheris_core::interface::web;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;
use std::net::TcpListener;
use std::path::PathBuf;
use std::fs;

// Helper to find a free port and prevent re-use
fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

async fn spawn_server(port: u16) {
    // Spawn server in background
    tokio::spawn(async move {
        // Run with env logger suppressed to avoid noise
        if let Err(e) = web::start_server(port).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give it a moment to start (longer for CI/parallel tests)
    sleep(Duration::from_millis(2000)).await;
}

#[tokio::test]
async fn test_secure_sessions() {
    // Use a temporary directory for the test to avoid messing with real config
    let temp_dir = std::env::temp_dir().join(format!("aetheris_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    // Set AETHERIS_HOME to the temp dir so config.yaml and users.yaml are looked for there
    // This isolates the test environment
    unsafe { std::env::set_var("AETHERIS_HOME", temp_dir.to_str().unwrap()); }

    // --- CASE 1: Default (Insecure) ---
    println!("--- Testing Case 1: Default (Insecure) ---");
    // Ensure env var is NOT set
    unsafe { std::env::remove_var("AETHERIS_SECURE_SESSIONS"); }

    // Clean start in temp dir (users.yaml will be created by server)
    let users_path = temp_dir.join("users.yaml");
    if users_path.exists() {
        fs::remove_file(&users_path).unwrap();
    }

    let port1 = get_free_port();
    println!("Starting server on port {}", port1);
    spawn_server(port1).await;

    let client = Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let params = [("username", "admin"), ("password", "admin")];
    let resp = client.post(format!("http://127.0.0.1:{}/login", port1))
        .form(&params)
        .send()
        .await
        .expect("Failed to send login request");

    let cookies = resp.headers().get_all("set-cookie");
    let mut found_session = false;
    for cookie in cookies {
        let cookie_str = cookie.to_str().unwrap();
        if cookie_str.contains("id=") {
            found_session = true;
            println!("Found cookie: {}", cookie_str);
            assert!(!cookie_str.contains("Secure"), "Cookie should NOT be Secure by default");
            assert!(cookie_str.contains("SameSite=Lax"), "Cookie SHOULD be SameSite=Lax");
        }
    }
    assert!(found_session, "Session cookie not found! Login likely failed.");

    // --- CASE 2: Secure via Env ---
    println!("--- Testing Case 2: Secure via Env ---");
    // Set env var
    unsafe { std::env::set_var("AETHERIS_SECURE_SESSIONS", "true"); }

    // Clean users again to ensure fresh start (optional, but good for isolation)
    if users_path.exists() {
        fs::remove_file(&users_path).unwrap();
    }

    let port2 = get_free_port();
    println!("Starting server on port {}", port2);
    spawn_server(port2).await;

    let resp2 = client.post(format!("http://127.0.0.1:{}/login", port2))
        .form(&params)
        .send()
        .await
        .expect("Failed to send login request");

    let cookies2 = resp2.headers().get_all("set-cookie");
    let mut found_session2 = false;
    for cookie in cookies2 {
        let cookie_str = cookie.to_str().unwrap();
        if cookie_str.contains("id=") {
            found_session2 = true;
            println!("Found cookie: {}", cookie_str);
            assert!(cookie_str.contains("Secure"), "Cookie SHOULD be Secure when configured");
            assert!(cookie_str.contains("SameSite=Lax"), "Cookie SHOULD be SameSite=Lax");
        }
    }
    assert!(found_session2, "Session cookie not found! Login likely failed.");

    // Cleanup
    unsafe { std::env::remove_var("AETHERIS_SECURE_SESSIONS"); }
    unsafe { std::env::remove_var("AETHERIS_HOME"); }
    fs::remove_dir_all(temp_dir).unwrap_or_else(|e| eprintln!("Failed to cleanup temp dir: {}", e));
}
