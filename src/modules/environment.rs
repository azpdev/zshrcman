use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use crate::models::EnvironmentState;

#[derive(Debug, Clone)]
pub enum ShellType {
    Zsh,
    Bash,
    Fish,
    PowerShell,
    Cmd,
}

pub struct EnvironmentManager {
    shell_type: ShellType,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        let shell_type = Self::detect_shell();
        Self { shell_type }
    }
    
    fn detect_shell() -> ShellType {
        if cfg!(windows) {
            if env::var("PSModulePath").is_ok() {
                ShellType::PowerShell
            } else {
                ShellType::Cmd
            }
        } else {
            match env::var("SHELL").unwrap_or_default().as_str() {
                s if s.contains("zsh") => ShellType::Zsh,
                s if s.contains("bash") => ShellType::Bash,
                s if s.contains("fish") => ShellType::Fish,
                _ => ShellType::Bash,
            }
        }
    }
    
    pub fn apply_profile_environment(&self, env_state: &EnvironmentState) -> Result<()> {
        if !env_state.active {
            return Ok(());
        }
        
        // Apply PATH modifications
        self.apply_path_changes(env_state)?;
        
        // Apply environment variables
        for (key, value) in &env_state.variables {
            env::set_var(key, value);
        }
        
        Ok(())
    }
    
    pub fn clear_profile_environment(&self, env_state: &EnvironmentState) -> Result<()> {
        // Remove PATH modifications
        self.remove_path_changes(env_state)?;
        
        // Clear environment variables (we can't truly unset them in the current process,
        // but we can set them to empty)
        for key in env_state.variables.keys() {
            env::remove_var(key);
        }
        
        Ok(())
    }
    
    pub fn generate_shell_config(&self, env_state: &EnvironmentState) -> Result<String> {
        match self.shell_type {
            ShellType::Zsh | ShellType::Bash => self.generate_bash_config(env_state),
            ShellType::Fish => self.generate_fish_config(env_state),
            ShellType::PowerShell => self.generate_powershell_config(env_state),
            ShellType::Cmd => self.generate_cmd_config(env_state),
        }
    }
    
    pub fn write_shell_config(&self, env_state: &EnvironmentState) -> Result<()> {
        let config = self.generate_shell_config(env_state)?;
        let config_path = self.get_profile_env_path()?;
        
        // Create parent directory if needed
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&config_path, config)?;
        
        // Source the config in the main shell config file
        self.add_source_line(&config_path)?;
        
        Ok(())
    }
    
    fn apply_path_changes(&self, env_state: &EnvironmentState) -> Result<()> {
        let mut current_path = env::var("PATH").unwrap_or_default();
        
        // Prepend paths
        for path in &env_state.paths_prepend {
            let expanded = self.expand_path(path)?;
            if !current_path.contains(&expanded) {
                current_path = format!("{}:{}", expanded, current_path);
            }
        }
        
        // Append paths
        for path in &env_state.paths_append {
            let expanded = self.expand_path(path)?;
            if !current_path.contains(&expanded) {
                current_path = format!("{}:{}", current_path, expanded);
            }
        }
        
        env::set_var("PATH", current_path);
        Ok(())
    }
    
    fn remove_path_changes(&self, env_state: &EnvironmentState) -> Result<()> {
        let current_path = env::var("PATH").unwrap_or_default();
        let mut paths: Vec<String> = current_path.split(':').map(|s| s.to_string()).collect();
        
        // Remove prepended paths
        for path in &env_state.paths_prepend {
            let expanded = self.expand_path(path)?;
            paths.retain(|p| p != &expanded);
        }
        
        // Remove appended paths
        for path in &env_state.paths_append {
            let expanded = self.expand_path(path)?;
            paths.retain(|p| p != &expanded);
        }
        
        env::set_var("PATH", paths.join(":"));
        Ok(())
    }
    
    fn expand_path(&self, path: &str) -> Result<String> {
        // Expand environment variables and tilde
        let expanded = if path.starts_with("~/") {
            let home = env::var("HOME").context("HOME not set")?;
            path.replacen("~", &home, 1)
        } else if path.starts_with("$HOME") {
            let home = env::var("HOME").context("HOME not set")?;
            path.replacen("$HOME", &home, 1)
        } else {
            path.to_string()
        };
        
        Ok(expanded)
    }
    
    fn generate_bash_config(&self, env_state: &EnvironmentState) -> Result<String> {
        let mut script = String::new();
        
        script.push_str("# zshrcman profile environment\n\n");
        
        // PATH modifications
        for path in &env_state.paths_prepend {
            script.push_str(&format!("export PATH=\"{}:$PATH\"\n", path));
        }
        
        for path in &env_state.paths_append {
            script.push_str(&format!("export PATH=\"$PATH:{}\"\n", path));
        }
        
        if !env_state.paths_prepend.is_empty() || !env_state.paths_append.is_empty() {
            script.push('\n');
        }
        
        // Environment variables
        for (key, value) in &env_state.variables {
            script.push_str(&format!("export {}=\"{}\"\n", key, value));
        }
        
        if !env_state.variables.is_empty() {
            script.push('\n');
        }
        
        // Aliases
        for (alias, command) in &env_state.aliases {
            script.push_str(&format!("alias {}='{}'\n", alias, command));
        }
        
        Ok(script)
    }
    
    fn generate_fish_config(&self, env_state: &EnvironmentState) -> Result<String> {
        let mut script = String::new();
        
        script.push_str("# zshrcman profile environment\n\n");
        
        // PATH modifications
        for path in &env_state.paths_prepend {
            script.push_str(&format!("set -gx PATH {} $PATH\n", path));
        }
        
        for path in &env_state.paths_append {
            script.push_str(&format!("set -gx PATH $PATH {}\n", path));
        }
        
        if !env_state.paths_prepend.is_empty() || !env_state.paths_append.is_empty() {
            script.push('\n');
        }
        
        // Environment variables
        for (key, value) in &env_state.variables {
            script.push_str(&format!("set -gx {} \"{}\"\n", key, value));
        }
        
        if !env_state.variables.is_empty() {
            script.push('\n');
        }
        
        // Aliases
        for (alias, command) in &env_state.aliases {
            script.push_str(&format!("alias {} '{}'\n", alias, command));
        }
        
        Ok(script)
    }
    
    fn generate_powershell_config(&self, env_state: &EnvironmentState) -> Result<String> {
        let mut script = String::new();
        
        script.push_str("# zshrcman profile environment\n\n");
        
        // PATH modifications
        if !env_state.paths_prepend.is_empty() || !env_state.paths_append.is_empty() {
            script.push_str("$env:Path = @(");
            
            for path in &env_state.paths_prepend {
                script.push_str(&format!("\n    \"{}\",", path));
            }
            
            script.push_str("\n    $env:Path");
            
            for path in &env_state.paths_append {
                script.push_str(&format!(",\n    \"{}\"", path));
            }
            
            script.push_str("\n) -join ';'\n\n");
        }
        
        // Environment variables
        for (key, value) in &env_state.variables {
            script.push_str(&format!("$env:{} = \"{}\"\n", key, value));
        }
        
        if !env_state.variables.is_empty() {
            script.push('\n');
        }
        
        // Aliases (functions in PowerShell)
        for (alias, command) in &env_state.aliases {
            script.push_str(&format!("function {} {{ {} }}\n", alias, command));
        }
        
        Ok(script)
    }
    
    fn generate_cmd_config(&self, env_state: &EnvironmentState) -> Result<String> {
        let mut script = String::new();
        
        script.push_str("@echo off\nREM zshrcman profile environment\n\n");
        
        // PATH modifications
        if !env_state.paths_prepend.is_empty() || !env_state.paths_append.is_empty() {
            script.push_str("set PATH=");
            
            for path in &env_state.paths_prepend {
                script.push_str(&format!("{};", path));
            }
            
            script.push_str("%PATH%");
            
            for path in &env_state.paths_append {
                script.push_str(&format!(";{}", path));
            }
            
            script.push_str("\n\n");
        }
        
        // Environment variables
        for (key, value) in &env_state.variables {
            script.push_str(&format!("set {}={}\n", key, value));
        }
        
        if !env_state.variables.is_empty() {
            script.push('\n');
        }
        
        // Note: CMD doesn't support aliases directly
        if !env_state.aliases.is_empty() {
            script.push_str("REM Aliases not supported in CMD batch files\n");
            for (alias, command) in &env_state.aliases {
                script.push_str(&format!("REM {} = {}\n", alias, command));
            }
        }
        
        Ok(script)
    }
    
    fn get_profile_env_path(&self) -> Result<PathBuf> {
        let home = env::var("HOME").unwrap_or_else(|_| {
            env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
        });
        
        Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("zshrcman")
            .join("env")
            .join("profile.env"))
    }
    
    fn add_source_line(&self, env_path: &PathBuf) -> Result<()> {
        let shell_config = self.get_shell_config_path()?;
        let env_path_str = env_path.to_string_lossy();
        
        let source_line = match self.shell_type {
            ShellType::Zsh | ShellType::Bash => {
                format!("[ -f {} ] && source {}", env_path_str, env_path_str)
            }
            ShellType::Fish => {
                format!("test -f {}; and source {}", env_path_str, env_path_str)
            }
            ShellType::PowerShell => {
                format!(". \"{}\"", env_path_str)
            }
            ShellType::Cmd => {
                return Ok(()); // CMD doesn't have a persistent config file like shells
            }
        };
        
        // Check if source line already exists
        if shell_config.exists() {
            let content = fs::read_to_string(&shell_config)?;
            if content.contains(&source_line) {
                return Ok(());
            }
        }
        
        // Add source line
        let mut content = if shell_config.exists() {
            fs::read_to_string(&shell_config)?
        } else {
            String::new()
        };
        
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        
        content.push_str(&format!("\n# zshrcman environment\n{}\n", source_line));
        
        fs::write(&shell_config, content)?;
        Ok(())
    }
    
    fn get_shell_config_path(&self) -> Result<PathBuf> {
        let home = env::var("HOME").unwrap_or_else(|_| {
            env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
        });
        
        let config_file = match self.shell_type {
            ShellType::Zsh => ".zshrc",
            ShellType::Bash => ".bashrc",
            ShellType::Fish => ".config/fish/config.fish",
            ShellType::PowerShell => {
                if cfg!(windows) {
                    "Documents/PowerShell/Microsoft.PowerShell_profile.ps1"
                } else {
                    ".config/powershell/profile.ps1"
                }
            }
            ShellType::Cmd => "zshrcman_env.bat",
        };
        
        Ok(PathBuf::from(home).join(config_file))
    }
}