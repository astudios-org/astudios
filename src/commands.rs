use crate::api::ApiClient;

use crate::cli::{Cli, Commands};
use crate::installer::Installer;
use crate::model::{AndroidStudio, Content};
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
            Commands::Install {
                version,
                latest,
                directory,
            } => Self::handle_install(version.as_deref(), latest, directory.as_deref()),
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
        let _client = ApiClient::new()?;

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

    fn print_version_info(item: &AndroidStudio) {
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

    fn handle_install(
        version: Option<&str>,
        latest: bool,
        directory: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let _client = ApiClient::new()?;
        let content = Self::get_cached_releases()?;

        // Find the target version
        let target_item = if latest {
            content.items.first().ok_or("No versions available")?
        } else if let Some(version_query) = version {
            Self::find_version_by_query(&content.items, version_query)?
        } else {
            return Err("Please specify a version or use --latest".into());
        };

        let version_str = &target_item.version;
        let full_name = &target_item.name;

        println!();
        println!(
            "{} Installing {} ({})...",
            "🚀".blue(),
            full_name.green().bold(),
            version_str
        );
        println!();

        // Step 1: Download
        println!("(1/6) Downloading {full_name}: 0%");
        let installer = Installer::new()?;
        installer.install_version_with_progress(version_str, full_name)?;

        // Step 2: Unarchive
        println!("(2/6) Unarchiving Android Studio (This can take a while)");
        // Already handled in install_version_with_progress

        // Step 3: Move to Applications
        println!(
            "(3/6) Moving Android Studio to {}...",
            directory.unwrap_or("/Applications")
        );
        if let Some(custom_dir) = directory {
            installer.install_macos_to_directory(version_str, full_name, custom_dir)?;
        } else {
            installer.install_macos(version_str, full_name)?;
        }

        // Step 4: Cleanup archive
        println!("(4/6) Moving Android Studio archive to the Trash");
        installer.cleanup_archive(version_str)?;

        // Step 5: Security assessment
        println!("(5/6) Checking security assessment and code signing");
        installer.verify_installation(version_str, full_name, directory)?;

        // Step 6: Finish
        println!("(6/6) Finishing installation");

        println!();
        println!(
            "{} has been installed to {}/{}.app",
            full_name.green().bold(),
            directory.unwrap_or("/Applications"),
            full_name
        );

        Ok(())
    }

    fn handle_uninstall(version: &str) -> Result<(), Box<dyn Error>> {
        let installer = Installer::new()?;
        installer.uninstall_version(version)?;
        Ok(())
    }

    fn handle_use(version: &str) -> Result<(), Box<dyn Error>> {
        let installer = Installer::new()?;
        installer.switch_to_version(version)?;
        Ok(())
    }

    fn handle_installed() -> Result<(), Box<dyn Error>> {
        let installer = Installer::new()?;
        installer.list_installed_versions()?;
        Ok(())
    }

    fn handle_which() -> Result<(), Box<dyn Error>> {
        let installer = Installer::new()?;
        installer.show_current_version()?;
        Ok(())
    }

    fn find_version_by_query<'a>(
        items: &'a [AndroidStudio],
        query: &str,
    ) -> Result<&'a AndroidStudio, Box<dyn Error>> {
        // First try exact version match
        if let Some(item) = items.iter().find(|item| item.version == query) {
            return Ok(item);
        }

        // Try partial version match
        if let Some(item) = items.iter().find(|item| item.version.contains(query)) {
            return Ok(item);
        }

        // Try name match
        if let Some(item) = items
            .iter()
            .find(|item| item.name.to_lowercase().contains(&query.to_lowercase()))
        {
            return Ok(item);
        }

        // Try channel-based search (e.g., "2023.3.1 Canary 8")
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.len() >= 3 {
            let version_part = parts[0];
            let channel_part = parts[1].to_lowercase();
            let build_part = parts[2];

            if let Some(item) = items.iter().find(|item| {
                item.version.contains(version_part)
                    && item.channel.to_lowercase() == channel_part
                    && item.build.contains(build_part)
            }) {
                return Ok(item);
            }
        }

        // Try simpler channel search
        for item in items {
            let search_string = format!("{} {} {}", item.version, item.channel, item.build);
            if search_string.to_lowercase().contains(&query.to_lowercase()) {
                return Ok(item);
            }
        }

        Err(format!(
            "Version '{query}' not found. Use 'as-man list' to see available versions."
        )
        .into())
    }

    fn handle_update() -> Result<(), Box<dyn Error>> {
        println!("{} Updating Android Studio version list...", "🔄".blue());

        // Fetch fresh releases from JetBrains
        let content = crate::api::ApiClient::new()?.fetch_releases()?;

        println!(
            "{} Found {} available versions",
            "✅".green(),
            content.items.len()
        );

        // Show the latest few versions
        let latest_versions: Vec<_> = content.items.iter().take(5).collect();

        if !latest_versions.is_empty() {
            println!();
            println!("{} Latest versions:", "📋".bold());
            for item in latest_versions {
                println!(
                    "  {} - {} ({})",
                    item.version.green(),
                    item.build,
                    if item.is_beta() {
                        "Beta".yellow()
                    } else if item.is_canary() {
                        "Canary".red()
                    } else {
                        "Release".normal()
                    }
                );
            }
        }

        Ok(())
    }
}
