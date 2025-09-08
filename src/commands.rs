use crate::cli::{Cli, Commands};
use astudios::{
    config::Config,
    downloader::Downloader,
    error::AstudiosError,
    installer::Installer,
    list::AndroidStudioLister,
    model::{AndroidStudio, InstalledAndroidStudio, ReleaseChannel},
    progress::ProgressReporter,
};
use colored::Colorize;
use std::{fs, path::Path, path::PathBuf};

/// Handles all CLI commands with proper error handling and user feedback
pub struct CommandHandler;

impl CommandHandler {
    /// Main entry point for handling CLI commands
    pub fn handle(cli: Cli) -> Result<(), AstudiosError> {
        match cli.command {
            Commands::List {
                release,
                beta,
                canary,
                limit,
                all_platforms,
            } => Self::handle_list(release, beta, canary, limit, all_platforms),
            Commands::Download {
                version,
                latest,
                latest_prerelease,
                directory,
            } => Self::handle_download(
                version.as_deref(),
                latest,
                latest_prerelease,
                directory.as_deref(),
            ),
            Commands::Install {
                version,
                latest,
                directory,
                skip_checks,
            } => Self::handle_install(
                version.as_deref(),
                latest,
                directory.as_deref(),
                skip_checks,
            ),
            Commands::Uninstall { version } => Self::handle_uninstall(&version),
            Commands::Use { version } => Self::handle_use(&version),
            Commands::Installed => Self::handle_installed(),
            Commands::Which => Self::handle_which(),
            Commands::Update => Self::handle_update(),
        }
    }

    /// Handle the list command to display available Android Studio versions
    fn handle_list(
        release: bool,
        beta: bool,
        canary: bool,
        limit: Option<usize>,
        all_platforms: bool,
    ) -> Result<(), AstudiosError> {
        let reporter = ProgressReporter::new(true);

        let lister = AndroidStudioLister::new()?;
        let releases = lister.get_releases()?;

        reporter.clear();

        let mut items = lister.filter_by_channel(releases, release, beta, canary);

        // Filter by current platform unless all_platforms flag is set
        if !all_platforms {
            items = lister.filter_by_current_platform(items);
        }

        let display_items: Vec<_> = if let Some(limit) = limit {
            items.into_iter().take(limit).collect()
        } else {
            items
        };

        // Get installed versions and active version for status display
        let installer = Installer::new()?;
        let installed_studios = installer.list_installed_studios().unwrap_or_default();
        let active_studio = installer.get_active_studio().unwrap_or_default();

        // Display header with platform information
        if all_platforms {
            println!(
                "{}",
                "Available Android Studio versions (all platforms):"
                    .green()
                    .bold()
            );
        } else {
            println!(
                "{} {}:",
                "Available Android Studio versions for".green().bold(),
                AndroidStudioLister::get_current_platform_name()
                    .green()
                    .bold()
            );
        }
        println!();

        if display_items.is_empty() {
            if all_platforms {
                println!(
                    "{} No versions found matching the specified criteria",
                    "⚠️".yellow()
                );
            } else {
                println!(
                    "{} No versions available for {} matching the specified criteria",
                    "⚠️".yellow(),
                    AndroidStudioLister::get_current_platform_name()
                );
                println!();
                println!("Use --all-platforms to see versions for all platforms");
            }
            return Ok(());
        }

        for item in display_items.iter().rev() {
            Self::print_version_info(item, &installed_studios, &active_studio);
        }

        Ok(())
    }

