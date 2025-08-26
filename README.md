# as-man - Android Studio Manager

A CLI tool inspired by [xcodes](https://github.com/XcodesOrg/xcodes), built specifically for managing Android Studio installations on your local machine.

## Features

- **List available versions**: View all available Android Studio versions from JetBrains
- **Install specific versions**: Download and install any Android Studio version
- **Switch between versions**: Easily switch between installed versions
- **Multiple download methods**: Support for both built-in HTTP client and aria2 for faster downloads

## Installation

### Build from source
```bash
git clone https://github.com/Binlogo/as-man.git
cd as-man
cargo build --release
```

## Usage

### List available versions
```bash
as-man list
```

### Install a specific version
```bash
# Install the latest version
as-man install --latest

# Install a specific version
as-man install 2024.3.2.14

# Install with custom directory
as-man install 2024.3.2.14 --directory ~/Applications/Custom

# Force use of specific downloader
as-man install 2024.3.2.14 --downloader aria2
```

If you have aria2 installed (it's available in Homebrew, apt, yum, etc.), then as-man will default to use it for downloads. This significantly speeds up the download process.

### Switch between versions
```bash
as-man use 2024.3.2.14
```

### Show installed versions
```bash
as-man installed
```

### Show current active version
```bash
as-man which
```

### Uninstall a version
```bash
as-man uninstall 2024.3.2.14
```

## Examples

```bash
# List available versions
as-man list --limit 10

# Install the latest release version
as-man install --latest

# Install a specific beta version
as-man install "2024.3.1 Beta 2"

# Install with aria2 for faster download
as-man install 2024.3.2.14 --downloader aria2

# Install to custom directory
as-man install 2024.3.2.14 --directory ~/Applications
```

## License

MIT License - see LICENSE file for details.