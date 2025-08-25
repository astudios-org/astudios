use crate::api::ApiClient;
use crate::app_installer::AppInstaller;
use crate::cli::{Cli, Commands};
use crate::installer::Installer;
use crate::model::Item;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;

pub struct CommandHandler;

impl CommandHandler {
    pub fn handle(cli: Cli) -> Result<(), Box<dyn Error>> {
        match cli.command {
            Commands::List {
                release,
                beta,
                canary,
                limit,
            } => Self::handle_list(release, beta, canary, limit),
            Commands::Install { version } => Self::handle_install(&version),
            Commands::Uninstall { version } => Self::handle_uninstall(&version),
            Commands::Use { version } => Self::handle_use(&version),
            Commands::Installed => Self::handle_installed(),
            Commands::Which => Self::handle_which(),
            Commands::Update => Self::handle_update(),
        }
    }

    fn handle_list(
        release: bool,
        beta: bool,
        canary: bool,
        limit: Option<usize>,
    ) -> Result<(), Box<dyn Error>> {
        let client = ApiClient::new()?;

        println!("{} Android Studio releases...", "Fetching".blue().bold());
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .unwrap(),
        );
        pb.set_message("Connecting to JetBrains...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let content = client.fetch_releases()?;

        pb.finish_and_clear();

        let mut items = content.items;

        // Filter based on channel flags
        if release {
            items.retain(|item| item.is_release());
        }
        if beta {
            items.retain(|item| item.is_beta());
        }
        if canary {
            items.retain(|item| item.is_canary());
        }

        // Apply limit if specified
        if let Some(limit) = limit {
            items.truncate(limit);
        }

        println!("{}", "Available Android Studio versions:".green().bold());
        println!();

        for item in items {
            Self::print_version_info(&item);
        }

        Ok(())
    }

    fn print_version_info(item: &Item) {
        let channel_color = match item.channel.as_str() {
            "Release" => "Release".green(),
            "Beta" => "Beta".yellow(),
            "Canary" => "Canary".red(),
            "RC" => "RC".blue(),
            "Patch" => "Patch".cyan(),
            _ => item.channel.as_str().normal(),
        };

        println!(
            "{} {} ({})",
            ">".dimmed(),
            item.version.bold(),
            channel_color
        );
        println!("  {} {}", "Name:".dimmed(), item.name);
        println!("  {} {}", "Build:".dimmed(), item.build);
        println!("  {} {}", "Date:".dimmed(), item.date);

        if let Some(download) = item.get_macos_download() {
            println!(
                "  {} {} ({})",
                "macOS:".dimmed(),
                "Available".green(),
                download.size
            );
        }
        if let Some(download) = item.get_windows_download() {
            println!(
                "  {} {} ({})",
                "Windows:".dimmed(),
                "Available".green(),
                download.size
            );
        }
        if let Some(download) = item.get_linux_download() {
            println!(
                "  {} {} ({})",
                "Linux:".dimmed(),
                "Available".green(),
                download.size
            );
        }

        println!();
    }

    fn handle_install(version: &str) -> Result<(), Box<dyn Error>> {
        let installer = Installer::new()?;
        installer.install_version(version)?;
        
        let app_installer = AppInstaller::new()?;
        app_installer.install_application(version)?;
        Ok(())
    }

    fn handle_uninstall(_version: &str) -> Result<(), Box<dyn Error>> {
        println!("{} Uninstall command not yet implemented", "⚠".yellow());
        Ok(())
    }

    fn handle_use(version: &str) -> Result<(), Box<dyn Error>> {
        let app_installer = AppInstaller::new()?;
        app_installer.switch_to_version(version)?;
        Ok(())
    }

    fn handle_installed() -> Result<(), Box<dyn Error>> {
        let app_installer = AppInstaller::new()?;
        app_installer.list_installed_versions()?;
        Ok(())
    }

    fn handle_which() -> Result<(), Box<dyn Error>> {
        let app_installer = AppInstaller::new()?;
        app_installer.show_current_version()?;
        Ok(())
    }

    fn handle_update() -> Result<(), Box<dyn Error>> {
        println!("{} Update command not yet implemented", "⚠".yellow());
        Ok(())
    }
}