    fn print_version_info(
        item: &AndroidStudio,
        installed_studios: &[InstalledAndroidStudio],
        active_studio: &Option<InstalledAndroidStudio>,
    ) {
        let channel_color = match item.channel_type() {
            ReleaseChannel::Release => "Release".green(),
            ReleaseChannel::Beta => "Beta".yellow(),
            ReleaseChannel::Canary => "Canary".red(),
            ReleaseChannel::ReleaseCandidate => "RC".blue(),
            ReleaseChannel::Patch => "Patch".cyan(),
        };

        // Check if this version is installed
        let is_installed = installed_studios.iter().any(|installed| {
            // Match by version or build number
            installed.version.short_version == item.version
                || installed.version.build_version == item.build
                || installed.version.build_number == item.build
        });

        // Check if this version is currently selected/active
        let is_selected = if let Some(active) = active_studio {
            active.version.short_version == item.version
                || active.version.build_version == item.build
                || active.version.build_number == item.build
        } else {
            false
        };

        // Build status string like xcodes
        let status = match (is_installed, is_selected) {
            (true, true) => " [Installed, Selected]".green(),
            (true, false) => " [Installed]".green(),
            (false, true) => " [Selected]".green(), // This shouldn't happen in practice
            (false, false) => "".normal(),
        };

        println!(
            "{} {} ({}){}",
            ">".dimmed(),
            item.version.bold(),
            channel_color,
            status
        );
        println!("  {} {}", "Name:".dimmed(), item.name);
        println!("  {} {}", "Build:".dimmed(), item.build);
        println!("  {} {}", "Date:".dimmed(), item.date);

        // Show download information for macOS
        if let Some(download) = item.get_platform_download() {
            println!(
                "  {} {} ({})",
                "macOS:".dimmed(),
                "Available".green(),
                download.size
            );
        } else {
            println!("  {} {}", "macOS:".dimmed(), "Not Available".red());
        }

        println!();
    }

    /// Handle the install command to install Android Studio versions
    fn handle_install(
        version: Option<&str>,
        latest: bool,
        directory: Option<&str>,
        skip_checks: bool,
    ) -> Result<(), AstudiosError> {
        let lister = AndroidStudioLister::new()?;

        // Find the target version
        let target_item = if latest {
            lister.get_latest_release()?
        } else if let Some(version_query) = version {
            lister.find_version_by_query(version_query)?
        } else {
            return Err(AstudiosError::General(
                "Please specify a version or use --latest".to_string(),
            ));
        };

        let version_str = &target_item.version;
        let full_name = &target_item.name;
        let install_dir = directory.unwrap_or("/Applications");

        // Display installation header with clear formatting
        println!();
        println!("{}", "━".repeat(80).dimmed());
        println!(
            "{} {} {}",
            "🚀".blue(),
            "Installing Android Studio".bold(),
            version_str.cyan()
        );
        println!("   {}", full_name.green());
        println!(
            "   {} {}",
            "Target directory:".dimmed(),
            install_dir.yellow()
        );
        println!("{}", "━".repeat(80).dimmed());
        println!();

        let installer = Installer::new()?;
        installer.install_version_with_checks(version_str, full_name, directory, !skip_checks)?;

        // Display success summary
        println!();
        println!("{}", "━".repeat(80).dimmed());
        println!(
            "{} {} {}",
            "✅".green(),
            "Installation Complete".bold().green(),
            "🎉".green()
        );
        println!();
        println!("   {} {}", "Version:".dimmed(), version_str.cyan().bold());
        println!("   {} {}", "Location:".dimmed(), install_dir.yellow());

        // Show different information based on installation directory
        if directory.is_none() || directory == Some("/Applications") {
            println!(
                "   {} {}",
                "Symlink:".dimmed(),
                "/Applications/Android Studio.app".blue()
            );
            println!();
            println!(
                "   {} Launch Android Studio from Applications or run:",
                "💡".blue()
            );
            println!("   {}", "open \"/Applications/Android Studio.app\"".cyan());
        } else {
            println!(
                "   {} {}",
                "App Bundle:".dimmed(),
                format!("{}/Android Studio {}.app", install_dir, version_str).blue()
            );
            println!();
            println!("   {} Launch Android Studio by running:", "💡".blue());
            println!(
                "   {}",
                format!(
                    "open \"{}/Android Studio {}.app\"",
                    install_dir, version_str
                )
                .cyan()
            );
        }

        println!("{}", "━".repeat(80).dimmed());

        Ok(())
    }

    /// Handle the download command to download Android Studio versions
    fn handle_download(
        version: Option<&str>,
        latest: bool,
        latest_prerelease: bool,
        directory: Option<&str>,
    ) -> Result<(), AstudiosError> {
        let lister = AndroidStudioLister::new()?;

        // Find the target version
        let target_item = if latest {
            lister.get_latest_release()?
        } else if latest_prerelease {
            lister.get_latest_prerelease()?
        } else if let Some(version_query) = version {
            lister.find_version_by_query(version_query)?
        } else {
            return Err(AstudiosError::General(
                "Please specify a version or use --latest or --latest-prerelease".to_string(),
            ));
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
            // Create version-specific subdirectory in versions directory
            Config::default_download_dir().join(version_str)
        };

        // Ensure download directory exists
        fs::create_dir_all(&download_dir)?;

        // Get appropriate download for current platform
        let download = target_item
            .get_platform_download()
            .ok_or(AstudiosError::Download(
                "No download available for current platform".to_string(),
            ))?;

        // Create filename from URL
        let default_filename = format!("android-studio-{version_str}.dmg");
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
        println!("  Location: {}", download_path.display());

        Ok(())
    }

