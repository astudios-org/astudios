# astudios - Android Studio Manager

A CLI tool inspired by [xcodes](https://github.com/XcodesOrg/xcodes), built specifically for managing Android Studio installations on macOS.

## Platform Support

**Currently supports macOS only.** This tool is designed specifically for macOS and uses macOS-specific features like DMG mounting and app bundle management.

We welcome community contributions to add support for other operating systems in the future. If you're interested in contributing Windows or Linux support, please feel free to open an issue or submit a pull request.

## Features

- **List available versions**: View all available Android Studio versions from JetBrains
- **Install specific versions**: Download and install any Android Studio version
- **Switch between versions**: Easily switch between installed versions
- **Multiple download methods**: Support for both built-in HTTP client and aria2 for faster downloads

## Installation

### System Requirements

- **macOS 10.14 or later**
- **Architecture**: Intel (x86_64) or Apple Silicon (aarch64)
- **Required tools**: `hdiutil`, `codesign`, `cp`, `rm` (usually pre-installed on macOS)
- **Optional**: `aria2` for faster downloads (install via Homebrew: `brew install aria2`)

### Build from source
```bash
git clone https://github.com/astudios-org/astudios.git
cd astudios
cargo install --path .
```

## Usage

### List available versions
```bash
astudios list
```

### Install a specific version
```bash
# Install the latest version
astudios install --latest

# Install a specific version
astudios install 2025.1.3.7

# Install with custom directory
astudios install 2025.1.3.7 --directory ~/Applications/Custom
```

If you have aria2 installed (available via Homebrew: `brew install aria2`), astudios will automatically use it for downloads, which significantly speeds up the download process.

### Switch between versions
```bash
astudios use 2025.1.3.7
```

### Show installed versions
```bash
astudios installed
```

### Show current active version
```bash
astudios which
```

### Uninstall a version
```bash
astudios uninstall 2025.1.3.7
```

## Examples

```bash
# List available versions
astudios list --limit 10

# Install the latest release version
astudios install --latest

# Install a specific beta version
astudios install "2025.1.4 Canary 4"

# Install to custom directory
astudios install 2025.1.3.7 --directory ~/Applications
```

## License

MIT License - see LICENSE file for details.