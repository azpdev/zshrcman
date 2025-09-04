mod models;
mod modules;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use modules::{
    alias::AliasManager,
    config::ConfigManager,
    git_mgr::GitManager,
    init::InitManager,
    install::InstallManager,
    state_manager::InstallationStateManager,
    profile_switcher::ProfileSwitcher,
};
use strsim::jaro_winkler;

#[derive(Parser)]
#[command(name = "zshrcman")]
#[command(author, version, about = "A Rust-based Zsh/dotfiles manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long, help = "Force re-initialization even if already initialized")]
        force: bool,
    },
    
    Install {
        #[arg(long, help = "Install all groups without prompting")]
        all: bool,
    },
    
    #[command(name = "remove-all")]
    RemoveAll,
    
    Sync {
        #[arg(long, help = "Force sync even with conflicts")]
        force: bool,
    },
    
    #[command(subcommand)]
    Group(GroupCommands),
    
    #[command(subcommand)]
    Device(DeviceCommands),
    
    #[command(subcommand)]
    Alias(AliasCommands),
    
    #[command(subcommand)]
    Profile(ProfileCommands),
    
    Status,
}

#[derive(Subcommand)]
enum GroupCommands {
    List,
    
    Add {
        name: String,
        #[arg(long, help = "Skip typo checking")]
        no_check: bool,
    },
    
    Remove {
        name: String,
    },
    
    Enable {
        name: String,
    },
    
    Disable {
        name: String,
    },
}

#[derive(Subcommand)]
enum DeviceCommands {
    List,
    
    Add {
        name: String,
    },
    
    Remove {
        name: String,
    },
    
    Enable {
        name: String,
    },
    
    Disable {
        name: String,
    },
}

#[derive(Subcommand)]
enum AliasCommands {
    List {
        #[arg(help = "Group name to list aliases for")]
        group: Option<String>,
    },
    
    Add {
        group: String,
        alias_def: String,
    },
    
    Remove {
        group: String,
        alias_def: String,
    },
    
    Toggle {
        group: String,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    List,
    
    Create {
        name: String,
        #[arg(long, help = "Parent profile to inherit from")]
        parent: Option<String>,
    },
    
    Switch {
        name: String,
    },
    
    Delete {
        name: String,
    },
    
    Activate {
        name: String,
    },
    
    Deactivate,
    
    Current,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { force } => {
            if !force {
                if let Ok(config) = ConfigManager::new() {
                    if config.config.repository.url.is_some() {
                        println!("{}", "Already initialized! Use --force to re-initialize.".yellow());
                        return Ok(());
                    }
                }
            }
            InitManager::run()?;
        }
        
        Commands::Install { all } => {
            let config_mgr = ConfigManager::new()?;
            let mut install_mgr = InstallManager::new(config_mgr);
            install_mgr.install(all)?;
        }
        
        Commands::RemoveAll => {
            let config_mgr = ConfigManager::new()?;
            let mut install_mgr = InstallManager::new(config_mgr);
            install_mgr.remove_all()?;
        }
        
        Commands::Sync { force: _ } => {
            let config_mgr = ConfigManager::new()?;
            let dotfiles_path = ConfigManager::get_dotfiles_path()?;
            let git_mgr = GitManager::init_or_clone(
                &dotfiles_path,
                config_mgr.config.repository.url.as_deref(),
            )?;
            
            git_mgr.sync(
                &config_mgr.config.repository.main_branch,
                &config_mgr.config.device.branch,
            )?;
            
            println!("{}", "✅ Repository synced successfully!".green());
        }
        
        Commands::Group(cmd) => handle_group_command(cmd)?,
        
        Commands::Device(cmd) => handle_device_command(cmd)?,
        
        Commands::Alias(cmd) => handle_alias_command(cmd)?,
        
        Commands::Profile(cmd) => handle_profile_command(cmd)?,
        
        Commands::Status => {
            let config_mgr = ConfigManager::new()?;
            
            println!("{}", "📊 zshrcman Status".bold().cyan());
            println!();
            
            if let Some(url) = &config_mgr.config.repository.url {
                println!("  Repository: {}", url);
            } else {
                println!("  Repository: {}", "Not configured".yellow());
            }
            
            println!("  Device: {}", config_mgr.config.device.name);
            println!("  Branch: {}", config_mgr.config.device.branch);
            println!();
            
            println!("{}", "  Global Groups:".bold());
            for group in &config_mgr.config.groups.global {
                let status = if config_mgr.config.groups.enabled_global.contains(group) {
                    "✅ enabled".green()
                } else {
                    "⭕ disabled".yellow()
                };
                println!("    {} - {}", group, status);
            }
            
            println!();
            println!("{}", "  Installation Status:".bold());
            if config_mgr.config.status.is_empty() {
                println!("    {}", "No groups installed".yellow());
            } else {
                for (group, status) in &config_mgr.config.status {
                    let icon = if status.success { "✅" } else { "❌" };
                    println!("    {} {} - {}", 
                        icon, 
                        group,
                        if status.success { "installed" } else { "failed" }
                    );
                }
            }
        }
    }
    
