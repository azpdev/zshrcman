# zshrcman Tech Stack Analysis
**Analysis Date**: 2025-09-03
**Project Type**: Rust-based Zsh/dotfiles Manager with Git Integration

## Project Architecture
- **Type**: Command-line Configuration Management System
- **Language**: Rust (2021 edition)
- **Purpose**: Cross-device shell configuration and dotfiles synchronization

## Core Technology Stack

### Rust Dependencies
- **CLI Framework**: clap 4.5 (derive macros for command parsing)
- **Git Operations**: git2 0.18 (libgit2 Rust bindings)
- **Serialization**: serde 1.0 + toml 0.8 (configuration persistence)
- **Interactive UI**: dialoguer 0.11 (user prompts and selection)
- **Path Management**: directories 5.0 (cross-platform path resolution)
- **Error Handling**: anyhow 1.0 (error chaining and context)
- **String Matching**: strsim 0.11 (Jaro-Winkler typo detection)
- **Logging**: tracing 0.1 + tracing-subscriber 0.3
- **Time**: chrono 0.4 (timestamp tracking)
- **Terminal Colors**: colored 2.1

### Testing Dependencies
- **Temp Files**: tempfile 3.10 (test isolation)
- **Mocking**: mockall 0.12 (mock objects)
- **CLI Testing**: assert_cmd 2.0 + predicates 3.1

## Architecture Patterns

### Design Patterns
- **Repository Pattern**: Git-backed configuration storage
- **Strategy Pattern**: Pluggable package manager installers
- **Command Pattern**: CLI command encapsulation
- **Manager Pattern**: Domain-specific managers for different concerns

### Module Structure
- **models.rs**: Serde data models (Config, Repository, Device, Groups)
- **modules/config.rs**: Configuration persistence and management
- **modules/git_mgr.rs**: Git operations wrapper around libgit2
- **modules/init.rs**: Interactive initialization flow
- **modules/install.rs**: Package manager integration (brew/npm/pnpm)
- **modules/alias.rs**: Shell alias management

## Key Features
- Multi-device configuration with Git branches
- Group-based organization (global + device-specific)
- Interactive setup with dialoguer prompts
- Package manager integration (Homebrew, npm, pnpm)
- SSH key deployment automation
- Typo protection with string similarity
- Installation status tracking for rollbacks

## Storage Architecture
- **Config Path**: `~/.config/zshrcman/config.toml`
- **Dotfiles Path**: `~/.local/share/zshrcman/dotfiles/`
- **Git Structure**: Device branches + main branch synchronization
- **Group Files**: TOML-based group configurations

## Integration Points
- **SSH Agent**: For Git authentication
- **Package Managers**: brew, npm, pnpm command execution
- **Shell Files**: `.zshrc`, `.zsh_aliases` modification
- **Git Remote**: SSH-based repository synchronization

## Development Workflow
- Rust standard toolchain (cargo build/test/clippy/fmt)
- Git-based version control
- Error handling with anyhow Result types
- Interactive CLI testing with assert_cmd