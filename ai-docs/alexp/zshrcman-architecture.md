# zshrcman Architecture Documentation

**Project**: zshrcman - Rust-based Zsh/dotfiles manager  
**Date**: 2025-09-03  
**Architect**: Atlas (Systems Architecture Expert)

## Executive Summary

zshrcman is a comprehensive dotfiles management system built in Rust, designed to solve the complex problem of maintaining consistent shell configurations across multiple devices while preserving device-specific customizations. The architecture employs a **layered modular design** with clear separation of concerns and leverages Git for version control and synchronization.

## Architecture Overview

### Design Patterns

1. **Repository Pattern**: Centralized configuration storage with Git backend
2. **Strategy Pattern**: Pluggable installers for different package managers
3. **Command Pattern**: CLI commands encapsulated as discrete operations
4. **Factory Pattern**: Dynamic installer creation based on group types

### Core Principles

- **Modularity**: Each module has a single responsibility
- **Extensibility**: New installers and group types can be added without core changes
- **Device Independence**: Configurations portable across different machines
- **Version Control**: All changes tracked through Git
- **Type Safety**: Leveraging Rust's type system for reliability

## System Architecture

```
┌─────────────────────────────────────────────────┐
│                   CLI Layer                     │
│                  (main.rs)                      │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│              Command Handlers                   │
│         (Clap-based routing)                    │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│              Module Layer                       │
├──────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│  │  Config  │ │   Init   │ │ Install  │       │
│  │  Manager │ │ Manager  │ │ Manager  │       │
│  └──────────┘ └──────────┘ └──────────┘       │
│  ┌──────────┐ ┌──────────┐                    │
│  │   Alias  │ │   Git    │                    │
│  │  Manager │ │ Manager  │                    │
│  └──────────┘ └──────────┘                    │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│              Data Layer                         │
│         (models.rs - Serde)                     │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│            Storage Layer                        │
├──────────────────────────────────────────────────┤
│  ┌──────────────┐ ┌──────────────┐            │
│  │   TOML       │ │     Git      │            │
│  │   Files      │ │  Repository  │            │
│  └──────────────┘ └──────────────┘            │
└──────────────────────────────────────────────────┘
```

## Module Descriptions

### 1. Config Module (`modules/config.rs`)
**Responsibility**: Configuration persistence and management  
**Key Components**:
- `ConfigManager`: Central configuration state manager
- Path resolution for dotfiles and config directories
- Group enable/disable logic
- Installation status tracking

**Design Decisions**:
- Uses TOML for human-readable configuration
- Separates global and device-specific configurations
- Maintains installation state for idempotency

### 2. Git Manager Module (`modules/git_mgr.rs`)
**Responsibility**: Git operations and synchronization  
**Key Components**:
- Repository cloning and initialization
- Branch management (device branches)
- Commit/push automation
- Rebase/merge conflict resolution

**Design Decisions**:
- Uses libgit2 for native Git operations
- SSH key authentication via agent
- Automatic conflict resolution strategies

### 3. Init Module (`modules/init.rs`)
**Responsibility**: First-time setup and onboarding  
**Key Components**:
- Interactive setup flow
- Device branch creation
- Default group scaffolding
- Initial alias configuration

**Design Decisions**:
- Uses dialoguer for rich CLI interactions
- Scaffolds sensible defaults
- Non-destructive initialization

### 4. Install Module (`modules/install.rs`)
**Responsibility**: Package and configuration deployment  
**Key Components**:
- Installer strategy selection
- Package manager interfaces (brew, npm, pnpm)
- Alias file generation
- SSH key deployment
- Status tracking

**Design Decisions**:
- Pluggable installer architecture
- Rollback capability through status tracking
- Idempotent operations

### 5. Alias Module (`modules/alias.rs`)
**Responsibility**: Shell alias management  
**Key Components**:
- Alias CRUD operations
- Active/inactive state management
- Group-based organization
- Interactive selection

**Design Decisions**:
- Separates definition from activation
- Group-based namespacing
- Preserves user customizations

## Data Models

### Core Entities

```rust
Config {
    repository: Repository     // Git repository metadata
    device: Device            // Current device information
    groups: Groups            // Group configurations
    aliases: Map<String, AliasGroup>  // Alias definitions
    status: Map<String, InstallStatus> // Installation tracking
}

GroupConfig {
    name: String
    packages: Vec<String>     // Package lists
    aliases: Vec<String>      // Alias definitions
    scripts: Vec<String>      // Script references
    files: Vec<FileMapping>   // File deployments
    ssh_keys: Vec<String>     // SSH key names
}
```

### State Management

- **Persistent State**: Stored in `~/.config/zshrcman/config.toml`
- **Repository State**: Managed through Git in `~/.local/share/zshrcman/dotfiles/`
- **Runtime State**: Held in memory during command execution

## Security Considerations

1. **SSH Key Management**: 
   - Keys copied with 0600 permissions
   - Uses ssh-agent for authentication
   - No plaintext credential storage

2. **File Permissions**:
   - Config directories created with user-only access
   - Sensitive files protected during copy operations

3. **Git Operations**:
   - SSH-based authentication only
   - No password prompts or storage

## Extensibility Points

### Adding New Package Managers

1. Add variant to `InstallerType` enum
2. Implement install/uninstall methods in `InstallManager`
3. Create corresponding group configuration

### Custom Group Types

1. Define group TOML structure
2. Implement custom installer logic
3. Register in `InstallerType::from_group_name`

### Device-Specific Extensions

1. Create device-specific group in `devices/<device>/groups/`
2. Override global configurations as needed
3. Enable through device group commands

## Performance Characteristics

- **Initialization**: O(1) - Single repository clone
- **Installation**: O(n) - Linear with number of packages
- **Sync**: O(1) - Git fetch/rebase operations
- **Alias Operations**: O(1) - HashMap lookups

## Future Architecture Enhancements

1. **Plugin System**: Dynamic loading of custom installers
2. **Templating Engine**: Variable substitution in configurations
3. **Encryption Layer**: GPG-based secret management
4. **Cloud Sync**: Alternative to Git for non-technical users
5. **GUI Frontend**: Electron-based configuration interface
6. **Dependency Resolution**: Automatic ordering of group installations

## Testing Strategy

- **Unit Tests**: Module-level logic testing
- **Integration Tests**: Full command execution paths
- **Mock Git Repositories**: Testing without network dependencies
- **Fixture-based Testing**: Predefined configuration scenarios

## Deployment Architecture

```
Binary Distribution:
├── Single static binary (via musl)
├── Homebrew formula
├── Cargo installation
└── Docker image (for CI/CD)
```

## Conclusion

The zshrcman architecture exemplifies **clean architecture principles** with clear boundaries between layers, dependency injection for testability, and a plugin-oriented design for extensibility. The use of Rust provides memory safety and performance, while the Git backend ensures reliable synchronization across devices. This architecture scales from single-user personal use to team-wide configuration management scenarios.