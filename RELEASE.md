# Release Process

This document describes the automated release process for the astudios project using `cargo-release`.

## Overview

The project uses `cargo-release` to automate the release process, which includes:

- Version bumping
- Changelog generation
- Git tagging
- Publishing to crates.io
- Creating GitHub releases
- Cross-platform binary builds

## Prerequisites

Before creating a release, ensure you have:

1. **cargo-release installed**: `cargo install cargo-release`
2. **cargo-audit installed**: `cargo install cargo-audit`
3. **Clean working directory**: All changes committed
4. **Main branch**: Currently on the main branch
5. **GitHub token**: Set up for automated releases
6. **Crates.io token**: Set up for publishing

## Release Types

### Semantic Versioning

The project follows [Semantic Versioning](https://semver.org/):

- **Patch** (0.1.0 → 0.1.1): Bug fixes and small improvements
- **Minor** (0.1.0 → 0.2.0): New features, backward compatible
- **Major** (0.1.0 → 1.0.0): Breaking changes

### Pre-release Versions

- **Alpha** (0.1.0 → 0.1.1-alpha.1): Early development versions
- **Beta** (0.1.0 → 0.1.1-beta.1): Feature-complete, testing phase
- **RC** (0.1.0 → 0.1.1-rc.1): Release candidates

## Release Methods

### Method 1: Using the Release Script (Recommended)

The easiest way to create a release is using the provided script:

```bash
# Interactive mode - prompts for release type
./scripts/release.sh

# Direct mode - specify release type
./scripts/release.sh patch
./scripts/release.sh minor
./scripts/release.sh major
./scripts/release.sh alpha
./scripts/release.sh beta
./scripts/release.sh rc
```

### Method 2: Using cargo-release Directly

For more control, use `cargo-release` commands directly:

```bash
# Patch release
cargo release patch --execute

# Minor release
cargo release minor --execute

# Major release
cargo release major --execute

# Pre-release versions
cargo release --profile alpha --execute
cargo release --profile beta --execute
cargo release --profile rc --execute
```

### Method 3: GitHub Actions (Automated)

Releases are automatically triggered when:

1. **Tag-based release**: Push a version tag (e.g., `v0.1.0`)
2. **Manual release**: Use GitHub Actions workflow dispatch

#### Manual GitHub Release

1. Go to the GitHub repository
2. Click "Actions" tab
3. Select "Release" workflow
4. Click "Run workflow"
5. Choose the release type and version
6. Click "Run workflow"

## Release Process Steps

When you initiate a release, the following steps are automatically executed:

### Pre-release Checks

1. **Code quality checks**:
   - Run all tests (`cargo test --all-features`)
   - Run Clippy lints (`cargo clippy --all-targets --all-features -- -D warnings`)
   - Check code formatting (`cargo fmt --all -- --check`)
   - Build documentation (`cargo doc --no-deps --all-features`)
   - Security audit (`cargo audit`)

2. **Version validation**:
   - Ensure working directory is clean
   - Verify current branch is main
   - Check that version follows semantic versioning

### Release Execution

1. **Version bump**: Update version in `Cargo.toml`
2. **Changelog update**: Add new version entry to `CHANGELOG.md`
3. **Git operations**:
   - Create release commit
   - Create and sign git tag
   - Push changes and tags to remote

4. **Publishing**:
   - Publish to crates.io (for stable releases)
   - Create GitHub release with binaries
   - Build cross-platform binaries

### Post-release

1. **Binary builds**: Create optimized binaries for multiple platforms:
   - Linux (x86_64, musl)
   - macOS (x86_64, aarch64)
   - Windows (x86_64)

2. **GitHub release**: Automatic creation with:
   - Release notes from changelog
   - Attached binary artifacts
   - Installation instructions

## Configuration

### release.toml

The release process is configured in `release.toml`:

- **Pre-release hooks**: Quality checks before release
- **Post-release hooks**: Actions after successful release
- **Publishing settings**: Crates.io configuration
- **Git settings**: Tag and commit configuration
- **Release profiles**: Different configurations for different release types

### GitHub Actions

Release workflows are defined in `.github/workflows/release.yml`:

- **Automated releases**: Triggered by version tags
- **Manual releases**: Workflow dispatch for controlled releases
- **Cross-platform builds**: Binary generation for multiple platforms
- **Security**: Uses GitHub secrets for tokens

## Secrets Configuration

Set up the following secrets in your GitHub repository:

1. **CARGO_REGISTRY_TOKEN**: Token for publishing to crates.io
   - Get from: https://crates.io/me
   - Scope: Publish packages

2. **RELEASE_TOKEN** (optional): GitHub token with enhanced permissions
   - Default: Uses `GITHUB_TOKEN` (automatic)
   - Enhanced: Personal access token for additional permissions

## Troubleshooting

### Common Issues

1. **"Working directory is dirty"**:
   - Commit or stash all changes before releasing

2. **"Not on main branch"**:
   - Switch to main branch: `git checkout main`
   - Or use `--allow-branch` flag for other branches

3. **"Version already exists"**:
   - Check if tag already exists: `git tag -l`
   - Use different version or delete existing tag

4. **"Cargo publish failed"**:
   - Verify crates.io token is valid
   - Check if package name is available
   - Ensure all dependencies are published

5. **"Pre-release hook failed"**:
   - Fix the failing check (tests, clippy, formatting, etc.)
   - Re-run the release process

### Dry Run

Always test your release configuration with a dry run:

```bash
# Test patch release
cargo release patch --dry-run

# Test with specific profile
cargo release --profile alpha --dry-run
```

### Manual Recovery

If a release fails partway through:

1. **Check git status**: `git status`
2. **Check tags**: `git tag -l`
3. **Reset if needed**: `git reset --hard HEAD~1`
4. **Delete tag if created**: `git tag -d v0.1.0`
5. **Re-run release**: Start the process again

## Best Practices

1. **Always test before releasing**:
   - Run full test suite
   - Test on multiple platforms
   - Verify documentation builds

2. **Keep changelog updated**:
   - Document all changes
   - Follow conventional commit format
   - Include breaking changes clearly

3. **Use semantic versioning**:
   - Patch: Bug fixes only
   - Minor: New features, backward compatible
   - Major: Breaking changes

4. **Test pre-releases**:
   - Use alpha/beta for testing
   - Get feedback before stable release
   - Document known issues

5. **Monitor releases**:
   - Check GitHub release page
   - Verify crates.io publication
   - Monitor download statistics

## Support

For issues with the release process:

1. Check this documentation
2. Review `release.toml` configuration
3. Check GitHub Actions logs
4. Create an issue in the repository