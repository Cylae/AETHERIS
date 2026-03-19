use axum::{
    extract::{Path, Form, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
    http::StatusCode,
};
use std::fmt::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::services;
use crate::core::config::Config;
use crate::core::users::{UserManager, Role};
use tokio::process::Command;
use log::{info, error, warn};
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
use serde::{Deserialize, Serialize};
use time::Duration;
use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use tokio::sync::RwLock;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone)]
struct SessionUser {
    username: String,
    role: Role,
}

const SESSION_KEY: &str = "user";

struct CachedConfig {
    config: Config,
    last_modified: Option<SystemTime>,
}

struct AppState {
    system: Mutex<System>,
    last_system_refresh: Mutex<SystemTime>,
    config_cache: RwLock<CachedConfig>,
}

type SharedState = Arc<AppState>;

impl AppState {
    async fn get_config(&self) -> Config {
        // Fast path: check metadata
        let current_mtime = tokio::fs::metadata("config.yaml").await
            .and_then(|m| m.modified())
            .ok();

        {
            let cache = self.config_cache.read().await;
            if cache.last_modified == current_mtime {
                return cache.config.clone();
            }
        }

        // Slow path: reload
        let mut cache = self.config_cache.write().await;

        // Re-check mtime under write lock to avoid race
        let current_mtime_2 = tokio::fs::metadata("config.yaml").await
            .and_then(|m| m.modified())
            .ok();

        if cache.last_modified == current_mtime_2 {
            return cache.config.clone();
        }

        if let Ok(cfg) = Config::load_async().await {
            cache.config = cfg;
            cache.last_modified = current_mtime_2;
        }

        cache.config.clone()
    }
}

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    // Session setup
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Localhost/LAN, http usually
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    // Initialize System once
    let mut sys = System::new_all();
    sys.refresh_all();

    let initial_config = Config::load().unwrap_or_default();
    let initial_mtime = std::fs::metadata("config.yaml").ok().and_then(|m| m.modified().ok());

    let app_state = Arc::new(AppState {
        system: Mutex::new(sys),
        last_system_refresh: Mutex::new(SystemTime::now()),
        config_cache: RwLock::new(CachedConfig {
            config: initial_config,
            last_modified: initial_mtime,
        }),
    });

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/users", get(users_page))
        .route("/users/add", post(add_user_handler))
        .route("/users/delete/:username", post(delete_user_handler))
        .route("/api/services/:name/enable", post(enable_service))
        .route("/api/services/:name/disable", post(disable_service))
        .route("/logout", post(logout))
        .route("/login", get(login_page).post(login_handler))
        .layer(session_layer)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting Web UI on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn login_page(session: Session) -> impl IntoResponse {
    if let Some(_user) = session.get::<SessionUser>(SESSION_KEY).await.unwrap_or(None) {
        return Redirect::to("/").into_response();
    }

    let html = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
        <meta name="theme-color" content="\#0a0a0a">
        <meta name="apple-mobile-web-app-capable" content="yes">
        <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
        <title>Login - AETHERIS</title>
        <script>
            function getTheme() { return localStorage.getItem('theme') || 'dark'; }
            function setTheme(theme) {
                document.documentElement.setAttribute('data-theme', theme);
                localStorage.setItem('theme', theme);
            }
            function toggleTheme() {
                setTheme(getTheme() === 'dark' ? 'light' : 'dark');
            }
            setTheme(getTheme());
        </script>
        <style>
            :root {
                --bg-color: #ffffff;
                --text-color: #111111;
                --card-bg: #f9f9f9;
                --border-color: #cccccc;
                --input-bg: #ffffff;
                --input-text: #111111;
                --primary: #00f0ff;
                --secondary: #8a2be2;
                --neon-glow: rgba(0, 240, 255, 0.4);
                --btn-text: #ffffff;
            }
            [data-theme="dark"] {
                --bg-color: #0a0a0a;
                --text-color: #e0e0e0;
                --card-bg: #141414;
                --border-color: #333333;
                --input-bg: #1a1a1a;
                --input-text: #e0e0e0;
                --neon-glow: rgba(0, 240, 255, 0.7);
            }
            body {
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                display: flex;
                justify-content: center;
                align-items: center;
                height: 100vh;
                margin: 0;
                background-color: var(--bg-color);
                color: var(--text-color);
                background-image: linear-gradient(var(--border-color) 1px, transparent 1px), linear-gradient(90deg, var(--border-color) 1px, transparent 1px);
                background-size: 40px 40px;
                background-position: center center;
                transition: background-color 0.3s, color 0.3s;
            }
            [data-theme="dark"] body {
                background-image: linear-gradient(rgba(0, 240, 255, 0.05) 1px, transparent 1px), linear-gradient(90deg, rgba(0, 240, 255, 0.05) 1px, transparent 1px);
            }
            .login-box {
                background: var(--card-bg);
                padding: 30px;
                border-radius: 12px;
                box-shadow: 0 0 20px var(--neon-glow);
                width: 320px;
                border: 1px solid var(--border-color);
                position: relative;
            }
            [data-theme="dark"] .login-box {
                border-image: linear-gradient(45deg, var(--primary), var(--secondary)) 1;
                border-width: 2px;
                border-style: solid;
            }
            h2 {
                text-align: center;
                margin-top: 0;
                color: var(--primary);
                text-shadow: 0 0 10px var(--neon-glow);
                letter-spacing: 2px;
                text-transform: uppercase;
            }
            input {
                width: 100%;
                padding: 12px;
                margin: 10px 0;
                box-sizing: border-box;
                background: var(--input-bg);
                color: var(--input-text);
                border: 1px solid var(--border-color);
                border-radius: 4px;
                outline: none;
                transition: box-shadow 0.3s, border-color 0.3s;
            }
            input:focus {
                border-color: var(--primary);
                box-shadow: 0 0 8px var(--neon-glow);
            }
            .btn {
                width: 100%;
                padding: 12px;
                background: linear-gradient(90deg, var(--secondary), var(--primary));
                color: var(--btn-text);
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-weight: bold;
                text-transform: uppercase;
                letter-spacing: 1px;
                transition: box-shadow 0.3s, transform 0.1s;
            }
            .btn:hover {
                box-shadow: 0 0 15px var(--neon-glow);
            }
            .btn:active {
                transform: scale(0.98);
            }
            .theme-toggle-container {
                text-align: center;
                margin-top: 20px;
            }
            .btn-toggle {
                background: transparent;
                border: 1px solid var(--primary);
                color: var(--primary);
                padding: 8px 16px;
                cursor: pointer;
                border-radius: 4px;
                text-transform: uppercase;
                font-size: 0.8em;
                letter-spacing: 1px;
                transition: all 0.3s;
            }
            .btn-toggle:hover {
                background: var(--primary);
                color: var(--bg-color);
                box-shadow: 0 0 10px var(--neon-glow);
            }
        </style>
    </head>
    <body>
        <div class="login-box">
            <h2>Login</h2>
            <form method="POST" action="/login">
                <input type="text" name="username" placeholder="Username" required>
                <input type="password" name="password" placeholder="Password" required>
                <button type="submit" class="btn">Login</button>
            </form>
            <div class="theme-toggle-container">
                <button type="button" class="btn-toggle" onclick="toggleTheme()">Toggle Theme</button>
            </div>
        </div>
    </body>
    </html>
    "#;
    Html(html).into_response()
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

async fn login_handler(session: Session, Form(payload): Form<LoginPayload>) -> impl IntoResponse {
    // Reload users on login attempt to get fresh data
    let user_manager = UserManager::load_async().await.unwrap_or_default();

    if let Some(user) = user_manager.verify_async(&payload.username, &payload.password).await {
        let session_user = SessionUser {
            username: user.username,
            role: user.role,
        };
        if let Err(e) = session.insert(SESSION_KEY, session_user).await {
            error!("Failed to insert session: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session").into_response();
        }
        Redirect::to("/").into_response()
    } else {
        // Simple error handling: redirect back to login
        warn!("Failed login attempt for user: {}", payload.username);
        Redirect::to("/login").into_response()
    }
}

async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.ok();
    Redirect::to("/login")
}

