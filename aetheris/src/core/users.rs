use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};
use log::{info, warn};
use bcrypt::{DEFAULT_COST, hash, verify};
use crate::ports::SystemPort;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Role {
    Admin,
    Observer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    #[serde(default)]
    pub quota_gb: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserManager {
    users: HashMap<String, User>,
}

fn get_users_path() -> PathBuf {
    if let Ok(home) = std::env::var("AETHERIS_HOME") {
        return Path::new(&home).join("users.yaml");
    }
    PathBuf::from("users.yaml")
}

impl UserManager {
    pub async fn load_async() -> Result<Self> {
        tokio::task::spawn_blocking(Self::load).await?
    }

    pub fn load() -> Result<Self> {
        let path = get_users_path();
        let fallback_path = Path::new("/opt/aetheris/users.yaml");

        let load_path = if path.exists() {
            Some(path.as_path())
        } else if fallback_path.exists() && std::env::var("AETHERIS_HOME").is_err() {
            // Only fallback if AETHERIS_HOME is not set, otherwise we strictly use AETHERIS_HOME
             Some(fallback_path)
        } else {
            None
        };

        let mut manager = if let Some(p) = load_path {
            let content = fs::read_to_string(p).context("Failed to read users.yaml")?;
            if content.trim().is_empty() {
                UserManager::default()
            } else {
                serde_yaml_ng::from_str(&content).context("Failed to parse users.yaml")?
            }
        } else {
            UserManager::default()
        };

        if manager.users.is_empty() {
            info!("No users found. Creating default 'admin' user.");
            let pass = "admin";
            let hash = hash(pass, DEFAULT_COST)?;
            manager.users.insert("admin".to_string(), User {
                username: "admin".to_string(),
                password_hash: hash,
                role: Role::Admin,
                quota_gb: None,
            });
            manager.save()?;
            info!("Default user 'admin' created with password 'admin'. CHANGE THIS IMMEDIATELY!");
        }

        Ok(manager)
    }

    pub fn save(&self) -> Result<()> {
        let target = if std::env::var("AETHERIS_HOME").is_ok() {
            get_users_path()
        } else if Path::new("/opt/aetheris").exists() {
             Path::new("/opt/aetheris/users.yaml").to_path_buf()
        } else {
             Path::new("users.yaml").to_path_buf()
        };

        let content = serde_yaml_ng::to_string(self)?;
        fs::write(target, content).context("Failed to write users.yaml")?;
        Ok(())
    }

    pub async fn add_user(&mut self, runtime: &dyn SystemPort, username: &str, password: &str, role: Role, quota_gb: Option<u64>) -> Result<()> {
        if self.users.contains_key(username) {
            return Err(anyhow!("User already exists"));
        }

        if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(anyhow!("Invalid username: allowed characters are ASCII alphanumeric, underscore, and hyphen"));
        }

        if runtime.check_root().is_ok() {
            runtime.create_user(username, password).await?;
            if let Some(gb) = quota_gb {
                runtime.set_quota(username, gb).await?;
            }
        } else {
            warn!("Not running as root. Skipping system user creation for '{}'.", username);
        }

        let hash = hash(password, DEFAULT_COST)?;
        self.users.insert(username.to_string(), User {
            username: username.to_string(),
            password_hash: hash,
            role,
            quota_gb,
        });
        self.save()
    }

    pub async fn delete_user(&mut self, runtime: &dyn SystemPort, username: &str) -> Result<()> {
        if let Some(user) = self.users.get(username) {
            if user.role == Role::Admin {
                let admin_count = self.users.values().filter(|u| u.role == Role::Admin).count();
                if admin_count <= 1 {
                    return Err(anyhow!("Cannot delete the last admin user"));
                }
            }
        } else {
            return Err(anyhow!("User not found"));
        }

        if runtime.check_root().is_ok() {
            runtime.delete_user(username).await?;
        } else {
            warn!("Not running as root. Skipping system user deletion for '{}'.", username);
        }

        self.users.remove(username);
        self.save()
    }

    pub async fn update_password(&mut self, runtime: &dyn SystemPort, username: &str, new_password: &str) -> Result<()> {
        if let Some(user) = self.users.get_mut(username) {
            if runtime.check_root().is_ok() {
                runtime.set_password(username, new_password).await?;
            } else {
                warn!("Not running as root. Skipping system password update for '{}'.", username);
            }

            user.password_hash = hash(new_password, DEFAULT_COST)?;
            self.save()
        } else {
            Err(anyhow!("User not found"))
        }
    }

    pub fn verify(&self, username: &str, password: &str) -> Option<User> {
        if let Some(user) = self.users.get(username) {
            if verify(password, &user.password_hash).unwrap_or(false) {
                return Some(user.clone());
            }
        }
        None
    }

    pub async fn verify_async(&self, username: &str, password: &str) -> Option<User> {
        if let Some(user) = self.users.get(username) {
            let hash = user.password_hash.clone();
            let password = password.to_string();
            let user_clone = user.clone();

            let is_valid = tokio::task::spawn_blocking(move || {
                verify(&password, &hash).unwrap_or(false)
            }).await.unwrap_or(false);

            if is_valid {
                return Some(user_clone);
            }
        }
        None
    }

    pub fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }

    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}
