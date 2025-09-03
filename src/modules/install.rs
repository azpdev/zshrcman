use anyhow::{Context, Result};
use dialoguer::Confirm;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::models::{InstallerType, InstallStatus};
use crate::modules::config::ConfigManager;

pub struct InstallManager {
    config_mgr: ConfigManager,
}

impl InstallManager {
    pub fn new(config_mgr: ConfigManager) -> Self {
        Self { config_mgr }
    }
    
    pub fn install(&mut self, all: bool) -> Result<()> {
        let groups = self.config_mgr.get_ordered_groups();
        
        println!("🔧 Installing groups: {:?}", groups);
        
        for group in groups {
            if !all {
                let proceed = Confirm::new()
                    .with_prompt(format!("Install group '{}'?", group))
                    .default(true)
                    .interact()?;
                
                if !proceed {
                    println!("⏭️  Skipping group '{}'", group);
                    continue;
                }
            }
            
            println!("📦 Installing group '{}'...", group);
            
            let result = self.install_group(&group);
            
            let status = match &result {
                Ok(_) => {
                    println!("✅ Successfully installed group '{}'", group);
                    InstallStatus {
                        installed: true,
                        success: true,
                        timestamp: Some(chrono::Utc::now()),
                        error: None,
                    }
                }
                Err(e) => {
                    println!("❌ Failed to install group '{}': {}", group, e);
                    InstallStatus {
                        installed: false,
                        success: false,
                        timestamp: Some(chrono::Utc::now()),
                        error: Some(e.to_string()),
                    }
                }
            };
            
            self.config_mgr.update_install_status(&group, status)?;
        }
        
        println!("🎉 Installation complete!");
        Ok(())
    }
    
    pub fn remove_all(&mut self) -> Result<()> {
        println!("🗑️  Removing all installed groups...");
        
        for (group, status) in self.config_mgr.config.status.clone() {
            if status.installed {
                println!("📦 Uninstalling group '{}'...", group);
                
                match self.uninstall_group(&group) {
                    Ok(_) => println!("✅ Successfully uninstalled group '{}'", group),
                    Err(e) => println!("⚠️  Failed to uninstall group '{}': {}", group, e),
                }
            }
        }
        
        self.config_mgr.clear_all_status()?;
        
        println!("🎉 All groups removed!");
        Ok(())
    }
    
    fn install_group(&self, group_name: &str) -> Result<()> {
        let installer_type = InstallerType::from_group_name(group_name);
        
        let group_config = if let Ok(config) = self.config_mgr.load_group_config(group_name) {
            config
        } else if let Ok(config) = self.config_mgr.load_device_group_config(
            &self.config_mgr.config.device.name, 
            group_name
        ) {
            config
        } else {
            return Ok(());
        };
        
        match installer_type {
            InstallerType::Brew => self.install_brew(&group_config.packages),
            InstallerType::Npm => self.install_npm(&group_config.packages),
            InstallerType::Pnpm => self.install_pnpm(&group_config.packages),
            InstallerType::Aliases => self.install_aliases(group_name),
            InstallerType::Ssh => self.install_ssh(&group_config.ssh_keys),
            InstallerType::Zshrc => self.install_zshrc(&group_config.scripts),
            InstallerType::Custom(_) => {
                println!("ℹ️  Custom installer for '{}' not implemented", group_name);
                Ok(())
            }
        }
    }
    
    fn uninstall_group(&self, group_name: &str) -> Result<()> {
        let installer_type = InstallerType::from_group_name(group_name);
        
        let group_config = if let Ok(config) = self.config_mgr.load_group_config(group_name) {
            config
        } else if let Ok(config) = self.config_mgr.load_device_group_config(
            &self.config_mgr.config.device.name, 
            group_name
        ) {
            config
        } else {
            return Ok(());
        };
        
        match installer_type {
            InstallerType::Brew => self.uninstall_brew(&group_config.packages),
            InstallerType::Npm => self.uninstall_npm(&group_config.packages),
            InstallerType::Pnpm => self.uninstall_pnpm(&group_config.packages),
            InstallerType::Aliases => self.uninstall_aliases(),
            InstallerType::Ssh => Ok(()),
            InstallerType::Zshrc => Ok(()),
            InstallerType::Custom(_) => Ok(()),
        }
    }
    
