use as_man::api::ApiClient;
use as_man::downloader::Downloader;
use as_man::installer::Installer;
use as_man::list::AndroidStudioLister;
use as_man::model::AndroidStudio;

use crate::cli::{Cli, Commands, DownloaderChoice};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use dirs;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

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
            Commands::Download {
                version,
                latest,
                latest_prerelease,
                directory,
            } => Self::handle_download(version.as_deref(), latest, latest_prerelease, directory.as_deref()),
            Commands::Install {
                version,
                latest,
                directory,
                downloader,
            } => Self::handle_install(version.as_deref(), latest, directory.as_deref(), downloader),
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

        let lister = AndroidStudioLister::new()?;
        let list = lister.get_releases()?;

        pb.finish_and_clear();

        let mut items = list.items;

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

        items.reverse();

        for item in items {
            Self::print_version_info(&item);
        }

        Ok(())
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
        downloader_choice: Option<crate::cli::DownloaderChoice>,
    ) -> Result<(), Box<dyn Error>> {
        let _client = ApiClient::new()?;
        let lister = AndroidStudioLister::new()?;

        // Find the target version
        let target_item = if latest {
            lister.get_latest_release()?
        } else if let Some(version_query) = version {
            lister.find_version_by_query(version_query)?
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

        // Create downloader based on choice
        let downloader = match downloader_choice {
            Some(DownloaderChoice::Reqwest) => Some(Downloader::Reqwest),
            Some(DownloaderChoice::Aria2) => Downloader::find_aria2().map(Downloader::Aria2).ok(),
            None => None, // Use auto-detection
        };

        let installer = Installer::new()?;

        // Step 1: Download (handled in install_version_with_progress)
        installer.install_version_with_progress(version_str, full_name, downloader)?;

        // Step 2: Unarchive (handled in extract_archive)
        // Step 3: Move to Applications
        let target_dir = directory.unwrap_or("/Applications");
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("(3/6) Moving Android Studio to {msg}...")
                .unwrap(),
        );
        pb.set_message(target_dir.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        if let Some(custom_dir) = directory {
            installer.install_macos_to_directory(version_str, full_name, custom_dir)?;
        } else {
            installer.install_macos_with_progress(version_str, full_name, &pb)?;
        }
        pb.finish_with_message("✅ Move complete");

        // Step 4: Cleanup archive
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("(4/6) Moving Android Studio archive to the Trash")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        installer.cleanup_archive(version_str)?;
        pb.finish_with_message("✅ Cleanup complete");

        // Step 5: Security assessment
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("(5/6) Checking security assessment and code signing")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        installer.verify_installation(version_str, full_name, directory)?;
        pb.finish_with_message("✅ Security check complete");

        // Step 6: Finish
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("(6/6) Finishing installation")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        std::thread::sleep(std::time::Duration::from_millis(500)); // Brief pause for user experience
        pb.finish_with_message("✅ Installation complete");

        println!();
        println!(
            "{} has been installed to {}",
            full_name.green().bold(),
            directory.unwrap_or("/Applications"),
        );

        Ok(())
    }

    fn handle_download(
        version: Option<&str>,
        latest: bool,
        latest_prerelease: bool,
        directory: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let _client = ApiClient::new()?;
        let lister = AndroidStudioLister::new()?;

        // Find the target version
        let target_item = if latest {
            lister.get_latest_release()?
        } else if latest_prerelease {
            lister.get_latest_prerelease()?
        } else if let Some(version_query) = version {
            lister.find_version_by_query(version_query)?
        } else {
            return Err("Please specify a version or use --latest or --latest-prerelease".into());
        };

        let version_str = &target_item.version;
        let full_name = &target_item.name;

        println!();
        println!(
            "{} Downloading {} ({})...",
            "🚀".blue(),
            full_name.green().bold(),
            version_str
        );
        println!();

        // Determine download directory
        let download_dir = if let Some(dir) = directory {
            PathBuf::from(dir)
        } else {
            let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
            home_dir.join("Downloads")
        };

        // Ensure download directory exists
        fs::create_dir_all(&download_dir)?;

        // Get appropriate download for current platform
        let download = target_item.get_macos_download()
            .ok_or("macOS download not available for this version")?;

        // Create filename from URL
        let default_filename = format!("android-studio-{}.dmg", version_str);
        let filename = Path::new(&download.link)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&default_filename);

        let download_path = download_dir.join(filename);

        // Skip if file already exists
        if download_path.exists() {
            let metadata = fs::metadata(&download_path)?;
            if metadata.len() > 0 {
                println!(
                    "{} File already exists: {}",
                    "ℹ️".yellow(),
                    download_path.display()
                );
                return Ok(());
            }
        }

        // Use best available downloader
        let downloader = Downloader::detect_best();
        println!(
            "{} Using downloader: {}",
            "📥".blue(),
            downloader.description()
        );

        // Download the file
        downloader.download(&download.link, &download_path, Some(full_name))?;

        println!();
        println!(
            "{} {} downloaded successfully!",
            "✅".green(),
            full_name.green().bold()
        );
        println!(
            "  Location: {}",
            download_path.display()
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

    fn handle_update() -> Result<(), Box<dyn Error>> {
        println!("{} Updating Android Studio version list...", "🔄".blue());

        // Fetch fresh releases from JetBrains
        let content = as_man::api::ApiClient::new()?.fetch_releases()?;

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
