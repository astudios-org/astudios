use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "as-man",
    version = "0.1.0",
    about = "Android Studio Manager - A CLI tool for managing Android Studio versions",
    long_about = "as-man is a command-line tool inspired by xcodes, built specifically for managing Android Studio installations on your local machine."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List available Android Studio versions
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
    },

    /// Install a specific Android Studio version to ~/.as-man/versions and /Applications
    Install {
        /// Version to install (e.g., "2024.3.2.14", "Android Studio Meerkat Feature Drop", "2023.3.1 Canary 8")
        version: Option<String>,

        /// Install the latest available version
        #[arg(long)]
        latest: bool,

        /// Custom installation directory (default: /Applications)
        #[arg(long, short)]
        directory: Option<String>,

        /// Force use of specific downloader [possible values: reqwest, aria2]
        #[arg(long, value_enum)]
        downloader: Option<DownloaderChoice>,
    },

    /// Uninstall a specific Android Studio version
    Uninstall {
        /// Version to uninstall
        version: String,
    },

    /// Switch to a different Android Studio version
    Use {
        /// Version to switch to
        version: String,
    },

    /// Show currently installed versions
    Installed,

    /// Show which version is currently active
    Which,

    /// Update the local releases cache
    Update,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum DownloaderChoice {
    Reqwest,
    Aria2,
}
