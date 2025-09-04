use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    
    #[serde(default)]
    pub active_profile: Option<String>,
    
    #[serde(default)]
    pub installations: HashMap<String, InstallationRecord>,
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
            profiles: HashMap::new(),
            active_profile: None,
            installations: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub parent: Option<String>,
    pub packages: HashSet<String>,
    pub environment: EnvironmentState,
    pub os_overrides: HashMap<OsType, ProfileOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileOverride {
    pub packages: Vec<String>,
    pub environment: Option<EnvironmentState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationRecord {
    pub package: String,
    pub version: Option<String>,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub installed_by: InstallationSource,
    pub active_for: HashSet<String>,
    pub scope: InstallScope,
    pub location: Option<PathBuf>,
    pub installer_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationSource {
    Profile(String),
    Global,
    System,
    Manual,
    Dependency(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallScope {
    System,
    Global,
    Profile,
    Local,
    Device,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub paths_prepend: Vec<String>,
    pub paths_append: Vec<String>,
    pub variables: HashMap<String, String>,
    pub aliases: HashMap<String, String>,
    pub active: bool,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            paths_prepend: Vec::new(),
            paths_append: Vec::new(),
            variables: HashMap::new(),
            aliases: HashMap::new(),
            active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OsType {
    MacOS,
    Windows,
    Linux,
    Universal,
}

impl OsType {
    pub fn detect() -> Self {
        if cfg!(target_os = "macos") {
            OsType::MacOS
        } else if cfg!(target_os = "windows") {
            OsType::Windows
        } else if cfg!(target_os = "linux") {
            OsType::Linux
        } else {
            OsType::Universal
        }
    }
}

#[derive(Debug, Clone)]
pub enum RemovalStrategy {
    Deactivate,
    RemoveFromProfile,
    SmartRemove,
    ForceRemove,
    MarkUnused,
}