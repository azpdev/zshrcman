# Profile-Based Installation Management

## Overview

The enhanced zshrcman now supports **stateful installation tracking** and **profile-based management** without the performance overhead of mass install/uninstall operations during profile switches.

## Key Concepts

### Installation State vs Active State

- **Installation State**: What's physically installed on the system
- **Active State**: What's currently accessible/configured for use  
- **Ownership**: Which profile/scope "owns" an installation

### Fast Profile Switching

Profile switching is now a **lightweight operation** (<1 second) that only:
1. Updates environment variables
2. Modifies PATH
3. Updates symlinks
4. Changes shell configuration

**No packages are installed or uninstalled during profile switches!**

## New Commands

### Profile Management

```bash
# Create a new profile
zshrcman profile create work
zshrcman profile create personal --parent work  # Inherit from work

# Switch profiles (fast, no reinstalls)
zshrcman profile switch work

# List all profiles
zshrcman profile list

# Show current profile
zshrcman profile current

# Activate without switching
zshrcman profile activate work

# Deactivate current profile
zshrcman profile deactivate

# Delete a profile
zshrcman profile delete personal
```

### Smart Installation

```bash
# Install package (or activate if already installed)
zshrcman install nodejs

# Install with specific scope
zshrcman install nodejs --scope global    # All profiles
zshrcman install nodejs --scope profile   # Current profile only
zshrcman install nodejs --scope local     # Current directory

# Force reinstall even if exists
zshrcman install nodejs --force
```

### Smart Removal

```bash
# Smart remove (checks usage across profiles)
zshrcman remove nodejs

# Just deactivate for current profile
zshrcman remove nodejs --deactivate

# Remove from profile list only
zshrcman remove nodejs --profile-only

# Force removal regardless of usage
zshrcman remove nodejs --force
```

## Architecture

### Installation Records

Each installed package is tracked with:
- Package name and version
- Installation timestamp
- Which profiles use it (reference counting)
- Installation scope (system/global/profile/local)
- Location on filesystem
- Installer type used

### Profile State

Each profile maintains:
- Set of active packages
- Environment variables
- PATH modifications
- Aliases
- OS-specific overrides

### State Persistence

```
~/.local/share/zshrcman/
├── state/
│   ├── installations.toml   # All installed packages
│   └── profiles/
│       ├── work.toml       # Work profile state
│       └── personal.toml   # Personal profile state
├── profiles/
│   ├── work/
│   │   └── bin/            # Symlinks to active binaries
│   └── personal/
│       └── bin/
└── env/
    └── profile.env         # Current profile environment
```

## Cross-Platform Support

### OS Detection

The system automatically detects the operating system:
- macOS
- Windows  
- Linux
- Universal (fallback)

### Platform-Specific Configurations

```toml
# In group configuration files
[packages.macos]
brew = ["iterm2", "rectangle"]

[packages.windows]
choco = ["windows-terminal", "powertoys"]

[packages.linux]
apt = ["terminator", "i3"]

[environment]
EDITOR = { macos = "code", windows = "code.exe", linux = "vim" }
```

## Environment Management

### Shell Support

Environment configurations are generated for:
- Zsh
- Bash
- Fish
- PowerShell
- CMD (Windows)

### Environment State

Each profile can modify:
- PATH (prepend/append)
- Environment variables
- Shell aliases
- Shell-specific scripts

## Performance Characteristics

### Profile Switching
- **Time**: <1 second
- **Operations**: Environment updates only
- **No installs/uninstalls**

### Smart Install
- **Detect existing**: O(1) hash lookup
- **Activation**: O(1) set insertion
- **New install**: Depends on package manager

### Reference Counting
- Packages shared across profiles
- Automatic cleanup when unused
- Safe removal with usage checking

## Migration from Old System

The new system is **backward compatible** with the existing device-based configuration:
- Existing device branches continue to work
- Devices can be converted to profiles
- Gradual migration supported

## Example Workflows

### Setting Up Work Environment

```bash
# Create work profile
zshrcman profile create work

# Switch to work profile
zshrcman profile switch work

# Install work tools
zshrcman install docker
zshrcman install kubectl
zshrcman install aws-cli

# Set work environment
zshrcman env set AWS_PROFILE work
zshrcman env set KUBECONFIG ~/.kube/work-config
```

### Quick Profile Switch

```bash
# Morning: Switch to work
zshrcman profile switch work
# Everything work-related is instantly available

# Evening: Switch to personal
zshrcman profile switch personal  
# Work tools hidden, personal tools active

# Weekend: Switch to opensource
zshrcman profile switch opensource
# Different environment for open source contributions
```

### Shared Tools

```bash
# Install git for all profiles
zshrcman profile switch work
zshrcman install git --scope global

# Switch to personal - git is still available
zshrcman profile switch personal
git --version  # Works!

# Remove git from personal only
zshrcman remove git --deactivate
# git still installed, just not active in personal profile
```

## Benefits

1. **Fast Switching**: <1 second profile changes
2. **Resource Efficient**: No duplicate installations
3. **Safe Operations**: Reference counting prevents accidental removals
4. **Cross-Platform**: Works on macOS, Windows, and Linux
5. **Flexible Scoping**: System, global, profile, or local installations
6. **Clean Separation**: Profiles are isolated but can share resources