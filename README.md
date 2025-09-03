# zshrcman

A powerful Rust-based Zsh/dotfiles manager with Git integration for managing configuration across multiple devices.

## Features

- **Git-backed Configuration**: Store your dotfiles in a Git repository with device-specific branches
- **Group Management**: Organize configurations into logical groups (brew, npm, pnpm, aliases, ssh, zshrc)
- **Device-specific Configurations**: Maintain separate configurations for different devices
- **Interactive Setup**: User-friendly prompts for initialization and configuration
- **Alias Management**: Easily manage shell aliases with active/inactive states
- **Package Management**: Integrate with Homebrew, npm, and pnpm for consistent package installation
- **SSH Key Management**: Deploy SSH keys across devices
- **Typo Protection**: Jaro-Winkler string similarity checking to prevent configuration mistakes

## Installation

### Prerequisites

- Rust 1.70+ and Cargo
- Git
- SSH key configured for your Git repository
- Optional: Homebrew, npm, pnpm (for package management features)

### Build from Source

```bash
git clone https://github.com/yourusername/zshrcman.git
cd zshrcman
cargo build --release
sudo cp target/release/zshrcman /usr/local/bin/
```

## Quick Start

### 1. Initialize zshrcman

```bash
zshrcman init
```

This will:
- Prompt for your Git repository URL
- Clone or create the repository
- Create/select a device-specific branch
- Set up initial groups (default, brew, npm, pnpm, aliases, ssh, zshrc)
- Configure active aliases for enabled groups

### 2. Install Configurations

```bash
# Interactive installation (prompts for each group)
zshrcman install

# Install all groups without prompting
zshrcman install --all
```

### 3. Sync with Remote Repository

```bash
zshrcman sync
```

## Directory Structure

Your dotfiles repository will be organized as follows:

```
~/.local/share/zshrcman/dotfiles/
├── config.toml          # Main configuration file
├── groups/               # Global groups
│   ├── default.toml
│   ├── brew.toml
│   ├── npm.toml
│   ├── pnpm.toml
│   ├── aliases.toml
│   ├── ssh.toml
│   └── zshrc.toml
└── devices/              # Device-specific configurations
    └── <device-name>/
        ├── .zshrc
        └── groups/
            └── <custom>.toml
```

## Commands

### Core Commands

```bash
zshrcman init [--force]           # Initialize zshrcman
zshrcman install [--all]          # Install configured groups
zshrcman remove-all               # Uninstall all groups
zshrcman sync [--force]           # Sync with remote repository
zshrcman status                   # Show current configuration status
```

### Group Management

```bash
zshrcman group list               # List all global groups
zshrcman group add <name>         # Add a new global group
zshrcman group remove <name>      # Remove a global group
zshrcman group enable <name>      # Enable a global group
zshrcman group disable <name>     # Disable a global group
```

### Device Group Management

```bash
zshrcman device list              # List per-device groups
zshrcman device add <name>        # Add a device-specific group
zshrcman device remove <name>     # Remove a device-specific group
zshrcman device enable <name>     # Enable a device group
zshrcman device disable <name>    # Disable a device group
```

### Alias Management

```bash
zshrcman alias list [group]       # List aliases (all or by group)
zshrcman alias add <group> "<alias>"      # Add an alias to a group
zshrcman alias remove <group> "<alias>"   # Remove an alias from a group
zshrcman alias toggle <group>     # Toggle active/inactive aliases
```

## Group Configuration Format

Each group is defined in a TOML file with the following structure:

```toml
name = "example"
description = "Example group configuration"
packages = ["package1", "package2"]  # For brew/npm/pnpm groups
aliases = [
    'alias ll="ls -la"',
    'alias gs="git status"'
]
scripts = ["script1.sh", "script2.sh"]  # For zshrc group
files = [
    { source = "config/example.conf", target = "~/.example.conf" }
]
ssh_keys = ["id_rsa", "id_ed25519"]  # For ssh group
```

## Configuration File

The main configuration file (`~/.config/zshrcman/config.toml`) contains:

```toml
[repository]
url = "git@github.com:username/dotfiles.git"
main_branch = "main"
dotfiles_path = "~/.local/share/zshrcman/dotfiles"

[device]
name = "laptop"
branch = "device/laptop"

[groups]
global = ["default", "brew", "npm"]
per_device = ["work-tools"]
enabled_global = ["default", "brew"]
enabled_devices = ["work-tools"]

[aliases.default]
items = ['alias ll="ls -la"', 'alias gs="git status"']
active = ['alias ll="ls -la"']

[status.default]
installed = true
success = true
timestamp = "2024-01-01T12:00:00Z"
```

## Advanced Usage

### Creating Custom Groups

1. Create a new group file in `groups/` or `devices/<device>/groups/`
2. Define the group configuration in TOML format
3. Add the group using `zshrcman group add <name>`
4. Enable the group using `zshrcman group enable <name>`

### Managing Multiple Devices

1. Each device gets its own branch (e.g., `device/laptop`, `device/desktop`)
2. Device-specific configurations override global ones
3. Use `zshrcman sync` to keep devices synchronized with the main branch

### Extending Functionality

The architecture supports future extensions for:
- Homebrew formula version pinning
- Node.js/Python version management (nvm, pyenv)
- Custom installer scripts
- Configuration templating
- Encrypted secrets management

## Troubleshooting

### SSH Key Issues
Ensure your SSH agent is running and has your key loaded:
```bash
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_rsa
```

### Git Authentication
zshrcman uses SSH keys from your SSH agent. Ensure your Git remote uses SSH URLs:
```bash
git@github.com:username/dotfiles.git  # Correct
https://github.com/username/dotfiles.git  # Will need conversion
```

### Permission Issues
Ensure proper permissions on configuration directories:
```bash
chmod 700 ~/.config/zshrcman
chmod 700 ~/.local/share/zshrcman
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - See LICENSE file for details

## Author

Created with zshrcman - Your friendly Zsh configuration manager 🦀