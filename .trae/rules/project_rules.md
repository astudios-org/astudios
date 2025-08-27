# TRAE Project Rules

This file provides guidance to TRAE AI when working with code in this repository.

## Project Overview

as-man is a CLI tool for managing Android Studio installations, inspired by xcodes. It's written in Rust and provides functionality to list, download, install, and switch between Android Studio versions.

## Build and Development Commands

### Building
```bash
# Build in debug mode
cargo build

# Build optimized release version
cargo build --release

# Run the application
cargo run -- <command>
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Check for unused dependencies
cargo machete
```

## Architecture Overview

### Module Structure
The codebase follows a clean modular architecture with clear separation of concerns:

- **`main.rs`**: Application entry point with centralized error handling
- **`cli.rs`**: Command-line interface definitions using clap
- **`commands.rs`**: Command handlers that orchestrate business logic
- **`error.rs`**: Comprehensive custom error types with user-friendly messages
- **`config.rs`**: Centralized configuration and constants
- **`api.rs`**: HTTP client for JetBrains API interactions
- **`list.rs`**: Release listing and caching functionality
- **`downloader.rs`**: Download management with multiple backend support
- **`installer.rs`**: Installation and version management
- **`model.rs`**: Data structures and domain models
- **`progress.rs`**: Unified progress reporting system

### Key Design Patterns

1. **Error Handling**: Uses a comprehensive `AsManError` enum with specific variants for different error scenarios. All errors provide user-friendly messages with actionable context.

2. **Progress Reporting**: Unified progress system using `indicatif` with configurable display (can be disabled). Supports both spinner and progress bar modes.

3. **Downloader Abstraction**: Supports multiple download backends (reqwest built-in, aria2 high-performance) with automatic detection and fallback.

4. **Configuration Management**: All constants, paths, and timeouts centralized in `config.rs` with platform-specific handling.

## Logging and Debugging Practices

### Current Logging Approach
The project currently uses **direct console output** rather than structured logging:

- **User Output**: Uses `println!` with colored formatting via the `colored` crate
- **Error Output**: Uses `eprintln!` in main.rs error handler with categorized error messages
- **Progress Feedback**: Uses `indicatif` progress bars and spinners for visual feedback

### Error Handling and Debugging
- **Comprehensive Error Types**: Each module has specific error variants in `AsManError` enum
- **Contextual Error Messages**: Errors include user-friendly descriptions and suggested actions
- **Error Categorization**: Errors are categorized (Network, Download, Installation, etc.) for better user experience
- **Progress Reporting**: Visual feedback during long-running operations helps identify where issues occur

### Debugging Workflow
1. **Error Messages**: Check the specific error variant and message in main.rs error handler
2. **Progress Indicators**: Use progress reporting to identify which operation is failing
3. **File System State**: Check installation directories and cache locations defined in config.rs
4. **Network Issues**: Network errors provide specific reqwest error details

### Adding Debug Information
When adding debug capabilities:
- Consider adding a `--verbose` flag to CLI for detailed output
- Use `eprintln!` for debug information to stderr
- Leverage the existing progress reporting system for operation status
- Add debug information to error contexts rather than separate logging

## Development Guidelines

### Code Style
- Follow standard Rust conventions and use `cargo fmt`
- Use meaningful function and variable names
- Keep functions focused and small
- Add comprehensive documentation for public APIs

### Error Handling
- Always use the `AsManError` enum for application errors
- Provide user-friendly error messages with actionable context
- Use appropriate error variants for different scenarios
- Include recovery suggestions in error messages

### Testing Strategy
- Write unit tests for pure functions
- Use integration tests for complex workflows
- Mock external dependencies (network, file system)
- Test error scenarios and edge cases

### Dependencies
Key dependencies and their purposes:
- **clap**: Command-line argument parsing with derive macros
- **reqwest**: HTTP client for API calls and downloads
- **indicatif**: Progress bars and spinners
- **colored**: Terminal color output
- **serde/serde_json**: JSON serialization
- **quick-xml**: XML parsing for JetBrains API responses
- **zip/tar/flate2**: Archive extraction support

### Platform Support
The application supports macOS, Windows, and Linux with platform-specific:
- Installation directories
- Archive formats (DMG for macOS, ZIP for Windows, TAR.GZ for Linux)
- Path handling and executable detection

## Common Development Tasks

### Adding New Commands
1. Add command variant to `cli.rs` Commands enum
2. Add handler method in `commands.rs` CommandHandler
3. Update main command dispatcher in `CommandHandler::handle`
4. Add appropriate error handling and user feedback

### Adding New Error Types
1. Add variant to `AsManError` enum in `error.rs`
2. Implement Display formatting with user-friendly message
3. Add From trait implementations if needed
4. Update main.rs error handler for specific handling

### Modifying Download/Installation Logic
1. Update relevant modules (`downloader.rs`, `installer.rs`)
2. Ensure progress reporting is maintained
3. Add appropriate error handling
4. Test with different platforms and archive formats

### Configuration Changes
1. Update constants in `config.rs`
2. Ensure platform-specific handling is maintained
3. Update documentation if user-facing settings change

## File Locations and Conventions

### Important Paths (defined in config.rs)
- **Cache Directory**: `~/.cache/as-man/` (Linux/macOS) or `%LOCALAPPDATA%/as-man/` (Windows)
- **Versions Directory**: `~/.local/share/as-man/versions/`
- **Applications Directory**: `/Applications` (macOS), `C:\Program Files` (Windows)
- **Download Directory**: User's Downloads folder by default

### Archive Handling
- **macOS**: DMG files (mounted and copied)
- **Windows**: ZIP files (extracted)
- **Linux**: TAR.GZ files (extracted)

### Version Management
- Versions are stored in separate directories under versions directory
- Symlinks are used for version switching (macOS/Linux)
- Registry entries for Windows version management