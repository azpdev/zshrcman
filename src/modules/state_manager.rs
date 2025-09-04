use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crate::models::{
    InstallationRecord, InstallationSource, InstallScope, 
    Profile, RemovalStrategy, OsType
};
use crate::modules::config::ConfigManager;

pub struct InstallationStateManager {
    pub installations: HashMap<String, InstallationRecord>,
    pub profiles: HashMap<String, Profile>,
    pub active_profile: Option<String>,
    config_mgr: ConfigManager,
}

impl InstallationStateManager {
    pub fn new(config_mgr: ConfigManager) -> Self {
        let installations = config_mgr.config.installations.clone();
        let profiles = config_mgr.config.profiles.clone();
        let active_profile = config_mgr.config.active_profile.clone();
        
        Self {
            installations,
            profiles,
            active_profile,
            config_mgr,
        }
    }
    
    pub fn is_installed(&self, package: &str) -> bool {
        self.installations.contains_key(package)
    }
    
    pub fn is_active(&self, package: &str) -> bool {
        if let Some(record) = self.installations.get(package) {
            if let Some(profile_id) = &self.active_profile {
                return record.active_for.contains(profile_id);
            }
        }
        false
    }
    
    pub fn smart_install(&mut self, package: &str, scope: InstallScope) -> Result<()> {
        if self.is_installed(package) {
            println!("📦 {} already installed, activating for current profile", package);
            self.activate_for_profile(package)?;
        } else {
            println!("📦 Installing {} with scope {:?}", package, scope);
            self.perform_installation(package, scope)?;
        }
        Ok(())
    }
    
    pub fn handle_removal(&mut self, package: &str, strategy: RemovalStrategy) -> Result<()> {
        match strategy {
            RemovalStrategy::Deactivate => {
                self.deactivate_for_profile(package)?;
            },
            
            RemovalStrategy::RemoveFromProfile => {
                self.remove_from_profile_list(package)?;
                if !self.used_by_other_profiles(package)? {
                    self.deactivate_for_profile(package)?;
                }
            },
            
            RemovalStrategy::SmartRemove => {
                let usage_count = self.get_usage_count(package)?;
                
                if usage_count <= 1 {
                    self.perform_uninstallation(package)?;
                } else {
                    self.deactivate_for_profile(package)?;
                    println!("ℹ️ {} still used by {} other profiles, deactivated only", 
                            package, usage_count - 1);
                }
            },
            
            RemovalStrategy::ForceRemove => {
                self.perform_uninstallation(package)?;
                self.remove_from_all_profiles(package)?;
            },
            
            RemovalStrategy::MarkUnused => {
                self.mark_for_gc(package)?;
                self.deactivate_for_profile(package)?;
            },
        }
        Ok(())
    }
    
    fn activate_for_profile(&mut self, package: &str) -> Result<()> {
        if let Some(profile_id) = &self.active_profile {
            if let Some(record) = self.installations.get_mut(package) {
                record.active_for.insert(profile_id.clone());
            }
            
            if let Some(profile) = self.profiles.get_mut(profile_id) {
                profile.packages.insert(package.to_string());
            }
            
            self.save_state()?;
        }
        Ok(())
    }
    
    fn deactivate_for_profile(&mut self, package: &str) -> Result<()> {
        if let Some(profile_id) = &self.active_profile {
            if let Some(record) = self.installations.get_mut(package) {
                record.active_for.remove(profile_id);
            }
            
            if let Some(profile) = self.profiles.get_mut(profile_id) {
                profile.packages.remove(package);
            }
            
            self.save_state()?;
        }
        Ok(())
    }
    
    fn perform_installation(&mut self, package: &str, scope: InstallScope) -> Result<()> {
        // This would call the actual installer (brew, npm, etc.)
        // For now, we'll create a record
        let profile_id = self.active_profile.clone().unwrap_or_else(|| "default".to_string());
        
        let record = InstallationRecord {
            package: package.to_string(),
            version: None,
            installed_at: chrono::Utc::now(),
            installed_by: InstallationSource::Profile(profile_id.clone()),
            active_for: {
                let mut set = HashSet::new();
                set.insert(profile_id.clone());
                set
            },
            scope,
            location: None,
            installer_type: "auto".to_string(),
        };
        
        self.installations.insert(package.to_string(), record);
        
        if let Some(profile) = self.profiles.get_mut(&profile_id) {
            profile.packages.insert(package.to_string());
        }
        
        self.save_state()?;
        Ok(())
    }
    
    fn perform_uninstallation(&mut self, package: &str) -> Result<()> {
        // This would call the actual uninstaller
        self.installations.remove(package);
        self.save_state()?;
        Ok(())
    }
    
    fn remove_from_profile_list(&mut self, package: &str) -> Result<()> {
        if let Some(profile_id) = &self.active_profile {
            if let Some(profile) = self.profiles.get_mut(profile_id) {
                profile.packages.remove(package);
            }
        }
        self.save_state()?;
        Ok(())
    }
    
    fn used_by_other_profiles(&self, package: &str) -> Result<bool> {
        if let Some(record) = self.installations.get(package) {
            if let Some(current) = &self.active_profile {
                return Ok(record.active_for.iter().any(|p| p != current));
            }
        }
        Ok(false)
    }
    
    fn get_usage_count(&self, package: &str) -> Result<usize> {
        if let Some(record) = self.installations.get(package) {
            Ok(record.active_for.len())
        } else {
            Ok(0)
        }
    }
    
    fn remove_from_all_profiles(&mut self, package: &str) -> Result<()> {
        for profile in self.profiles.values_mut() {
            profile.packages.remove(package);
        }
        self.save_state()?;
        Ok(())
    }
    
    fn mark_for_gc(&mut self, _package: &str) -> Result<()> {
        // TODO: Implement garbage collection marking
        Ok(())
    }
    
    pub fn save_state(&mut self) -> Result<()> {
        self.config_mgr.config.installations = self.installations.clone();
        self.config_mgr.config.profiles = self.profiles.clone();
        self.config_mgr.config.active_profile = self.active_profile.clone();
        self.config_mgr.save()?;
        Ok(())
    }
    
    pub fn create_profile(&mut self, name: &str, parent: Option<String>) -> Result<()> {
        let profile = Profile {
            name: name.to_string(),
            parent,
            packages: HashSet::new(),
            environment: Default::default(),
            os_overrides: HashMap::new(),
        };
        
        self.profiles.insert(name.to_string(), profile);
        self.save_state()?;
        Ok(())
    }
    
    pub fn switch_profile(&mut self, name: &str) -> Result<()> {
        if !self.profiles.contains_key(name) {
            anyhow::bail!("Profile '{}' does not exist", name);
        }
        
        self.active_profile = Some(name.to_string());
        self.save_state()?;
        Ok(())
    }
    
    pub fn get_active_packages(&self, profile: &str) -> Result<Vec<String>> {
        if let Some(profile_data) = self.profiles.get(profile) {
            Ok(profile_data.packages.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    pub fn get_package_info(&self, package: &str) -> Option<&InstallationRecord> {
        self.installations.get(package)
    }
}