use anyhow::{Context, Result};
use dialoguer::MultiSelect;
use crate::models::AliasGroup;
use crate::modules::config::ConfigManager;

pub struct AliasManager {
    config_mgr: ConfigManager,
}

impl AliasManager {
    pub fn new(config_mgr: ConfigManager) -> Self {
        Self { config_mgr }
    }
    
    pub fn list(&self, group: Option<&str>) -> Result<()> {
        if let Some(group_name) = group {
            if let Some(alias_group) = self.config_mgr.config.aliases.get(group_name) {
                println!("📝 Aliases for group '{}':", group_name);
                println!("   Total: {} | Active: {}", 
                    alias_group.items.len(), 
                    alias_group.active.len()
                );
                println!("\n   All aliases:");
                for alias in &alias_group.items {
                    let status = if alias_group.active.contains(alias) { "✅" } else { "⭕" };
                    println!("   {} {}", status, alias);
                }
            } else {
                println!("No aliases found for group '{}'", group_name);
            }
        } else {
            println!("📝 All alias groups:");
            for (group_name, alias_group) in &self.config_mgr.config.aliases {
                println!("\n   Group '{}': {} total, {} active", 
                    group_name,
                    alias_group.items.len(),
                    alias_group.active.len()
                );
            }
        }
        
        Ok(())
    }
    
    pub fn add(&mut self, group: &str, alias_def: &str) -> Result<()> {
        let alias_group = self.config_mgr.config.aliases
            .entry(group.to_string())
            .or_insert_with(|| AliasGroup {
                items: Vec::new(),
                active: Vec::new(),
            });
        
        if !alias_group.items.contains(&alias_def.to_string()) {
            alias_group.items.push(alias_def.to_string());
            println!("✅ Added alias to group '{}': {}", group, alias_def);
            
            self.config_mgr.save()?;
        } else {
            println!("ℹ️  Alias already exists in group '{}'", group);
        }
        
        Ok(())
    }
    
    pub fn remove(&mut self, group: &str, alias_def: &str) -> Result<()> {
        if let Some(alias_group) = self.config_mgr.config.aliases.get_mut(group) {
            alias_group.items.retain(|a| a != alias_def);
            alias_group.active.retain(|a| a != alias_def);
            
            println!("✅ Removed alias from group '{}': {}", group, alias_def);
            
            self.config_mgr.save()?;
        } else {
            println!("⚠️  Group '{}' not found", group);
        }
        
        Ok(())
    }
    
    pub fn toggle(&mut self, group: &str) -> Result<()> {
        let alias_group = self.config_mgr.config.aliases
            .get(group)
            .context(format!("Group '{}' not found", group))?
            .clone();
        
        if alias_group.items.is_empty() {
            println!("ℹ️  No aliases in group '{}' to toggle", group);
            return Ok(());
        }
        
        let defaults: Vec<bool> = alias_group.items
            .iter()
            .map(|item| alias_group.active.contains(item))
            .collect();
        
        let selected = MultiSelect::new()
            .with_prompt(format!("Toggle active aliases for group '{}'", group))
            .items(&alias_group.items)
            .defaults(&defaults)
            .interact()?;
        
        let mut active = Vec::new();
        for idx in selected {
            active.push(alias_group.items[idx].clone());
        }
        
        self.config_mgr.config.aliases.insert(
            group.to_string(),
            AliasGroup {
                items: alias_group.items,
                active: active.clone(),
            },
        );
        
        self.config_mgr.save()?;
        
        println!("✅ Updated active aliases for group '{}': {} active", 
            group, active.len());
        
        Ok(())
    }
}