    /// Handle the uninstall command
    fn handle_uninstall(version: &str) -> Result<(), AstudiosError> {
        let installer = Installer::new()?;

        println!();
        println!("{} Uninstalling Android Studio {}...", "🗑️".red(), version);
        println!();

        installer.uninstall_version(version)?;

        println!();
        println!(
            "{} Successfully uninstalled Android Studio {}",
            "✅".green(),
            version
        );

        Ok(())
    }

    /// Handle the use command to switch versions
    fn handle_use(version: &str) -> Result<(), AstudiosError> {
        let installer = Installer::new()?;
        installer.switch_to_version(version)?;
        println!("{} Now using Android Studio {}", "✅".green(), version);
        Ok(())
    }

    /// Handle the installed command to show installed versions
    fn handle_installed() -> Result<(), AstudiosError> {
        let installer = Installer::new()?;
        let installations = installer.list_installed_studios()?;

        if installations.is_empty() {
            println!("{} No Android Studio versions installed", "⚠️".yellow());
            println!();
            println!("Use 'astudios install <version>' to install a version");
        } else {
            println!("{} Installed Android Studio versions:", "📋".green().bold());
            println!();

            let active = installer.get_active_studio()?;
            let active_id = active.as_ref().map(|a| a.identifier());

            for installation in installations {
                let is_active = active_id.as_ref() == Some(&installation.identifier());

                // Enhanced display name with better formatting
                let enhanced_name = installation.enhanced_display_name();
                let detailed_version = installation.extract_detailed_version();

                // Status indicator with better visual distinction
                let status_indicator = if is_active {
                    "✅ [Selected]".green().bold()
                } else {
                    "".normal()
                };

                // Main version line with improved formatting and proper spacing
                if is_active {
                    println!("   {} {}", enhanced_name.cyan().bold(), status_indicator);
                } else {
                    println!("   {}", enhanced_name.cyan().bold());
                }

                // Version and build info with better alignment and spacing
                println!(
                    "        Version: {} | Build: {}",
                    detailed_version.green(),
                    installation.identifier().blue()
                );

                // Path with proper formatting
                let path_str = installation.path.display().to_string();
                println!("        Path: {}", path_str.dimmed());

                println!();
            }
        }

        Ok(())
    }

    /// Handle the which command to show current version
    fn handle_which() -> Result<(), AstudiosError> {
        let installer = Installer::new()?;
        let active = installer.get_active_studio()?;

        match active {
            Some(installation) => {
                println!(
                    "{} Currently using {} ({})",
                    "✅".green(),
                    installation.display_name().green(),
                    installation.identifier().blue()
                );
                println!(
                    "   Path: {}",
                    installation.path.display().to_string().dimmed()
                );
            }
            None => {
                println!(
                    "{} Android Studio is not installed or symlink is missing",
                    "⚠️".yellow()
                );
                println!();
                println!("Use 'astudios install <version>' to install a version");
            }
        }

        Ok(())
    }

    /// Handle the update command to refresh version cache
    fn handle_update() -> Result<(), AstudiosError> {
        let reporter = ProgressReporter::new(true);

        // Force refresh by clearing cache
        let lister = AndroidStudioLister::new()?;
        let releases = lister.get_releases()?;

        reporter.finish_with_success("Version list updated");

        println!(
            "{} Found {} available versions",
            "✅".green(),
            releases.items.len()
        );

        // Show the latest few versions
        let latest_versions: Vec<_> = releases.items.into_iter().take(5).collect();

        if !latest_versions.is_empty() {
            println!();
            println!("{} Latest versions:", "📋".bold());
            for item in latest_versions {
                let channel = match item.channel_type() {
                    ReleaseChannel::Release => "Release".normal(),
                    ReleaseChannel::Beta => "Beta".yellow(),
                    ReleaseChannel::Canary => "Canary".red(),
                    ReleaseChannel::ReleaseCandidate => "RC".blue(),
                    ReleaseChannel::Patch => "Patch".cyan(),
                };
                println!("  {} - {} ({}", item.version.green(), item.build, channel);
            }
        }

        Ok(())
    }
}
