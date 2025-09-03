use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};
use crate::models::{Config, GroupConfig, InstallStatus};

pub struct ConfigManager {
    config_path: PathBuf,
    pub config: Config,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_or_create(&config_path)?;
        
        Ok(Self {
            config_path,
            config,
        })
    }
    
    pub fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "zshrcman", "zshrcman")
            .context("Could not determine project directories")?;
        
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        
        Ok(config_dir.join("config.toml"))
    }
    
    pub fn get_dotfiles_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "zshrcman", "zshrcman")
            .context("Could not determine project directories")?;
        
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)?;
        
        Ok(data_dir.join("dotfiles"))
    }
    
    fn load_or_create(path: &Path) -> Result<Config> {
        if path.exists() {
            let contents = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Config::default();
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let toml = toml::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, toml)?;
        Ok(())
    }
    
    pub fn load_group_config(&self, group_name: &str) -> Result<GroupConfig> {
        let dotfiles_path = Self::get_dotfiles_path()?;
        let group_path = dotfiles_path.join("groups").join(format!("{}.toml", group_name));
        
        if !group_path.exists() {
            anyhow::bail!("Group config file does not exist: {:?}", group_path);
        }
        
        let contents = fs::read_to_string(group_path)?;
        let config: GroupConfig = toml::from_str(&contents)?;
        Ok(config)
    }
    
    pub fn load_device_group_config(&self, device: &str, group_name: &str) -> Result<GroupConfig> {
        let dotfiles_path = Self::get_dotfiles_path()?;
        let group_path = dotfiles_path
            .join("devices")
            .join(device)
            .join("groups")
            .join(format!("{}.toml", group_name));
        
        if !group_path.exists() {
            anyhow::bail!("Device group config file does not exist: {:?}", group_path);
        }
        
        let contents = fs::read_to_string(group_path)?;
        let config: GroupConfig = toml::from_str(&contents)?;
        Ok(config)
    }
    
    pub fn add_global_group(&mut self, name: String) -> Result<()> {
        if !self.config.groups.global.contains(&name) {
            self.config.groups.global.push(name);
            self.save()?;
        }
        Ok(())
    }
    
    pub fn remove_global_group(&mut self, name: &str) -> Result<()> {
        if name == "default" {
            anyhow::bail!("Cannot remove built-in 'default' group");
        }
        
        self.config.groups.global.retain(|g| g != name);
        self.config.groups.enabled_global.retain(|g| g != name);
        self.save()?;
        Ok(())
    }
    
    pub fn enable_global_group(&mut self, name: &str) -> Result<()> {
        if self.config.groups.global.contains(&name.to_string()) {
            if !self.config.groups.enabled_global.contains(&name.to_string()) {
                self.config.groups.enabled_global.push(name.to_string());
                self.save()?;
            }
        } else {
            anyhow::bail!("Group '{}' is not defined", name);
        }
        Ok(())
    }
    
    pub fn disable_global_group(&mut self, name: &str) -> Result<()> {
        self.config.groups.enabled_global.retain(|g| g != name);
        self.save()?;
        Ok(())
    }
    
    pub fn update_install_status(&mut self, group: &str, status: InstallStatus) -> Result<()> {
        self.config.status.insert(group.to_string(), status);
        self.save()?;
        Ok(())
    }
    
    pub fn get_ordered_groups(&self) -> Vec<String> {
        let mut groups = Vec::new();
        
        groups.push("default".to_string());
        
        for group in &self.config.groups.enabled_global {
            if group != "default" && !groups.contains(group) {
                groups.push(group.clone());
            }
        }
        
        for device_group in &self.config.groups.enabled_devices {
            if !groups.contains(device_group) {
                groups.push(device_group.clone());
            }
        }
        
        groups
    }
    
    pub fn clear_all_status(&mut self) -> Result<()> {
        self.config.status.clear();
        self.save()?;
        Ok(())
    }
}