// Helper for HTML escaping
fn escape_html(s: &str) -> String {
    let mut output = String::with_capacity(s.len() + 10);
    for c in s.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(c),
        }
    }
    output
}

// Helper for common HTML head
fn html_head(title: &str) -> String {
    format!(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
        <meta name="theme-color" content="\#0a0a0a">
        <meta name="apple-mobile-web-app-capable" content="yes">
        <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
        <title>{}</title>
        <script>
            function getTheme() {{ return localStorage.getItem('theme') || 'dark'; }}
            function setTheme(theme) {{
                document.documentElement.setAttribute('data-theme', theme);
                localStorage.setItem('theme', theme);
            }}
            function toggleTheme() {{
                setTheme(getTheme() === 'dark' ? 'light' : 'dark');
            }}
            setTheme(getTheme());
        </script>
        <style>
            :root {{
                --bg-color: #ffffff;
                --text-color: #111111;
                --card-bg: #f9f9f9;
                --border-color: #cccccc;
                --input-bg: #ffffff;
                --input-text: #111111;
                --primary: #00f0ff;
                --secondary: #8a2be2;
                --neon-glow: rgba(0, 240, 255, 0.4);
                --btn-text: #ffffff;
                --table-bg: #ffffff;
                --table-header: #f4f4f4;
            }}
            [data-theme="dark"] {{
                --bg-color: #0a0a0a;
                --text-color: #e0e0e0;
                --card-bg: #141414;
                --border-color: #333333;
                --input-bg: #1a1a1a;
                --input-text: #e0e0e0;
                --neon-glow: rgba(0, 240, 255, 0.7);
                --table-bg: #111111;
                --table-header: #1a1a1a;
            }}
            body {{
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                max-width: 900px;
                margin: 0 auto;
                padding: 20px;
                background-color: var(--bg-color);
                color: var(--text-color);
                background-image: linear-gradient(var(--border-color) 1px, transparent 1px), linear-gradient(90deg, var(--border-color) 1px, transparent 1px);
                background-size: 40px 40px;
                background-position: center center;
                transition: background-color 0.3s, color 0.3s;
            }}
            [data-theme="dark"] body {{
                background-image: linear-gradient(rgba(0, 240, 255, 0.05) 1px, transparent 1px), linear-gradient(90deg, rgba(0, 240, 255, 0.05) 1px, transparent 1px);
            }}
            .container {{
                background: var(--card-bg);
                padding: 20px;
                border-radius: 8px;
                box-shadow: 0 0 15px var(--neon-glow);
                border: 1px solid var(--border-color);
            }}
            [data-theme="dark"] .container {{
                border-image: linear-gradient(45deg, var(--primary), var(--secondary)) 1;
                border-width: 1px;
                border-style: solid;
            }}
            h1, h2, h3 {{
                color: var(--primary);
                text-shadow: 0 0 8px var(--neon-glow);
                text-transform: uppercase;
                letter-spacing: 1px;
            }}
            table {{ width: 100%; border-collapse: collapse; margin-top: 20px; background: var(--table-bg); }}
            th, td {{ padding: 12px; border-bottom: 1px solid var(--border-color); text-align: left; }}
            th {{ background-color: var(--table-header); color: var(--primary); text-transform: uppercase; }}
            .btn {{ padding: 8px 16px; text-decoration: none; border-radius: 4px; color: var(--btn-text); border: none; cursor: pointer; display: inline-block; font-weight: bold; text-transform: uppercase; transition: box-shadow 0.3s, transform 0.1s; background: linear-gradient(90deg, var(--secondary), var(--primary)); }}
            .btn:hover {{ box-shadow: 0 0 12px var(--neon-glow); }}
            .btn:active {{ transform: scale(0.98); }}
            .btn-primary {{ background: linear-gradient(90deg, var(--secondary), var(--primary)); }}
            .btn-danger {{ background: linear-gradient(90deg, #ff0055, #aa0033); box-shadow: 0 0 8px rgba(255,0,85,0.4); }}
            .btn-danger:hover {{ box-shadow: 0 0 15px rgba(255,0,85,0.7); }}
            .btn-enable {{ background: linear-gradient(90deg, #00ffaa, #009966); box-shadow: 0 0 8px rgba(0,255,170,0.4); }}
            .btn-enable:hover {{ box-shadow: 0 0 15px rgba(0,255,170,0.7); }}
            .btn-disable {{ background: linear-gradient(90deg, #ff0055, #aa0033); box-shadow: 0 0 8px rgba(255,0,85,0.4); }}
            .btn-disable:hover {{ box-shadow: 0 0 15px rgba(255,0,85,0.7); }}
            .btn-logout {{ background: transparent; border: 1px solid var(--border-color); color: var(--text-color); }}
            .btn-logout:hover {{ background: var(--border-color); box-shadow: none; color: var(--bg-color); }}
            .btn-toggle {{ background: transparent; border: 1px solid var(--primary); color: var(--primary); padding: 8px 16px; cursor: pointer; border-radius: 4px; text-transform: uppercase; font-size: 0.8em; font-weight: bold; transition: all 0.3s; margin-right: 10px; }}
            .btn-toggle:hover {{ background: var(--primary); color: var(--bg-color); box-shadow: 0 0 10px var(--neon-glow); }}
            .status-enabled {{ color: #00ffaa; font-weight: bold; text-shadow: 0 0 5px rgba(0,255,170,0.5); }}
            .status-disabled {{ color: #ff0055; font-weight: bold; text-shadow: 0 0 5px rgba(255,0,85,0.5); }}
            .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; flex-wrap: wrap; gap: 10px; }}
            .header-actions {{ display: flex; align-items: center; }}
            .nav {{ margin-bottom: 20px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color); }}
            .nav a {{ margin-right: 15px; text-decoration: none; color: var(--text-color); font-weight: bold; text-transform: uppercase; letter-spacing: 1px; transition: color 0.3s; }}
            .nav a:hover {{ color: var(--primary); text-shadow: 0 0 8px var(--neon-glow); }}
            .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 20px; }}
            .stat-card {{ background: var(--table-header); padding: 15px; border-radius: 6px; border: 1px solid var(--border-color); transition: border-color 0.3s, box-shadow 0.3s; }}
            .stat-card:hover {{ border-color: var(--primary); box-shadow: 0 0 10px var(--neon-glow); }}
            .stat-value {{ font-size: 1.5em; font-weight: bold; color: var(--primary); text-shadow: 0 0 5px var(--neon-glow); margin-top: 10px; }}
            input, select {{ width: 100%; padding: 10px; margin-top: 5px; box-sizing: border-box; background: var(--input-bg); color: var(--input-text); border: 1px solid var(--border-color); border-radius: 4px; outline: none; transition: border-color 0.3s, box-shadow 0.3s; }}
            input:focus, select:focus {{ border-color: var(--primary); box-shadow: 0 0 8px var(--neon-glow); }}
        </style>
    </head>
    <body>
        <div class="container">
    "#, title)
}

fn html_foot() -> String {
    r#"
        </div>
    </body>
    </html>
    "#.to_string()
}

// Protected Dashboard
async fn dashboard(State(state): State<SharedState>, session: Session) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    let is_admin = matches!(user.role, Role::Admin);
    let escaped_username = escape_html(&user.username);

    let services = services::get_all_services();
    let config = state.get_config().await;

    // System Stats
    let mut sys = state.system.lock().unwrap();
    let now = SystemTime::now();
    let mut last_refresh = state.last_system_refresh.lock().unwrap();

    // Throttle refresh to max once every 500ms
    if now.duration_since(*last_refresh).unwrap_or_default().as_millis() > 500 {
        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_disks();
        *last_refresh = now;
    }
    let ram_used = sys.used_memory() / 1024 / 1024; // MB
    let ram_total = sys.total_memory() / 1024 / 1024; // MB
    let swap_used = sys.used_swap() / 1024 / 1024; // MB
    let swap_total = sys.total_swap() / 1024 / 1024; // MB
    let cpu_usage = sys.global_cpu_info().cpu_usage();

    // Simple Disk Usage (Root)
    let mut disk_total = 0;
    let mut disk_used = 0;
    for disk in sys.disks() {
        if disk.mount_point() == std::path::Path::new("/") {
            disk_total = disk.total_space() / 1024 / 1024 / 1024; // GB
            disk_used = (disk.total_space() - disk.available_space()) / 1024 / 1024 / 1024; // GB
            break;
        }
    }
    drop(sys); // Release lock explicitely

    let mut html = html_head("Dashboard - AETHERIS");

    let _ = write!(html, r#"
        <div class="header">
            <h1>AETHERIS 🚀</h1>
            <div class="header-actions">
                <button type="button" class="btn-toggle" onclick="toggleTheme()">Toggle Theme</button>
                <form method="POST" action="/logout" style="margin: 0;">
                    <button type="submit" class="btn btn-logout">Logout ({})</button>
                </form>
            </div>
        </div>
    "#, escaped_username);

    // Navigation
    if is_admin {
        html.push_str(r#"
        <div class="nav">
            <a href="/">Dashboard</a>
            <a href="/users">User Management</a>
        </div>
        "#);
    }

    // Stats Grid
    let _ = write!(html, r#"
        <div class="stats-grid">
            <div class="stat-card">
                <div>CPU Usage</div>
                <div class="stat-value">{:.1}%</div>
            </div>
            <div class="stat-card">
                <div>RAM Usage</div>
                <div class="stat-value">{} / {} MB</div>
            </div>
            <div class="stat-card">
                <div>Swap Usage</div>
                <div class="stat-value">{} / {} MB</div>
            </div>
            <div class="stat-card">
                <div>Disk (/)</div>
                <div class="stat-value">{} / {} GB</div>
            </div>
        </div>
    "#, cpu_usage, ram_used, ram_total, swap_used, swap_total, disk_used, disk_total);

    // Services Table
    html.push_str(r#"
        <h2>Services</h2>
        <table>
            <thead>
                <tr>
                    <th>Service</th>
                    <th>Image</th>
                    <th>Status</th>
                    <th>Action</th>
                </tr>
            </thead>
            <tbody>
    "#);

    for svc in services {
        let name = svc.name();
        let enabled = config.is_enabled(name);
        let status_class = if enabled { "status-enabled" } else { "status-disabled" };
        let status_text = if enabled { "Enabled" } else { "Disabled" };

        let _ = write!(html, r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
                <td class="{}">{}</td>
                <td>
        "#, name, svc.image(), status_class, status_text);

        if is_admin {
             let _ = write!(html, r#"
                    <form method="POST" action="/api/services/{}/{}">
                        <button type="submit" class="btn {}">{}</button>
                    </form>
             "#,
             name,
             if enabled { "disable" } else { "enable" },
             if enabled { "btn-disable" } else { "btn-enable" },
             if enabled { "Disable" } else { "Enable" }
             );
        } else {
            html.push_str("<span>Read-only</span>");
        };

        html.push_str("</td></tr>");
    }

    html.push_str(r#"
            </tbody>
        </table>
        <p><em>Note: Actions may take a moment to apply.</em></p>
    "#);
    html.push_str(&html_foot());

    Html(html).into_response()
}

// User Management Page
async fn users_page(session: Session) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(user.role, Role::Admin) {
        return Redirect::to("/").into_response();
    }

    let user_manager = UserManager::load_async().await.unwrap_or_default();
    let mut html = html_head("User Management - AETHERIS");

    html.push_str(r#"
        <div class="header">
            <h1>User Management 👥</h1>
            <div class="header-actions">
                <button type="button" class="btn-toggle" onclick="toggleTheme()">Toggle Theme</button>
                <form method="POST" action="/logout" style="margin: 0;">
                    <button type="submit" class="btn btn-logout">Logout</button>
                </form>
            </div>
        </div>
        <div class="nav">
            <a href="/">Dashboard</a>
            <a href="/users">User Management</a>
        </div>

        <h3>Add New User</h3>
        <form method="POST" action="/users/add" style="background: var(--table-header); padding: 15px; border-radius: 6px; margin-bottom: 20px; display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px; align-items: end; border: 1px solid var(--border-color);">
            <div>
                <label>Username</label><br>
                <input type="text" name="username" required style="width: 100%; padding: 8px;">
            </div>
            <div>
                <label>Password</label><br>
                <input type="password" name="password" required style="width: 100%; padding: 8px;">
            </div>
            <div>
                <label>Role</label><br>
                <select name="role" style="width: 100%; padding: 8px;">
                    <option value="Observer">Observer</option>
                    <option value="Admin">Admin</option>
                </select>
            </div>
            <div>
                <label>Quota (GB) <small>(0 = unlimited)</small></label><br>
                <input type="number" name="quota" value="0" style="width: 100%; padding: 8px;">
            </div>
            <button type="submit" class="btn btn-primary" style="height: 35px;">Add User</button>
        </form>

        <h3>Existing Users</h3>
        <table>
            <thead>
                <tr>
                    <th>Username</th>
                    <th>Role</th>
                    <th>Quota (GB)</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
    "#);

    for u in user_manager.list_users() {
        let quota_display = match u.quota_gb {
            Some(gb) if gb > 0 => format!("{} GB", gb),
            _ => "Unlimited".to_string(),
        };

        // Don't allow deleting self or last admin logic is handled in delete handler/manager
        // But let's show delete button generally
        html.push_str(&format!(r#"
            <tr>
                <td>{}</td>
                <td>{:?}</td>
                <td>{}</td>
                <td>
                    <form method="POST" action="/users/delete/{}" onsubmit="return confirm('Are you sure you want to delete this user? This will delete their system account and data.');">
                        <button type="submit" class="btn btn-danger">Delete</button>
                    </form>
                </td>
            </tr>
        "#, u.username, u.role, quota_display, u.username));
    }

    html.push_str("</tbody></table>");
    html.push_str(&html_foot());

    Html(html).into_response()
}

#[derive(Deserialize)]
struct AddUserPayload {
    username: String,
    password: String,
    role: String,
    quota: Option<u64>,
}

async fn add_user_handler(session: Session, Form(payload): Form<AddUserPayload>) -> impl IntoResponse {
    let session_user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(session_user.role, Role::Admin) {
        return (StatusCode::FORBIDDEN, "Access Denied").into_response();
    }

    let role_enum = match payload.role.as_str() {
        "Admin" => Role::Admin,
        _ => Role::Observer,
    };

    let quota_val = match payload.quota {
        Some(0) => None,
        Some(v) => Some(v),
        None => None,
    };

    let mut user_manager = UserManager::load_async().await.unwrap_or_default();
    let runtime = crate::adapters::live::LiveAdapter;
    if let Err(e) = user_manager.add_user(&runtime, &payload.username, &payload.password, role_enum, quota_val).await {
        error!("Failed to add user: {}", e);
        // In a real app we'd flash a message. Here just redirect.
    } else {
        info!("User {} added via Web UI by {}", payload.username, session_user.username);
    }

    Redirect::to("/users").into_response()
}

async fn delete_user_handler(session: Session, Path(username): Path<String>) -> impl IntoResponse {
    let session_user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(session_user.role, Role::Admin) {
        return (StatusCode::FORBIDDEN, "Access Denied").into_response();
    }

    let mut user_manager = UserManager::load_async().await.unwrap_or_default();
    let runtime = crate::adapters::live::LiveAdapter;
    if let Err(e) = user_manager.delete_user(&runtime, &username).await {
        error!("Failed to delete user: {}", e);
    } else {
        info!("User {} deleted via Web UI by {}", username, session_user.username);
    }

    Redirect::to("/users").into_response()
}

async fn enable_service(session: Session, Path(name): Path<String>) -> impl IntoResponse {
    check_admin_role(session, &name, true).await
}

async fn disable_service(session: Session, Path(name): Path<String>) -> impl IntoResponse {
    check_admin_role(session, &name, false).await
}

async fn check_admin_role(session: Session, name: &str, enable: bool) -> impl IntoResponse {
    let user: SessionUser = match session.get(SESSION_KEY).await {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };

    if !matches!(user.role, Role::Admin) {
        return (StatusCode::FORBIDDEN, "Access Denied: Admin role required").into_response();
    }

    run_cli_toggle(name, enable);
    Redirect::to("/").into_response()
}

fn run_cli_toggle(service: &str, enable: bool) {
    let action = if enable { "enable" } else { "disable" };
    info!("Web UI triggering: aetheris {} {}", action, service);

    if let Ok(exe) = std::env::current_exe() {
        match Command::new(exe).arg(action).arg(service).spawn() {
            Ok(mut child) => {
                // Spawn a background task to wait for the child process to exit.
                // This prevents zombie processes by collecting the exit status.
                tokio::spawn(async move {
                    if let Err(e) = child.wait().await {
                        error!("Failed to wait on child process: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to spawn command: {}", e);
            }
        }
    } else {
        error!("Failed to determine current executable path.");
    }
}