    fn install_brew(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        let output = Command::new("brew")
            .arg("install")
            .args(packages)
            .output()
            .context("Failed to run brew install")?;
        
        if !output.status.success() {
            anyhow::bail!("brew install failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    fn uninstall_brew(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        Command::new("brew")
            .arg("uninstall")
            .args(packages)
            .output()
            .context("Failed to run brew uninstall")?;
        
        Ok(())
    }
    
    fn install_npm(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        let output = Command::new("npm")
            .arg("install")
            .arg("-g")
            .args(packages)
            .output()
            .context("Failed to run npm install")?;
        
        if !output.status.success() {
            anyhow::bail!("npm install failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    fn uninstall_npm(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        Command::new("npm")
            .arg("uninstall")
            .arg("-g")
            .args(packages)
            .output()
            .context("Failed to run npm uninstall")?;
        
        Ok(())
    }
    
    fn install_pnpm(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        let output = Command::new("pnpm")
            .arg("add")
            .arg("-g")
            .args(packages)
            .output()
            .context("Failed to run pnpm add")?;
        
        if !output.status.success() {
            anyhow::bail!("pnpm add failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    fn uninstall_pnpm(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        Command::new("pnpm")
            .arg("remove")
            .arg("-g")
            .args(packages)
            .output()
            .context("Failed to run pnpm remove")?;
        
        Ok(())
    }
    
    fn install_aliases(&self, group_name: &str) -> Result<()> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let aliases_file = home_dir.join(".zsh_aliases");
        
        let mut aliases_content = if aliases_file.exists() {
            fs::read_to_string(&aliases_file)?
        } else {
            String::new()
        };
        
        if let Some(alias_group) = self.config_mgr.config.aliases.get(group_name) {
            aliases_content.push_str(&format!("\n# Aliases from zshrcman group '{}'\n", group_name));
            
            for alias in &alias_group.active {
                aliases_content.push_str(&format!("{}\n", alias));
            }
        }
        
        fs::write(&aliases_file, aliases_content)?;
        
        Ok(())
    }
    
    fn uninstall_aliases(&self) -> Result<()> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let aliases_file = home_dir.join(".zsh_aliases");
        
        if aliases_file.exists() {
            let content = fs::read_to_string(&aliases_file)?;
            
            let filtered: Vec<&str> = content
                .lines()
                .filter(|line| !line.contains("zshrcman"))
                .collect();
            
            fs::write(&aliases_file, filtered.join("\n"))?;
        }
        
        Ok(())
    }
    
    fn install_ssh(&self, keys: &[String]) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }
        
        let dotfiles_path = ConfigManager::get_dotfiles_path()?;
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let ssh_dir = home_dir.join(".ssh");
        
        fs::create_dir_all(&ssh_dir)?;
        
        for key_name in keys {
            let source = dotfiles_path.join("ssh").join(key_name);
            let target = ssh_dir.join(key_name);
            
            if source.exists() {
                fs::copy(&source, &target)?;
                
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&target)?.permissions();
                    perms.set_mode(0o600);
                    fs::set_permissions(&target, perms)?;
                }
                
                Command::new("ssh-add")
                    .arg(&target)
                    .output()
                    .context("Failed to run ssh-add")?;
            }
        }
        
        Ok(())
    }
    
    fn install_zshrc(&self, scripts: &[String]) -> Result<()> {
        if scripts.is_empty() {
            return Ok(());
        }
        
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let zshrc_file = home_dir.join(".zshrc");
        
        let mut zshrc_content = if zshrc_file.exists() {
            fs::read_to_string(&zshrc_file)?
        } else {
            String::new()
        };
        
        let dotfiles_path = ConfigManager::get_dotfiles_path()?;
        
        zshrc_content.push_str("\n# zshrcman managed scripts\n");
        
        for script in scripts {
            let script_path = dotfiles_path.join("scripts").join(script);
            if script_path.exists() {
                zshrc_content.push_str(&format!("source {}\n", script_path.display()));
            }
        }
        
        fs::write(&zshrc_file, zshrc_content)?;
        
        Ok(())
    }
}