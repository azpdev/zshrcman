# Tech Stack Analysis - zshrcman
**Generated**: 2025-09-04
**Project**: Dotfiles Management System with Multi-Device Git Sync

## 🦀 Core Technologies

### Language & Runtime
- **Rust** (Edition 2021) - Systems programming with memory safety
- **Cargo** - Build system and package management

### Architecture Patterns
- **Repository Pattern** - Git-backed configuration storage
- **Strategy Pattern** - Pluggable installer system (brew, npm, pnpm)
- **Command Pattern** - CLI through clap derive macros
- **Manager Pattern** - Domain-specific managers for each concern

## 📦 Key Dependencies

### CLI & User Interface
- **clap** (4.5) - Command-line argument parsing with derive macros
- **dialoguer** (0.11) - Interactive terminal prompts for setup flows
- **colored** (2.1) - Terminal output colorization

### Data & Configuration
- **serde** (1.0) - Serialization/deserialization framework
- **toml** (0.8) - TOML configuration file parsing
- **chrono** (0.4) - Date/time handling with serde support

### Git Operations
- **git2** (0.18) - libgit2 bindings for Git repository management
- SSH authentication support through system SSH agent

### System Integration
- **directories** (5.0) - Cross-platform standard directory paths
- **anyhow** (1.0) - Error handling with context chaining

### Observability
- **tracing** (0.1) - Structured logging framework
- **tracing-subscriber** (0.3) - Log formatting and filtering

### Utilities
- **strsim** (0.11) - String similarity for fuzzy matching

## 🏗️ System Architecture

### Module Structure
```
src/
├── models.rs          # Domain models (Config, Device, Groups)
├── main.rs           # Entry point & CLI setup
└── modules/
    ├── config.rs     # ConfigManager - TOML persistence
    ├── git_mgr.rs    # GitManager - Git operations & branching
    ├── init.rs       # InitManager - First-time setup
    ├── install.rs    # InstallManager - Package installation
    └── alias.rs      # AliasManager - Shell alias management
```

### Data Flow
1. **Configuration**: TOML → Serde Models → Managers
2. **Git Sync**: Local branches ↔ Remote repository (SSH)
3. **Installation**: Groups → Installer Strategy → System packages
4. **Device Management**: Main branch → Device branches (inheritance)

### Storage Locations
- **Config**: `~/.config/zshrcman/config.toml`
- **Repository**: `~/.local/share/zshrcman/dotfiles/`
- **Groups**: `groups/<name>.toml` (global) + `devices/<device>/groups/<name>.toml`

## 🎯 Key Architectural Decisions

### Multi-Device Strategy
- Each device gets its own Git branch (`device/<name>`)
- Inheritance from main branch with device-specific overrides
- Group-based configuration organization

### Error Handling
- `anyhow::Result` throughout for error context preservation
- Idempotent operations where possible
- Graceful handling of network/auth failures

### Extension Points
- New package managers via `InstallerType` enum
- Custom group types through TOML configurations
- Device-specific logic in isolated group directories

## 🔧 Development Tools

### Testing
- **tempfile** (3.10) - Temporary file/directory creation
- **mockall** (0.12) - Mock object generation for unit tests
- **assert_cmd** (2.0) - CLI integration testing
- **predicates** (3.1) - Assertion predicates for tests

### Build Commands
```bash
cargo build --release    # Optimized build
cargo test              # Run test suite
cargo clippy            # Lint with clippy
cargo fmt               # Format code
```

## 🚀 Performance Characteristics
- **Binary size**: Compiled Rust binary (~5-10MB release)
- **Memory**: Low footprint, no runtime/GC overhead
- **Startup**: Fast cold start (<50ms typical)
- **Git operations**: Network-bound (SSH authentication required)