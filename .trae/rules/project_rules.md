# TRAE Project Rules

This file provides guidance to TRAE AI when working with code in this repository.

## Project Overview

astudios is a CLI tool for managing Android Studio installations on macOS, written in Rust. It's inspired by xcodes and provides functionality to list, download, install, and switch between Android Studio versions.

## Development Commands

### Building and Testing
```bash
# Build the project
cargo build --release

# Run all tests
cargo test --verbose

# Run tests with documentation
cargo test --doc

# Check code without building
cargo check --verbose

# Format code
cargo fmt --all

# Run linting
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --no-deps --document-private-items

# Security audit
cargo audit
```

### Running the Application
```bash
# Build and run locally
cargo run -- --help

# Install from source
cargo build --release
# Binary will be in target/release/astudios
```

## Architecture Overview

### Module Structure
The codebase follows a modular architecture with clear separation of concerns:

- `src/lib.rs` - Library root exposing public modules
- `src/main.rs` - Application entry point with centralized error handling
- `src/cli.rs` - Command-line interface definitions using clap
- `src/commands.rs` - Command handlers that orchestrate business logic
- `src/error.rs` - Comprehensive custom error types with user-friendly messages
- `src/config.rs` - Centralized configuration constants and utilities
- `src/progress.rs` - Unified progress reporting system using indicatif
- `src/api.rs` - HTTP client for JetBrains API interactions
- `src/model.rs` - Data models and structures for Android Studio releases
- `src/downloader.rs` - Download management supporting both reqwest and aria2
- `src/installer.rs` - Installation management with multi-platform support
- `src/list.rs` - Release listing and caching functionality
- `src/detector.rs` - System detection and prerequisite checking

### Key Design Patterns

#### Error Handling Strategy
The project uses a comprehensive `AstudiosError` enum in `src/error.rs` with:
- Type-safe error variants for different failure scenarios
- User-friendly error messages with actionable suggestions
- Automatic conversion from standard library errors via `From` implementations
- Centralized error handling in `main.rs` with specific user guidance

#### Progress Reporting
The `ProgressReporter` in `src/progress.rs` provides:
- Unified interface for all progress operations
- Configurable display (can be disabled for scripting)
- Support for both spinners and progress bars
- Multi-step progress tracking via `ProgressSteps`

#### Configuration Management
All configuration is centralized in `src/config.rs`:
- Application constants (timeouts, paths, API endpoints)
- Platform-specific defaults
- User directory management
- Network and download settings

## Logging and Debugging Practices

### Current Logging Approach
The project currently uses:
- **Standard output**: `println!` for normal user-facing information
- **Error output**: `eprintln!` for error messages in main.rs
- **Progress reporting**: indicatif progress bars and spinners for visual feedback
- **Colored output**: colored crate for enhanced terminal output

### Error Context and Debugging
- All errors include contextual information and user-friendly messages
- Error variants are categorized by operation type (Network, Download, Installation, etc.)
- Main error handler provides specific guidance for each error type
- Use `RUST_BACKTRACE=1` environment variable for detailed stack traces during debugging

### Testing and Debugging
- Snapshot testing with `insta` crate for CLI output validation
- Integration tests using `assert_cmd` for command-line testing
- Unit tests for individual modules and error handling
- No structured logging framework currently implemented

## Testing Strategy

### Test Organization
- `tests/cli_tests.rs` - Command-line interface integration tests
- `tests/error_tests.rs` - Error handling and conversion tests  
- `tests/model_tests.rs` - Data model serialization and validation tests
- `tests/snapshots/` - Insta snapshot files for CLI output validation

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test cli_tests

# Update snapshots (when CLI output changes)
cargo insta review

# Run tests with output
cargo test -- --nocapture
```

## Platform-Specific Considerations

### macOS Focus
- Primary target platform is macOS with specific DMG handling
- Uses macOS-specific tools: `hdiutil`, `codesign`, `cp`, `rm`
- Application bundle management for .app installations
- Homebrew integration for aria2 downloader detection

### Cross-Platform Architecture
- Designed for future Windows/Linux support
- Platform detection in config.rs
- Abstracted installation logic in installer.rs

## Dependencies and External Tools

### Required Dependencies
- `clap` - Command-line argument parsing
- `reqwest` - HTTP client for downloads
- `indicatif` - Progress bars and spinners
- `colored` - Terminal color output
- `serde`/`serde_json` - JSON serialization
- `quick-xml` - XML parsing for JetBrains API

### Optional External Tools
- `aria2c` - High-performance downloader (auto-detected)
- Homebrew paths: `/usr/local/bin/aria2c`, `/opt/homebrew/bin/aria2c`

## Code Style and Conventions

### Error Handling
- Always use `AstudiosError` for application errors
- Provide user-friendly error messages with context
- Include actionable suggestions in error output
- Use `?` operator for error propagation

### Progress Reporting
- Use `ProgressReporter` for all long-running operations
- Provide meaningful progress messages
- Support both enabled/disabled modes for scripting

### Module Organization
- Keep modules focused on single responsibilities
- Use public interfaces between modules
- Centralize configuration in config.rs
- Abstract external dependencies behind traits when possible

## Future Architecture Considerations

The ARCHITECTURE.md mentions planned improvements:
- Structured logging framework
- Configuration files for user settings
- Checksum verification for downloads
- Enhanced validation and signature checking