    Ok(())
}

fn handle_group_command(cmd: GroupCommands) -> Result<()> {
    let mut config_mgr = ConfigManager::new()?;
    
    match cmd {
        GroupCommands::List => {
            println!("{}", "📦 Global Groups:".bold());
            for group in &config_mgr.config.groups.global {
                let status = if config_mgr.config.groups.enabled_global.contains(group) {
                    "enabled".green()
                } else {
                    "disabled".yellow()
                };
                println!("  {} [{}]", group, status);
            }
        }
        
        GroupCommands::Add { name, no_check } => {
            if !no_check {
                check_typo(&name, &config_mgr.config.groups.global)?;
            }
            config_mgr.add_global_group(name.clone())?;
            println!("{} {}", "✅ Added group:".green(), name);
        }
        
        GroupCommands::Remove { name } => {
            config_mgr.remove_global_group(&name)?;
            println!("{} {}", "✅ Removed group:".green(), name);
        }
        
        GroupCommands::Enable { name } => {
            config_mgr.enable_global_group(&name)?;
            println!("{} {}", "✅ Enabled group:".green(), name);
        }
        
        GroupCommands::Disable { name } => {
            config_mgr.disable_global_group(&name)?;
            println!("{} {}", "✅ Disabled group:".green(), name);
        }
    }
    
    Ok(())
}

fn handle_device_command(cmd: DeviceCommands) -> Result<()> {
    let mut config_mgr = ConfigManager::new()?;
    
    match cmd {
        DeviceCommands::List => {
            println!("{}", "🖥️  Per-Device Groups:".bold());
            for group in &config_mgr.config.groups.per_device {
                let status = if config_mgr.config.groups.enabled_devices.contains(group) {
                    "enabled".green()
                } else {
                    "disabled".yellow()
                };
                println!("  {} [{}]", group, status);
            }
        }
        
        DeviceCommands::Add { name } => {
            if !config_mgr.config.groups.per_device.contains(&name) {
                config_mgr.config.groups.per_device.push(name.clone());
                config_mgr.save()?;
            }
            println!("{} {}", "✅ Added device group:".green(), name);
        }
        
        DeviceCommands::Remove { name } => {
            config_mgr.config.groups.per_device.retain(|g| g != &name);
            config_mgr.config.groups.enabled_devices.retain(|g| g != &name);
            config_mgr.save()?;
            println!("{} {}", "✅ Removed device group:".green(), name);
        }
        
        DeviceCommands::Enable { name } => {
            if config_mgr.config.groups.per_device.contains(&name) {
                if !config_mgr.config.groups.enabled_devices.contains(&name) {
                    config_mgr.config.groups.enabled_devices.push(name.clone());
                    config_mgr.save()?;
                }
            }
            println!("{} {}", "✅ Enabled device group:".green(), name);
        }
        
        DeviceCommands::Disable { name } => {
            config_mgr.config.groups.enabled_devices.retain(|g| g != &name);
            config_mgr.save()?;
            println!("{} {}", "✅ Disabled device group:".green(), name);
        }
    }
    
    Ok(())
}

