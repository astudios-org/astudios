# Rust/astudios

This is a Rust CLI tool for managing Android Studio installations on macOS, inspired by xcodes.

## Project Structure

- Single crate project named `astudios`
- Source code in `src/` directory with modular organization:
  - `api.rs` - Android Studio API interactions
  - `cli.rs` - Command-line interface definitions
  - `commands.rs` - Command implementations
  - `config.rs` - Configuration management
  - `detector.rs` - Platform and environment detection
  - `downloader.rs` - Download functionality
  - `error.rs` - Custom error types
  - `installer.rs` - Installation logic
  - `list.rs` - Version listing functionality
  - `model.rs` - Data models and structures
  - `progress.rs` - Progress reporting

## Code Conventions

- When using `format!` and you can inline variables into `{}`, always do that.
- Use comprehensive error handling with the custom `AstudiosError` enum defined in `src/error.rs`
- Follow the existing module structure and naming conventions
- Use `serde` for serialization/deserialization with appropriate derive macros
- Prefer descriptive variable names and comprehensive documentation comments

## Build and Test Commands

Run `cargo fmt` automatically after making Rust code changes; do not ask for approval to run it.

Before finalizing changes, run the following commands to ensure code quality:
1. **Format code**: `cargo fmt --all`
2. **Check code**: `cargo check --verbose`
3. **Run linter**: `cargo clippy --all-targets --all-features -- -D warnings`
4. **Run tests**: `cargo test --verbose`

When running interactively, ask the user before running the full test suite. Individual tests and formatting can be run without asking.

## Testing

### Snapshot Tests

This project uses snapshot tests via `insta` to validate CLI output and error messages. When CLI output or error messages change intentionally, update the snapshots as follows:

- Run tests to generate updated snapshots:
  - `cargo test`
- Check what's pending:
  - `cargo insta pending-snapshots`
- Review changes by reading the generated `*.snap.new` files directly in the repo, or preview a specific file:
  - `cargo insta show path/to/file.snap.new`
- Only if you intend to accept all new snapshots, run:
  - `cargo insta accept`

If you don't have the tool:
- `cargo install cargo-insta`

### Test Assertions

- Tests should use `assert_cmd` for CLI testing and `predicates` for output validation
- Use `tempfile` for creating temporary directories and files in tests
- Use `strip-ansi-escapes` to handle colored output in tests

## Minimum Supported Rust Version (MSRV)

- **Rust 1.89.0** (as specified in `rust-toolchain.toml`)
- **Edition 2024** (as specified in `Cargo.toml`)

Always ensure your changes pass all CI checks before submitting.