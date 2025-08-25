use clap::{Parser, Subcommand};

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

    /// Install a specific Android Studio version
    Install {
        /// Version to install (e.g., "2023.1.1")
        version: String,
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
