# as-man manual

## Synopsis

```
OVERVIEW: Manage Android Studio installations on your machine

USAGE: as-man <subcommand>

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
  version                 Print the version number of as-man itself

  See 'as-man help <subcommand>' for detailed help.
```

## Subcommands

### as-man download

```
OVERVIEW: Download a specific version of Android Studio

`as-man` will download the specified version from the official JetBrains data feed.
By default, it uses a high-performance parallel downloader to accelerate the process.

The version can be specified by its codename or full version number.

EXAMPLES:
  as-man download Hedgehog
  as-man download "Giraffe Patch 2"
  as-man download 2023.1.1
  as-man download --latest
  as-man download Iguana --directory ~/AS_Archives/

USAGE: as-man download [<version> ...] [--latest] [--latest-prerelease] [--directory <directory>]

ARGUMENTS:
  <version>               The version to download (e.g., "Hedgehog", "2022.3.1").
                          If omitted, you will be prompted to choose from a list.

OPTIONS:
  --latest                Download the latest stable release version available.
  --latest-prerelease     Download the latest pre-release version available (Canary or Beta).
  --directory <directory> The directory to download the archive to. Defaults to ~/Downloads.
  -h, --help              Show help information.

```

### as-man install

```
OVERVIEW: Download and install a specific version of Android Studio

This command first downloads the specified version archive and then unpacks and
installs it to the appropriate applications directory.

EXAMPLES:
  as-man install Hedgehog
  as-man install "Iguana Canary 15"
  as-man install --latest --select
  as-man install 2023.1.1 --path ~/Downloads/android-studio-2023.1.1-mac_arm.dmg
  as-man install --latest --directory "/Custom/Android/Studios"

USAGE: as-man install [<options>] [<version> ...]

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

### as-man installed

```
OVERVIEW: List the versions of Android Studio that are installed locally

USAGE: as-man installed [--directory <directory>]

OPTIONS:
  --directory <directory> The base directory where your Android Studio versions are installed.
  -h, --help              Show help information.

```

### as-man list

```
OVERVIEW: List all versions of Android Studio that are available to install

This command fetches data from the official JetBrains XML feed. The data is
cached locally for a short period to improve performance.

USAGE: as-man list [--force]

OPTIONS:
  --force                 Force an update of the available version list, ignoring the cache.
  -h, --help              Show help information.

```

### as-man use

```
OVERVIEW: Change the active Android Studio version

This command creates a symbolic link named `Android Studio` pointing to the
specified installed version. This allows for consistent launch behavior from the
command line or UI.

Run without arguments to interactively select from a list of installed versions.

EXAMPLES:
  as-man use Hedgehog
  as-man use 2022.3.1
  as-man use /Applications/Android\ Studio\ Iguana.app

USAGE: as-man use [<version-or-path>] [--directory <directory>]

ARGUMENTS:
  <version-or-path>       The codename, version number, or direct path of the
                          installed version to activate.

OPTIONS:
  --directory <directory> The directory where your Android Studio versions are installed.
  -h, --help              Show help information.

```

### as-man uninstall

```
OVERVIEW: Uninstall a version of Android Studio

Run without any arguments to interactively select a version to uninstall from a list.

EXAMPLES:
  as-man uninstall Giraffe
  as-man uninstall 2022.3.1

USAGE: as-man uninstall [<version> ...] [--directory <directory>]

ARGUMENTS:
  <version>               The version to uninstall.

OPTIONS:
  --directory <directory> The directory where your Android Studio versions are installed.
  -h, --help              Show help information.

```

### as-man clean

```
OVERVIEW: Remove cache and log files from old Android Studio installations

This utility helps free up disk space by deleting logs, caches, and other temporary
files associated with uninstalled or outdated versions of Android Studio.

USAGE: as-man clean [--dry-run]

OPTIONS:
  --dry-run               List the files and directories that would be deleted,
                          without actually deleting them.
  -h, --help              Show help information.

```

### as-man update

```
OVERVIEW: Update the list of available versions of Android Studio

This is an alias for `as-man list --force`. It forces a refresh of the local
cache of available Android Studio versions from the JetBrains server.

USAGE: as-man update

```

### as-man version

```
OVERVIEW: Print the version number of as-man itself

USAGE: as-man version

```

