use clap::{Parser, Subcommand};
use log::{info, error};
use anyhow::{Result, Context};
use std::io::{self, Write};
use rpassword::read_password;

use crate::core::{hardware, secrets, config, users};
use crate::core::runtime::{LinuxRuntime, SystemRuntime, DockerRuntime};
use crate::services;
use crate::build_compose_structure;

#[derive(Parser)]
#[command(name = "server_manager")]
#[command(about = "Next-Gen Media Server Orchestrator", long_about = None)]
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
    /// Generate docker-compose.yml only
    Generate,
    /// Enable a service
    Enable { service: String },
    /// Disable a service
    Disable { service: String },
    /// Start the Web Administration Interface
    Web {
        #[arg(long, default_value_t = 8099)]
        port: u16,
    },
    /// Manage Users
    User {
        #[command(subcommand)]
        action: UserCommands,
    },
}

#[derive(Subcommand)]
pub enum UserCommands {
    /// Add a new user
    Add {
        username: String,
        #[arg(long, default_value = "Observer")]
        role: String, // "Admin" or "Observer"
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
/// Returns an error if the password is empty or if reading fails.
///
/// SECURITY FIX v1.0.5: Prevents password visibility in terminal and shell history
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
    let runtime = LinuxRuntime;

    match cli.command {
        Commands::Install => run_install(&runtime).await?,
        Commands::Status => run_status(&runtime),
        Commands::Generate => run_generate(&runtime).await?,
        Commands::Enable { service } => run_toggle_service(&runtime, service, true).await?,
        Commands::Disable { service } => run_toggle_service(&runtime, service, false).await?,
        Commands::Web { port } => crate::interface::web::start_server(port).await?,
        Commands::User { action } => run_user_management(&runtime, action)?,
    }

    Ok(())
}

fn run_user_management(runtime: &LinuxRuntime, action: UserCommands) -> Result<()> {
    let mut user_manager = users::UserManager::load()?;

    match action {
        UserCommands::Add { username, role, quota } => {
            let role_enum = match role.to_lowercase().as_str() {
                "admin" => users::Role::Admin,
                "observer" => users::Role::Observer,
                _ => return Err(anyhow::anyhow!("Invalid role. Use 'Admin' or 'Observer'")),
            };

            let password = read_password_securely(&format!("Enter password for {}: ", username))?;

            user_manager.add_user(runtime, &username, &password, role_enum, quota)?;
            info!("User '{}' added successfully.", username);
        }
        UserCommands::Delete { username } => {
            user_manager.delete_user(runtime, &username)?;
            info!("User '{}' deleted successfully.", username);
        }
        UserCommands::List => {
            println!("{:<20} | {:<15}", "Username", "Role");
            println!("{:<20} | {:<15}", "--------", "----");
            for user in user_manager.list_users() {
                println!("{:<20} | {:?}", user.username, user.role);
            }
        }
        UserCommands::Passwd { username } => {
            // Check existence first
            if user_manager.get_user(&username).is_none() {
                 return Err(anyhow::anyhow!("User not found"));
            }

            let password = read_password_securely(&format!("Enter new password for {}: ", username))?;

            user_manager.update_password(runtime, &username, &password)?;
            info!("Password for '{}' updated successfully.", username);
        }
    }
    Ok(())
}

async fn run_toggle_service(runtime: &LinuxRuntime, service_name: String, enable: bool) -> Result<()> {
    // 1. Load Config
    // We assume we are in the install directory or user provides it.
    // For safety, let's try to switch to /opt/server_manager if config not found locally
    if !std::path::Path::new("config.yaml").exists() && std::path::Path::new("/opt/server_manager/config.yaml").exists() {
        std::env::set_current_dir("/opt/server_manager")?;
    }

    let mut config = config::Config::load()?;

    // Check if service exists
    let services = services::get_all_services();
    if !services.iter().any(|s| s.name() == service_name) {
        error!("Service '{}' not found!", service_name);
        return Ok(());
    }

    if enable {
        config.enable_service(&service_name);
    } else {
        config.disable_service(&service_name);
    }

    config.save()?;

    info!("Configuration updated. Re-running generation...");

    // 2. Re-run generation logic (similar to run_generate/run_install subset)
    // We need secrets for this
    let secrets = secrets::Secrets::load_or_create("secrets.yaml")?;
    let hw = hardware::HardwareInfo::detect(runtime);

    // Only configure/generate, don't necessarily fully install dependencies again
    // But we should probably trigger docker compose up to apply changes
    configure_services(runtime, &hw, &secrets, &config)?;
    initialize_services(runtime, &hw, &secrets, &config)?;
    generate_compose(runtime, &hw, &secrets, &config).await?;

    info!("Applying changes via Docker Compose...");
    if runtime.run_compose_up().is_ok() {
        info!("Service '{}' {} successfully!", service_name, if enable { "enabled" } else { "disabled" });
    } else {
        error!("Failed to apply changes via Docker Compose.");
    }

    Ok(())
}

async fn run_install(runtime: &LinuxRuntime) -> Result<()> {
    info!("Starting Server Manager Installation...");

    // 1. Root Check
    runtime.check_root()?;

    // 1.1 Create Install Directory
    let install_dir = std::path::Path::new("/opt/server_manager");
    if !install_dir.exists() {
        info!("Creating installation directory at /opt/server_manager...");
        runtime.create_dir_all(install_dir).context("Failed to create /opt/server_manager")?;
    }
    std::env::set_current_dir(install_dir).context("Failed to chdir to /opt/server_manager")?;

    // 1.2 Load Secrets & Config
    let secrets = secrets::Secrets::load_or_create("secrets.yaml")?;
    let config = config::Config::load()?;

    // 2. Hardware Detection
    let hw = hardware::HardwareInfo::detect(runtime);

    // 3. System Dependencies & Optimization
    runtime.install_dependencies()?;
    runtime.apply_optimizations(&hw)?;

    // 4. Firewall
    runtime.configure_firewall()?;

    // 5. Docker
    runtime.install()?;

    // 6. Initialize Services
    configure_services(runtime, &hw, &secrets, &config)?;
    initialize_services(runtime, &hw, &secrets, &config)?;

    // 7. Generate Compose
    generate_compose(runtime, &hw, &secrets, &config).await?;

    // 8. Launch
    info!("Launching Services via Docker Compose...");
    if runtime.run_compose_up().is_ok() {
        info!("Server Manager Stack Deployed Successfully! 🚀");
        print_deployment_summary(&secrets);
    } else {
        error!("Docker Compose failed.");
    }

    Ok(())
}

fn print_deployment_summary(secrets: &secrets::Secrets) {
    println!("\n=================================================================================");
    println!("                           DEPLOYMENT SUMMARY 🚀");
    println!("=================================================================================");
    println!("{:<15} | {:<25} | {:<15} | Password / Info", "Service", "URL", "User");
    println!("{:<15} | {:<25} | {:<15} | ---------------", "-------", "---", "----");

    let print_row = |service: &str, url: &str, user: &str, pass: &str| {
        println!("{:<15} | {:<25} | {:<15} | {}", service, url, user, pass);
    };

    // Helper to format Option<String>
    let pass = |opt: &Option<String>| opt.clone().unwrap_or_else(|| "ERROR".to_string());

    print_row("Nginx Proxy", "http://<IP>:81", "admin@example.com", "changeme");
    print_row("Portainer", "http://<IP>:9000", "admin", "Set on first login");
    print_row("Nextcloud", "https://<IP>:4443", "admin", &pass(&secrets.nextcloud_admin_password));
    print_row("Vaultwarden", "http://<IP>:8001/admin", "(Token)", &pass(&secrets.vaultwarden_admin_token));
    print_row("Gitea", "http://<IP>:3000", "Register", "DB pre-configured");
    print_row("GLPI", "http://<IP>:8088", "glpi", "glpi (Change immediately!)");
    print_row("Yourls", "http://<IP>:8003/admin", "admin", &pass(&secrets.yourls_admin_password));
    print_row("Roundcube", "http://<IP>:8090", "-", "Login with Mail creds");
    print_row("MailServer", "PORTS: 25, 143...", "CLI", "docker exec -ti mailserver setup ...");
    print_row("Plex", "http://<IP>:32400/web", "-", "Follow Web Setup");
    print_row("ArrStack", "http://<IP>:8989 (Sonarr)", "-", "No auth by default");

    println!("=================================================================================\n");
    println!("NOTE: Replace <IP> with your server's IP address.");
}

fn run_status(runtime: &LinuxRuntime) {
    let hw = hardware::HardwareInfo::detect(runtime);
    println!("=== System Status ===");
    println!("RAM: {} GB", hw.ram_gb);
    println!("Swap: {} GB", hw.swap_gb);
    println!("Disk: {} GB", hw.disk_gb);
    println!("Cores: {}", hw.cpu_cores);
    println!("Profile: {:?}", hw.profile);
    println!("Nvidia GPU: {}", hw.has_nvidia);
    println!("Intel QuickSync: {}", hw.has_intel_quicksync);

    println!("\n=== Docker Status ===");
    if runtime.is_installed() {
         println!("Docker is installed.");
    } else {
         println!("Docker is NOT installed.");
    }
}

async fn run_generate(runtime: &LinuxRuntime) -> Result<()> {
    let hw = hardware::HardwareInfo::detect(runtime);
    // For generate, we might not be in /opt/server_manager, but let's try to load secrets from CWD.
    // We propagate the error because generating a compose file with empty passwords is bad.
    let secrets = secrets::Secrets::load_or_create("secrets.yaml").context("Failed to load or create secrets.yaml")?;
    let config = config::Config::load()?;
    configure_services(runtime, &hw, &secrets, &config)?;
    generate_compose(runtime, &hw, &secrets, &config).await
}

fn configure_services(runtime: &LinuxRuntime, hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, config: &config::Config) -> Result<()> {
    info!("Configuring services (generating config files)...");
    let services = services::get_all_services();
    for service in services {
        if !config.is_enabled(service.name()) {
            continue;
        }
        service.configure(runtime, hw, secrets).with_context(|| format!("Failed to configure service: {}", service.name()))?;
    }
    Ok(())
}

fn initialize_services(runtime: &LinuxRuntime, hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, config: &config::Config) -> Result<()> {
    info!("Initializing services (system setup)...");
    let services = services::get_all_services();
    for service in services {
        if !config.is_enabled(service.name()) {
            continue;
        }
        service.initialize(runtime, hw, secrets).with_context(|| format!("Failed to initialize service: {}", service.name()))?;
    }
    Ok(())
}

async fn generate_compose(runtime: &LinuxRuntime, hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, config: &config::Config) -> Result<()> {
    info!("Generating docker-compose.yml based on hardware profile...");
    let top_level = build_compose_structure(hw, secrets, config)?;
    let yaml_output = serde_yaml_ng::to_string(&top_level)?;

    runtime.write_file(std::path::Path::new("docker-compose.yml"), &yaml_output).context("Failed to write docker-compose.yml")?;
    info!("docker-compose.yml generated.");

    Ok(())
}
