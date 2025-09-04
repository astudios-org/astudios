# Architecture Documentation

## Overview

This document describes the architecture and design decisions of the astudios (Android Studio Manager) codebase.

## Architecture Goals

- **Modularity**: Clear separation of concerns with well-defined module boundaries
- **Error Handling**: Comprehensive error handling with custom error types
- **Configuration**: Centralized configuration management
- **Progress Reporting**: Unified progress reporting system
- **Maintainability**: Clean, readable, and testable code
- **Performance**: Efficient download and installation processes

## Module Structure

```
src/
├── api.rs          # HTTP client for JetBrains API
├── cli.rs          # Command-line interface definitions
├── commands.rs     # Command handlers
├── config.rs       # Application configuration
├── downloader.rs   # Download management (reqwest/aria2)
├── error.rs        # Custom error types
├── installer.rs    # Installation management
├── lib.rs          # Library root
├── list.rs         # Release listing and caching
├── model.rs        # Data models and structures
├── progress.rs     # Progress reporting utilities
└── main.rs         # Application entry point
```

## Key Design Decisions

### 1. Error Handling

The codebase now uses a comprehensive `AstudiosError` enum:

- **Type Safety**: Each error type is explicitly handled
- **User Experience**: Clear, actionable error messages
- **Debugging**: Detailed error context for developers

### 2. Configuration Management

All configuration is centralized in `config.rs`:

- **Constants**: Application-wide constants and defaults
- **Paths**: System-specific directory paths
- **Timeouts**: Network and download timeouts
- **Platform Detection**: Cross-platform path handling

### 3. Progress Reporting

The `progress.rs` module provides:

- **Unified Interface**: Consistent progress reporting across all operations
- **Flexible Display**: Configurable progress display (enabled/disabled)
- **Multi-step Support**: Progress steps for complex operations

### 4. Downloader Abstraction

The `downloader.rs` module supports:

- **Multiple Backends**: reqwest (built-in) and aria2 (high-performance)
- **Auto-detection**: Automatic selection of best available downloader
- **Progress Integration**: Seamless progress reporting

### 5. Installation Management

The `installer.rs` module provides:

- **Multi-step Process**: Clear installation stages
- **Platform Support**: macOS, Windows, and Linux support
- **Archive Handling**: Support for ZIP, TAR, TAR.GZ, and DMG formats
- **Cleanup**: Automatic cleanup of temporary files
- **Verification**: Installation integrity verification

## Data Models

### AndroidStudio

Represents a single Android Studio release with:
- Version information
- Build details
- Release channel
- Platform-specific downloads

### Download

Contains download information for a specific platform:
- URL
- File size
- Checksum

### ReleaseChannel

Enum representing different release channels:
- Release
- Beta
- Canary
- Release Candidate
- Patch

## Error Handling Strategy

Each module has specific error types:

- **AstudiosError**: Main error enum with variants for all error scenarios
- **Contextual Messages**: User-friendly error messages with context
- **Recovery Suggestions**: Actionable next steps for users

## Performance Optimizations

### 1. Caching

- **Release Cache**: 24-hour cache for release information
- **Smart Refresh**: Only fetch new data when cache expires

### 2. Parallel Downloads

- **aria2 Integration**: 16 concurrent connections when available
- **Fallback**: Graceful fallback to reqwest when aria2 unavailable

### 3. Efficient Archive Handling

- **Streaming**: Stream-based archive extraction
- **Memory Usage**: Optimized memory usage during extraction

## Testing Strategy

The refactored codebase is designed for testability:

- **Pure Functions**: Most functions are pure and easily testable
- **Dependency Injection**: Configurable dependencies for testing
- **Mock Support**: Easy to mock external dependencies

## Future Improvements

### 1. Configuration Files
- User configuration files
- Environment-specific settings

### 2. Logging
- Structured logging
- Debug/verbose modes

### 3. Validation
- Checksum verification
- Signature validation

## Usage Examples

### Basic Usage
```bash
# List available versions
astudios list

# Install latest stable
astudios install --latest

# Download specific version
astudios download "2023.3.1"

# Switch versions
astudios use "2023.3.1"
```

### Advanced Usage
```bash
# Install with custom directory
astudios install "2023.3.1" --directory /Applications/Custom

# Download with specific downloader
astudios download "2023.3.1" --downloader aria2

# List only canary versions
astudios list --canary
```

## Development Guidelines

### Code Style
- Use meaningful function and variable names
- Add comprehensive documentation
- Follow Rust best practices
- Keep functions small and focused

### Error Handling
- Always provide user-friendly error messages
- Include context for debugging
- Use appropriate error variants

### Testing
- Write unit tests for pure functions
- Use integration tests for complex workflows
- Mock external dependencies
