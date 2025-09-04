use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::env;
use crate::modules::state_manager::InstallationStateManager;
use crate::modules::environment::EnvironmentManager;

pub struct ProfileSwitcher {
    state_mgr: InstallationStateManager,
    env_mgr: EnvironmentManager,
}

impl ProfileSwitcher {
    pub fn new(state_mgr: InstallationStateManager) -> Self {
        let env_mgr = EnvironmentManager::new();
        Self { state_mgr, env_mgr }
    }
    
    pub fn switch_profile(&mut self, new_profile: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        let old_profile = self.state_mgr.active_profile.clone();
        
        // Step 1: Deactivate old profile's environment
        if let Some(old) = &old_profile {
            self.deactivate_environment(old)?;
        }
        
        // Step 2: Switch to new profile in state manager
        self.state_mgr.switch_profile(new_profile)?;
        
        // Step 3: Activate new profile's environment
        self.activate_environment(new_profile)?;
        
        // Step 4: Update symlinks for profile-specific tools
        self.update_active_binaries(new_profile)?;
        
        // Step 5: Update shell configuration
        self.update_shell_config(new_profile)?;
        
        let duration = start.elapsed();
        println!("✅ Switched to profile '{}' in {:?}", new_profile, duration);
        
        Ok(())
    }
    
    pub fn activate_profile(&mut self, profile: &str) -> Result<()> {
        self.activate_environment(profile)?;
        self.update_active_binaries(profile)?;
        self.update_shell_config(profile)?;
        println!("✅ Profile '{}' activated", profile);
        Ok(())
    }
    
    pub fn deactivate_current(&mut self) -> Result<()> {
        if let Some(profile) = self.state_mgr.active_profile.clone() {
            self.deactivate_environment(&profile)?;
            self.clear_profile_binaries(&profile)?;
            self.state_mgr.active_profile = None;
            println!("✅ Profile '{}' deactivated", profile);
        }
        Ok(())
    }
    
    fn activate_environment(&self, profile: &str) -> Result<()> {
        if let Some(profile_state) = self.state_mgr.profiles.get(profile) {
            // Apply environment variables
            self.env_mgr.apply_profile_environment(&profile_state.environment)?;
            
            // Update PATH with profile-specific directories
            let profile_bin_dir = self.get_profile_bin_dir(profile)?;
            self.add_to_path(&profile_bin_dir)?;
        }
        
        Ok(())
    }
    
    fn deactivate_environment(&self, profile: &str) -> Result<()> {
        if let Some(profile_state) = self.state_mgr.profiles.get(profile) {
            // Remove profile-specific environment variables
            self.env_mgr.clear_profile_environment(&profile_state.environment)?;
            
            // Remove from PATH
            let profile_bin_dir = self.get_profile_bin_dir(profile)?;
            self.remove_from_path(&profile_bin_dir)?;
        }
        
        Ok(())
    }
    
    fn update_active_binaries(&self, profile: &str) -> Result<()> {
        let profile_bin = self.get_profile_bin_dir(profile)?;
        
        // Create profile bin directory if it doesn't exist
        fs::create_dir_all(&profile_bin)?;
        
        // Clear old symlinks
        if profile_bin.exists() {
            for entry in fs::read_dir(&profile_bin)? {
                let entry = entry?;
                if entry.path().is_file() || entry.path().is_symlink() {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        
        // Create new symlinks for active packages
        for package in self.state_mgr.get_active_packages(profile)? {
            if let Some(record) = self.state_mgr.get_package_info(&package) {
                if let Some(location) = &record.location {
                    let target = profile_bin.join(&package);
                    self.create_symlink(location, &target)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn update_shell_config(&self, profile: &str) -> Result<()> {
        let shell_config = self.get_shell_config_path()?;
        let profile_marker = format!("# ZSHRCMAN_PROFILE: {}", profile);
        
        // Read existing config
        let mut content = if shell_config.exists() {
            fs::read_to_string(&shell_config)?
        } else {
            String::new()
        };
        
        // Remove old profile marker if exists
        if let Some(start) = content.find("# ZSHRCMAN_PROFILE:") {
            if let Some(end) = content[start..].find('\n') {
                content.replace_range(start..start + end + 1, "");
            }
        }
        
        // Add new profile marker
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(&profile_marker);
        content.push('\n');
        
        // Write back
        fs::write(&shell_config, content)?;
        
        Ok(())
    }
    
    fn clear_profile_binaries(&self, profile: &str) -> Result<()> {
        let profile_bin = self.get_profile_bin_dir(profile)?;
        if profile_bin.exists() {
            for entry in fs::read_dir(&profile_bin)? {
                let entry = entry?;
                if entry.path().is_file() || entry.path().is_symlink() {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(())
    }
    
    fn get_profile_bin_dir(&self, profile: &str) -> Result<PathBuf> {
        let home = env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("zshrcman")
            .join("profiles")
            .join(profile)
            .join("bin"))
    }
    
    fn get_shell_config_path(&self) -> Result<PathBuf> {
        let home = env::var("HOME").context("HOME not set")?;
        
        // Determine shell config file based on current shell
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        
        let config_file = if shell.contains("zsh") {
            ".zshrc"
        } else if shell.contains("bash") {
            ".bashrc"
        } else if shell.contains("fish") {
            ".config/fish/config.fish"
        } else {
            ".profile"
        };
        
        Ok(PathBuf::from(home).join(config_file))
    }
    
    fn add_to_path(&self, dir: &PathBuf) -> Result<()> {
        let current_path = env::var("PATH").unwrap_or_default();
        let dir_str = dir.to_string_lossy();
        
        if !current_path.contains(&*dir_str) {
            let new_path = format!("{}:{}", dir_str, current_path);
            env::set_var("PATH", new_path);
        }
        
        Ok(())
    }
    
    fn remove_from_path(&self, dir: &PathBuf) -> Result<()> {
        let current_path = env::var("PATH").unwrap_or_default();
        let dir_str = dir.to_string_lossy();
        
        let paths: Vec<&str> = current_path.split(':').filter(|p| *p != dir_str).collect();
        let new_path = paths.join(":");
        
        env::set_var("PATH", new_path);
        Ok(())
    }
    
    #[cfg(unix)]
    fn create_symlink(&self, source: &PathBuf, target: &PathBuf) -> Result<()> {
        std::os::unix::fs::symlink(source, target)?;
        Ok(())
    }
    
    #[cfg(windows)]
    fn create_symlink(&self, source: &PathBuf, target: &PathBuf) -> Result<()> {
        std::os::windows::fs::symlink_file(source, target)?;
        Ok(())
    }
}