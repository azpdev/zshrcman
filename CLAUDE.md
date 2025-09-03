# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Testing
```bash
# Build debug version
cargo build

# Build release version  
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Run clippy with all targets
cargo clippy --all-targets --all-features
```

### Running the Application
```bash
# Run from source (debug)
cargo run -- init

# Run built binary
./target/release/zshrcman status

# Install system-wide
sudo cp target/release/zshrcman /usr/local/bin/
```

## Architecture Overview

zshrcman is a layered Rust application for managing dotfiles across multiple devices using Git synchronization.

### Core Architecture Patterns
- **Repository Pattern**: Git-backed configuration storage in `~/.local/share/zshrcman/dotfiles/`
- **Strategy Pattern**: Pluggable installers for different package managers (brew, npm, pnpm, etc.)
- **Command Pattern**: CLI commands handled through clap derive macros
- **Manager Pattern**: Each domain has a dedicated manager class

### Key Architectural Concepts

**Multi-Device Configuration**: Each device gets its own Git branch (`device/<name>`) that inherits from main branch but can have device-specific overrides.

**Group-Based Organization**: Configurations are organized into groups (global and device-specific):
- Global groups live in `groups/<name>.toml`
- Device groups live in `devices/<device>/groups/<name>.toml`
- Built-in groups: default, brew, npm, pnpm, aliases, ssh, zshrc

**State Management**: Configuration persisted in `~/.config/zshrcman/config.toml` with installation status tracking for rollback capabilities.

### Module Responsibilities

**`src/models.rs`**: Serde-based data structures defining the core domain models (Config, Repository, Device, Groups, etc.)

**`src/modules/config.rs`**: ConfigManager handles TOML persistence, group management, and path resolution using directories crate.

**`src/modules/git_mgr.rs`**: GitManager wraps libgit2 for repository operations, branch management, and SSH-based authentication.

**`src/modules/init.rs`**: InitManager orchestrates first-time setup using dialoguer for interactive prompts.

**`src/modules/install.rs`**: InstallManager implements the strategy pattern for different installer types, with status tracking and rollback.

**`src/modules/alias.rs`**: AliasManager handles shell alias CRUD operations with active/inactive state management.

### Data Flow Patterns

**Configuration Loading**: ConfigManager loads from TOML → deserializes to structs → passes to appropriate managers
**Git Operations**: All Git operations go through GitManager with SSH agent authentication
**Installation Flow**: InstallManager dispatches to specific installers based on group type, tracks status
**Device Synchronization**: Git branches keep device configs in sync while preserving local customizations

### Extension Points

**Adding Package Managers**: Extend `InstallerType` enum and implement install/uninstall methods in `InstallManager`
**Custom Group Types**: Create new TOML group configurations and register custom installer logic
**Device-Specific Logic**: Add device-specific groups in `devices/<device>/groups/` directory

### Critical Integration Dependencies
- **git2**: Low-level Git operations requiring SSH agent setup
- **dialoguer**: Interactive CLI prompts for user configuration
- **directories**: Cross-platform path resolution for config/data directories
- **serde + toml**: Configuration serialization format
- **clap**: Command-line argument parsing with derive macros

### Error Handling Strategy
Uses `anyhow::Result` throughout for error chaining. Git operations can fail due to network/auth issues. File operations can fail due to permissions. All operations designed to be idempotent where possible.