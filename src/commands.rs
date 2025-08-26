use crate::api::ApiClient;
use crate::app_installer::AppInstaller;
use crate::cli::{Cli, Commands};
use crate::installer::Installer;
use crate::model::{Content, Item};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

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
        pb.set_message("Loading version list...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let content = Self::get_cached_releases()?;

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

    fn get_cached_releases() -> Result<Content, Box<dyn Error>> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("as-man");
        
        fs::create_dir_all(&cache_dir)?;
        
        let cache_path = cache_dir.join("releases.json");
        let cache_duration = Duration::from_secs(60 * 60); // 1 hour cache
        
        // Check if cache exists and is valid
        if cache_path.exists() {
            let metadata = fs::metadata(&cache_path)?;
            let modified = metadata.modified()?;
            let age = SystemTime::now().duration_since(modified)?;
            
            if age < cache_duration {
                // Use cached data
                println!("{} Using cached version list", "💾".blue());
                let data = fs::read_to_string(&cache_path)?;
                let content: Content = serde_json::from_str(&data)?;
                return Ok(content);
            }
        }
        
        // Fetch fresh data
        println!("{} Fetching latest releases from JetBrains...", "🔄".blue());
        let client = ApiClient::new()?;
        let content = client.fetch_releases()?;
        
        // Cache the data
        let data = serde_json::to_string_pretty(&content)?;
        fs::write(&cache_path, data)?;
        
        Ok(content)
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

    fn handle_uninstall(version: &str) -> Result<(), Box<dyn Error>> {
        let app_installer = AppInstaller::new()?;
        app_installer.uninstall_version(version)?;
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
        println!("{} Updating Android Studio version list...", "🔄".blue());
        
        // Fetch fresh releases from JetBrains
        let content = crate::api::ApiClient::new()?.fetch_releases()?;
        
        println!("{} Found {} available versions", "✅".green(), content.items.len());
        
        // Show the latest few versions
        let latest_versions: Vec<_> = content.items.iter().take(5).collect();
        
        if !latest_versions.is_empty() {
            println!();
            println!("{} Latest versions:", "📋".bold());
            for item in latest_versions {
                println!("  {} - {} ({})", 
                    item.version.green(), 
                    item.build, 
                    if item.is_beta() { "Beta".yellow() } else if item.is_canary() { "Canary".red() } else { "Release".normal() }
                );
            }
        }
        
        Ok(())
    }
}
