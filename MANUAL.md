# astudios manual

## Synopsis

```
OVERVIEW: Manage Android Studio installations on your machine

USAGE: astudios <subcommand>

OPTIONS:
  -h, --help              Show help information.

SUBCOMMANDS:
  download                Download a specific version of Android Studio
  install                 Download and install a specific version of Android Studio
  installed               List locally installed versions of Android Studio
  list                    List all versions of Android Studio available to install
  use                     Change the active Android Studio version
  uninstall               Uninstall a version of Android Studio
  clean                   Remove cache and log files from old installations
  update                  Update the list of available versions of Android Studio
  version                 Print the version number of astudios itself

  See 'astudios help <subcommand>' for detailed help.
```

## Subcommands

### astudios download

```
OVERVIEW: Download a specific version of Android Studio

`astudios` will download the specified version from the official JetBrains data feed.
By default, it uses a high-performance parallel downloader to accelerate the process.

The version can be specified by its codename or full version number.

EXAMPLES:
  astudios download Hedgehog
  astudios download "Giraffe Patch 2"
  astudios download 2023.1.1
  astudios download --latest
  astudios download Iguana --directory ~/AS_Archives/

USAGE: astudios download [<version> ...] [--latest] [--latest-prerelease] [--directory <directory>]

ARGUMENTS:
  <version>               The version to download (e.g., "Hedgehog", "2022.3.1").
                          If omitted, you will be prompted to choose from a list.

OPTIONS:
  --latest                Download the latest stable release version available.
  --latest-prerelease     Download the latest pre-release version available (Canary or Beta).
  --directory <directory> The directory to download the archive to. Defaults to ~/Downloads.
  -h, --help              Show help information.

```

### astudios install

```
OVERVIEW: Download and install a specific version of Android Studio

This command first downloads the specified version archive and then unpacks and
installs it to the appropriate applications directory.

EXAMPLES:
  astudios install Hedgehog
  astudios install "Iguana Canary 15"
  astudios install --latest --select
  astudios install 2023.1.1 --path ~/Downloads/android-studio-2023.1.1-mac_arm.dmg
  astudios install --latest --directory "/Custom/Android/Studios"

USAGE: astudios install [<options>] [<version> ...]

ARGUMENTS:
  <version>               The version to install. If omitted, an interactive
                          prompt will be shown.

OPTIONS:
  --path <path>           Local path to an Android Studio .zip or .dmg file.
  --latest                Download and install the latest stable release available.
  --latest-prerelease     Download and install the latest pre-release version.
  --select                Set this version as the active one after installation.
  --directory <directory> The directory to install Android Studio into.
                          Defaults to /Applications (macOS) or ~/.local/share (Linux).
  --clean                 Completely delete the downloaded archive after a
                          successful installation.
  -h, --help              Show help information.

```

### astudios installed

```
OVERVIEW: List the versions of Android Studio that are installed locally

USAGE: astudios installed [--directory <directory>]

OPTIONS:
  --directory <directory> The base directory where your Android Studio versions are installed.
  -h, --help              Show help information.

```

### astudios list

```
OVERVIEW: List all versions of Android Studio that are available to install

This command fetches data from the official JetBrains XML feed. The data is
cached locally for a short period to improve performance.

USAGE: astudios list [--force]

OPTIONS:
  --force                 Force an update of the available version list, ignoring the cache.
  -h, --help              Show help information.

```

### astudios use

```
OVERVIEW: Change the active Android Studio version

This command creates a symbolic link named `Android Studio` pointing to the
specified installed version. This allows for consistent launch behavior from the
command line or UI.

Run without arguments to interactively select from a list of installed versions.

EXAMPLES:
  astudios use Hedgehog
  astudios use 2022.3.1
  astudios use /Applications/Android\ Studio\ Iguana.app

USAGE: astudios use [<version-or-path>] [--directory <directory>]

ARGUMENTS:
  <version-or-path>       The codename, version number, or direct path of the
                          installed version to activate.

OPTIONS:
  --directory <directory> The directory where your Android Studio versions are installed.
  -h, --help              Show help information.

```

### astudios uninstall

```
OVERVIEW: Uninstall a version of Android Studio

Run without any arguments to interactively select a version to uninstall from a list.

EXAMPLES:
  astudios uninstall Giraffe
  astudios uninstall 2022.3.1

USAGE: astudios uninstall [<version> ...] [--directory <directory>]

ARGUMENTS:
  <version>               The version to uninstall.

OPTIONS:
  --directory <directory> The directory where your Android Studio versions are installed.
  -h, --help              Show help information.

```

### astudios clean

```
OVERVIEW: Remove cache and log files from old Android Studio installations

This utility helps free up disk space by deleting logs, caches, and other temporary
files associated with uninstalled or outdated versions of Android Studio.

USAGE: astudios clean [--dry-run]

OPTIONS:
  --dry-run               List the files and directories that would be deleted,
                          without actually deleting them.
  -h, --help              Show help information.

```

### astudios update

```
OVERVIEW: Update the list of available versions of Android Studio

This is an alias for `astudios list --force`. It forces a refresh of the local
cache of available Android Studio versions from the JetBrains server.

USAGE: astudios update

```

### astudios version

```
OVERVIEW: Print the version number of astudios itself

USAGE: astudios version

```

