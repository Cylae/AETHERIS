use clap::{Parser, Subcommand};
use log::info;
use anyhow::{Result, Context};
use std::io::{self, Write};
use rpassword::read_password;

use crate::core::users::{self, Role};
use crate::adapters::live::LiveAdapter;
use crate::domain::orchestrator::AetherisOrchestrator;

#[derive(Parser)]
#[command(name = "aetheris")]
#[command(about = "AETHERIS: Next-Gen Hexagonal Server Orchestrator", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Full installation (Idempotent)
    Install,
    /// Show system status
    Status,
    /// Manage Users
    User {
        #[command(subcommand)]
        action: UserCommands,
    },
    /// Start the Web Administration Interface
    Web {
        #[arg(long, default_value_t = 8099)]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum UserCommands {
    /// Add a new user
    Add {
        username: String,
        #[arg(long, default_value = "Observer")]
        role: String,
        #[arg(long)]
        quota: Option<u64>,
    },
    /// Delete a user
    Delete { username: String },
    /// List users
    List,
    /// Change user password
    Passwd { username: String },
}

/// Securely reads a password from stdin without echoing characters.
fn read_password_securely(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let password = read_password()
        .context("Failed to read password from terminal. Ensure stdin is a TTY.")?;

    if password.trim().is_empty() {
        return Err(anyhow::anyhow!("Password cannot be empty"));
    }

    Ok(password)
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    // In production, we use the LiveAdapter
    let adapter = Box::new(LiveAdapter);
    let orchestrator = AetherisOrchestrator::new(adapter.clone(), adapter);

    match cli.command {
        Commands::Install => orchestrator.install().await?,
        Commands::Status => orchestrator.status()?,
        Commands::User { action } => run_user_management(action).await?,
        Commands::Web { port } => crate::interface::web::start_server(port).await?,
    }

    Ok(())
}

async fn run_user_management(action: UserCommands) -> Result<()> {
    let mut user_manager = users::UserManager::load()?;
    let adapter = LiveAdapter;

    match action {
        UserCommands::Add { username, role, quota } => {
            let role_enum = match role.to_lowercase().as_str() {
                "admin" => Role::Admin,
                "observer" => Role::Observer,
                _ => return Err(anyhow::anyhow!("Invalid role. Use 'Admin' or 'Observer'")),
            };

            let password = read_password_securely(&format!("Enter password for {}: ", username))?;
            user_manager.add_user(&adapter, &username, &password, role_enum, quota).await?;
            info!("User '{}' added successfully.", username);
        }
        UserCommands::Delete { username } => {
            user_manager.delete_user(&adapter, &username).await?;
            info!("User '{}' deleted successfully.", username);
        }
        UserCommands::List => {
            println!("{:<20} | {:<15} | {:<10}", "Username", "Role", "Quota (GB)");
            println!("{:<20} | {:<15} | {:<10}", "--------", "----", "----------");
            for user in user_manager.list_users() {
                let quota_display = user.quota_gb.map(|q| q.to_string()).unwrap_or_else(|| "Unlimited".to_string());
                println!("{:<20} | {:<15?} | {:<10}", user.username, user.role, quota_display);
            }
        }
        UserCommands::Passwd { username } => {
            if user_manager.get_user(&username).is_none() {
                 return Err(anyhow::anyhow!("User not found"));
            }

            let password = read_password_securely(&format!("Enter new password for {}: ", username))?;
            user_manager.update_password(&adapter, &username, &password).await?;
            info!("Password for '{}' updated successfully.", username);
        }
    }
    Ok(())
}
