use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub repository: Repository,
    
    #[serde(default)]
    pub device: Device,
    
    #[serde(default)]
    pub groups: Groups,
    
    #[serde(default)]
    pub aliases: HashMap<String, AliasGroup>,
    
    #[serde(default)]
    pub status: HashMap<String, InstallStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Repository {
    pub url: Option<String>,
    pub main_branch: String,
    pub dotfiles_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Device {
    pub name: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Groups {
    pub global: Vec<String>,
    pub per_device: Vec<String>,
    pub enabled_global: Vec<String>,
    pub enabled_devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasGroup {
    pub items: Vec<String>,
    pub active: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallStatus {
    pub installed: bool,
    pub success: bool,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub files: Vec<FileMapping>,
    #[serde(default)]
    pub ssh_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Clone)]
pub enum InstallerType {
    Brew,
    Npm,
    Pnpm,
    Aliases,
    Ssh,
    Zshrc,
    Custom(String),
}

impl InstallerType {
    pub fn from_group_name(name: &str) -> Self {
        match name {
            "brew" => Self::Brew,
            "npm" => Self::Npm,
            "pnpm" => Self::Pnpm,
            "aliases" => Self::Aliases,
            "ssh" => Self::Ssh,
            "zshrc" => Self::Zshrc,
            _ => Self::Custom(name.to_string()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repository: Repository {
                url: None,
                main_branch: "main".to_string(),
                dotfiles_path: PathBuf::from("~/.local/share/zshrcman/dotfiles"),
            },
            device: Device::default(),
            groups: Groups {
                global: vec!["default".to_string()],
                per_device: vec![],
                enabled_global: vec!["default".to_string()],
                enabled_devices: vec![],
            },
            aliases: HashMap::new(),
            status: HashMap::new(),
        }
    }
}