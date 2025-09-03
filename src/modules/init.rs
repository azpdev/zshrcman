use anyhow::{Context, Result};
use dialoguer::{Input, MultiSelect, Select};
use std::fs;
use std::path::Path;
use crate::models::{AliasGroup, GroupConfig};
use crate::modules::config::ConfigManager;
use crate::modules::git_mgr::GitManager;

pub struct InitManager;

impl InitManager {
    pub fn run() -> Result<()> {
        println!("🚀 Welcome to zshrcman initialization!");
        
        let mut config_mgr = ConfigManager::new()?;
        
        let remote_url: String = Input::new()
            .with_prompt("Enter remote Git repository URL")
            .interact_text()?;
        
        config_mgr.config.repository.url = Some(remote_url.clone());
        
        let dotfiles_path = ConfigManager::get_dotfiles_path()?;
        fs::create_dir_all(&dotfiles_path)?;
        
        let git_mgr = GitManager::init_or_clone(&dotfiles_path, Some(&remote_url))?;
        
        let branches = git_mgr.list_remote_branches()
            .unwrap_or_else(|_| vec!["main".to_string()]);
        
        let mut branch_options = branches.clone();
        branch_options.push("Create new device branch".to_string());
        
        let branch_selection = Select::new()
            .with_prompt("Select or create a device branch")
            .items(&branch_options)
            .default(branch_options.len() - 1)
            .interact()?;
        
        let device_branch = if branch_selection == branch_options.len() - 1 {
            let device_name: String = Input::new()
                .with_prompt("Enter device name")
                .interact_text()?;
            
            let branch_name = format!("device/{}", device_name);
            git_mgr.checkout_branch(&branch_name, true)?;
            
            Self::scaffold_device_files(&dotfiles_path, &device_name)?;
            
            config_mgr.config.device.name = device_name;
            config_mgr.config.device.branch = branch_name.clone();
            branch_name
        } else {
            let branch = branches[branch_selection].clone();
            git_mgr.checkout_branch(&branch, false)?;
            
            let device_name = branch.strip_prefix("device/")
                .unwrap_or(&branch)
                .to_string();
            
            config_mgr.config.device.name = device_name;
            config_mgr.config.device.branch = branch.clone();
            branch
        };
        
        Self::ensure_default_groups(&dotfiles_path)?;
        
        let built_in_groups = vec![
            "default", "brew", "npm", "pnpm", "aliases", "ssh", "zshrc"
        ];
        
        let selected_groups = MultiSelect::new()
            .with_prompt("Select groups to enable")
            .items(&built_in_groups)
            .defaults(&vec![true, false, false, false, false, false, false])
            .interact()?;
        
        let mut enabled_groups = Vec::new();
        for idx in selected_groups {
            enabled_groups.push(built_in_groups[idx].to_string());
            
            if !config_mgr.config.groups.global.contains(&built_in_groups[idx].to_string()) {
                config_mgr.config.groups.global.push(built_in_groups[idx].to_string());
            }
        }
        config_mgr.config.groups.enabled_global = enabled_groups;
        
        for group in &config_mgr.config.groups.enabled_global {
            if let Ok(group_config) = config_mgr.load_group_config(group) {
                if !group_config.aliases.is_empty() {
                    let active_aliases = MultiSelect::new()
                        .with_prompt(&format!("Select active aliases for group '{}'", group))
                        .items(&group_config.aliases)
                        .interact()?;
                    
                    let mut active = Vec::new();
                    for idx in active_aliases {
                        active.push(group_config.aliases[idx].clone());
                    }
                    
                    config_mgr.config.aliases.insert(
                        group.clone(),
                        AliasGroup {
                            items: group_config.aliases.clone(),
                            active,
                        },
                    );
                }
            }
        }
        
        config_mgr.save()?;
        
        git_mgr.add_all()?;
        git_mgr.commit_and_push(
            &format!("Initialize zshrcman for device '{}'", config_mgr.config.device.name),
            &device_branch,
        )?;
        
        println!("✅ zshrcman initialized successfully!");
        println!("   Repository: {}", remote_url);
        println!("   Device: {}", config_mgr.config.device.name);
        println!("   Branch: {}", device_branch);
        println!("   Enabled groups: {:?}", config_mgr.config.groups.enabled_global);
        
        Ok(())
    }
    
    fn scaffold_device_files(dotfiles_path: &Path, device_name: &str) -> Result<()> {
        let device_dir = dotfiles_path.join("devices").join(device_name);
        fs::create_dir_all(&device_dir)?;
        fs::create_dir_all(device_dir.join("groups"))?;
        
        let zshrc_content = format!(
            "# .zshrc for device: {}\n\
             # Generated by zshrcman\n\n\
             # Device-specific configuration goes here\n",
            device_name
        );
        
        fs::write(device_dir.join(".zshrc"), zshrc_content)?;
        
        Ok(())
    }
    
    fn ensure_default_groups(dotfiles_path: &Path) -> Result<()> {
        let groups_dir = dotfiles_path.join("groups");
        fs::create_dir_all(&groups_dir)?;
        
        let default_config = GroupConfig {
            name: "default".to_string(),
            description: "Default configuration for all devices".to_string(),
            packages: vec![],
            aliases: vec![
                r#"alias ll="ls -la""#.to_string(),
                r#"alias ..="cd ..""#.to_string(),
                r#"alias ...="cd ../..""#.to_string(),
            ],
            scripts: vec![],
            files: vec![],
            ssh_keys: vec![],
        };
        
        if !groups_dir.join("default.toml").exists() {
            let toml = toml::to_string_pretty(&default_config)?;
            fs::write(groups_dir.join("default.toml"), toml)?;
        }
        
        let brew_config = GroupConfig {
            name: "brew".to_string(),
            description: "Homebrew packages".to_string(),
            packages: vec!["git".to_string(), "curl".to_string(), "wget".to_string()],
            aliases: vec![],
            scripts: vec![],
            files: vec![],
            ssh_keys: vec![],
        };
        
        if !groups_dir.join("brew.toml").exists() {
            let toml = toml::to_string_pretty(&brew_config)?;
            fs::write(groups_dir.join("brew.toml"), toml)?;
        }
        
        let npm_config = GroupConfig {
            name: "npm".to_string(),
            description: "NPM global packages".to_string(),
            packages: vec![],
            aliases: vec![],
            scripts: vec![],
            files: vec![],
            ssh_keys: vec![],
        };
        
        if !groups_dir.join("npm.toml").exists() {
            let toml = toml::to_string_pretty(&npm_config)?;
            fs::write(groups_dir.join("npm.toml"), toml)?;
        }
        
        Ok(())
    }
}