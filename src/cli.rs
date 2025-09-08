use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "astudios",
    version = "0.1.0",
    about = "Manage the Android Studio installations",
    long_about = "Manage the Android Studio installations"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all versions of Android Studio that are available to install
    List {
        /// Show only release versions
        #[arg(long)]
        release: bool,

        /// Show only beta versions
        #[arg(long)]
        beta: bool,

        /// Show only canary versions
        #[arg(long)]
        canary: bool,

        /// Limit the number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Show download information for all platforms instead of just the current platform
        #[arg(long)]
        all_platforms: bool,
    },

    /// Download a specific version of Android Studio
    Download {
        /// Version to download (e.g., "Hedgehog", "2022.3.1")
        version: Option<String>,

        /// Download the latest stable release version available
        #[arg(long)]
        latest: bool,

        /// Download the latest pre-release version available (Canary or Beta)
        #[arg(long)]
        latest_prerelease: bool,

        /// The directory to download the archive to. Defaults to ~/.astudios/versions/{version}
        #[arg(long)]
        directory: Option<String>,
    },

    /// Download and install a specific version of Android Studio
    Install {
        /// Version to install (e.g., "2024.3.2.14", "Android Studio Meerkat Feature Drop", "2023.3.1 Canary 8")
        version: Option<String>,

        /// Install the latest available version
        #[arg(long)]
        latest: bool,

        /// Custom installation directory (default: /Applications)
        #[arg(long, short)]
        directory: Option<String>,

        /// Skip prerequisite checks (not recommended)
        #[arg(long)]
        skip_checks: bool,
    },

    /// Uninstall a version of Android Studio
    Uninstall {
        /// Version to uninstall
        version: String,
    },

    /// Change the selected Android Studio
    Use {
        /// Version to switch to
        version: String,
    },

    /// List the versions of Android Studio that are installed
    Installed,

    /// Show which version is currently selected
    Which,

    /// Update the list of available versions of Android Studio
    Update,
}