fn handle_alias_command(cmd: AliasCommands) -> Result<()> {
    let config_mgr = ConfigManager::new()?;
    let mut alias_mgr = AliasManager::new(config_mgr);
    
    match cmd {
        AliasCommands::List { group } => {
            alias_mgr.list(group.as_deref())?;
        }
        
        AliasCommands::Add { group, alias_def } => {
            alias_mgr.add(&group, &alias_def)?;
        }
        
        AliasCommands::Remove { group, alias_def } => {
            alias_mgr.remove(&group, &alias_def)?;
        }
        
        AliasCommands::Toggle { group } => {
            alias_mgr.toggle(&group)?;
        }
    }
    
    Ok(())
}

fn handle_profile_command(cmd: ProfileCommands) -> Result<()> {
    let config_mgr = ConfigManager::new()?;
    let mut state_mgr = InstallationStateManager::new(config_mgr);
    
    match cmd {
        ProfileCommands::List => {
            println!("{}", "📋 Profiles:".bold());
            for (name, _profile) in &state_mgr.profiles {
                let is_active = state_mgr.active_profile.as_ref() == Some(name);
                let marker = if is_active { " (active)".green() } else { "".normal() };
                println!("  {}{}", name, marker);
            }
            
            if state_mgr.profiles.is_empty() {
                println!("  {}", "No profiles created yet".yellow());
            }
        }
        
        ProfileCommands::Create { name, parent } => {
            state_mgr.create_profile(&name, parent)?;
            println!("{} {}", "✅ Created profile:".green(), name);
        }
        
        ProfileCommands::Switch { name } => {
            let mut switcher = ProfileSwitcher::new(state_mgr);
            switcher.switch_profile(&name)?;
        }
        
        ProfileCommands::Delete { name } => {
            if state_mgr.active_profile.as_ref() == Some(&name) {
                anyhow::bail!("Cannot delete active profile. Switch to another profile first.");
            }
            
            state_mgr.profiles.remove(&name);
            // Save state through state manager
            let config_mgr = ConfigManager::new()?;
            let mut state_mgr_new = InstallationStateManager::new(config_mgr);
            state_mgr_new.profiles = state_mgr.profiles;
            state_mgr_new.save_state()?;
            
            println!("{} {}", "✅ Deleted profile:".green(), name);
        }
        
        ProfileCommands::Activate { name } => {
            let mut switcher = ProfileSwitcher::new(state_mgr);
            switcher.activate_profile(&name)?;
        }
        
        ProfileCommands::Deactivate => {
            let mut switcher = ProfileSwitcher::new(state_mgr);
            switcher.deactivate_current()?;
        }
        
        ProfileCommands::Current => {
            if let Some(current) = &state_mgr.active_profile {
                println!("Current profile: {}", current.green());
            } else {
                println!("{}", "No active profile".yellow());
            }
        }
    }
    
    Ok(())
}

fn check_typo(name: &str, existing: &[String]) -> Result<()> {
    const THRESHOLD: f64 = 0.8;
    
    for existing_name in existing {
        let similarity = jaro_winkler(name, existing_name);
        if similarity > THRESHOLD && name != existing_name {
            println!(
                "{} '{}' is similar to existing group '{}'. Did you mean that?",
                "⚠️  Warning:".yellow(),
                name,
                existing_name
            );
            
            use dialoguer::Confirm;
            let proceed = Confirm::new()
                .with_prompt("Continue anyway?")
                .default(false)
                .interact()?;
            
            if !proceed {
                anyhow::bail!("Aborted due to potential typo");
            }
        }
    }
    
    Ok(())
}