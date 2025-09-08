use crate::{
    config::Config, detector::SystemDetector, downloader::Downloader, error::AstudiosError,
    model::InstalledAndroidStudio,
};
use colored::Colorize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Archive extraction support
#[derive(Debug, Clone, Copy)]
pub enum ArchiveType {
    Dmg,
    Unsupported,
}

/// Installation manager for Android Studio
pub struct Installer {
    install_dir: PathBuf,
    applications_dir: PathBuf,
}

impl Installer {
    /// Create a new installer with default directories
    pub fn new() -> Result<Self, AstudiosError> {
        let install_dir = Config::versions_dir();
        let applications_dir = Config::default_applications_dir();

        fs::create_dir_all(&install_dir)?;

        Ok(Self {
            install_dir,
            applications_dir,
        })
    }

    /// Create a new installer with custom directories
    pub fn with_directories(
        install_dir: PathBuf,
        applications_dir: PathBuf,
    ) -> Result<Self, AstudiosError> {
        fs::create_dir_all(&install_dir)?;
        Ok(Self {
            install_dir,
            applications_dir,
        })
    }

    /// Install Android Studio version
    pub fn install_version(
        &self,
        version: &str,
        full_name: &str,
        custom_dir: Option<&str>,
    ) -> Result<(), AstudiosError> {
        self.install_version_with_checks(version, full_name, custom_dir, true)
    }

    /// Install Android Studio version with optional prerequisite checks
    pub fn install_version_with_checks(
        &self,
        version: &str,
        full_name: &str,
        custom_dir: Option<&str>,
        run_checks: bool,
    ) -> Result<(), AstudiosError> {
        let target_dir = if let Some(dir) = custom_dir {
            PathBuf::from(dir)
        } else {
            self.applications_dir.clone()
        };

        // Run prerequisite checks if enabled
        if run_checks {
            println!("{} Running prerequisite checks...", "🔍".blue());
            let detection_result =
                SystemDetector::detect_system_requirements(&self.install_dir, &target_dir)?;

            // Display warnings if any
            if detection_result.has_warnings() {
                for warning in &detection_result.warnings {
                    println!("{} {}", "⚠️".yellow(), warning.yellow());
                }
                println!();
            }

            // Check if system meets requirements
            if !detection_result.is_valid() {
                println!(
                    "{} System does not meet installation requirements:",
                    "❌".red()
                );
                for issue in &detection_result.issues {
                    println!("  • {}", issue.red());
                }
                println!();
                println!(
                    "{} Please resolve the above issues and try again.",
                    "💡".blue()
                );
                println!("You can use --skip-checks to bypass these checks (not recommended).");

                return Err(AstudiosError::PrerequisiteNotMet(
                    "System requirements not met".to_string(),
                ));
            }

            println!("{} All prerequisite checks passed!", "✅".green());
            println!();
        }

        let download_path = self.download_version(version, full_name)?;
        let extracted_path = self.extract_archive(&download_path, version)?;
        let app_path = self.move_to_applications(version, &extracted_path, custom_dir)?;
        self.cleanup_files(&download_path, &extracted_path)?;
        self.verify_installation(&app_path)?;
        self.create_symlink(&app_path)?;
        Ok(())
    }

    /// Download a specific version
    fn download_version(&self, version: &str, full_name: &str) -> Result<PathBuf, AstudiosError> {
        use crate::list::AndroidStudioLister;

        let version_dir = self.install_dir.join(version);
        fs::create_dir_all(&version_dir)?;

        // Get the actual download URL from the API
        let lister = AndroidStudioLister::new()?;
        let releases = lister.get_releases()?;

        let target_item = releases
            .items
            .iter()
            .find(|item| item.version == version)
            .ok_or_else(|| {
                AstudiosError::VersionNotFound(format!("Version {version} not found"))
            })?;

        let download = target_item
            .get_platform_download()
            .ok_or(AstudiosError::Download(
                "No download available for current platform".to_string(),
            ))?;

        let default_filename = format!("android-studio-{version}.dmg");
        let filename = Path::new(&download.link)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&default_filename);

        let download_path = version_dir.join(filename);

        // Skip if file already exists and has content
        if download_path.exists() {
            let metadata = fs::metadata(&download_path)?;
            if metadata.len() > 0 {
                println!("Download already exists: {}", download_path.display());
                return Ok(download_path);
            }
        }

        println!("Downloading Android Studio {version}...");

        // Use the downloader to actually download the file
        let downloader = Downloader::detect_best();
        downloader.download(&download.link, &download_path, Some(full_name))?;

        println!("Download completed: {}", download_path.display());

        Ok(download_path)
    }

    /// Extract archive based on type
    fn extract_archive(
        &self,
        archive_path: &Path,
        version: &str,
    ) -> Result<PathBuf, AstudiosError> {
        let extract_dir = self.install_dir.join(version).join("extracted");
        fs::create_dir_all(&extract_dir)?;

        let archive_type = self.detect_archive_type(archive_path);

        match archive_type {
            ArchiveType::Dmg => self.extract_dmg(archive_path, &extract_dir)?,
            ArchiveType::Unsupported => {
                return Err(AstudiosError::Extraction(format!(
                    "Unsupported archive format: {}. Only DMG files are supported on macOS.",
                    archive_path.display()
                )));
            }
        }

        Ok(extract_dir)
    }

    /// Detect archive type from file extension
    fn detect_archive_type(&self, path: &Path) -> ArchiveType {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.ends_with(".dmg") {
            ArchiveType::Dmg
        } else {
            ArchiveType::Unsupported
        }
    }

    /// Extract DMG archive (macOS only)
    fn extract_dmg(&self, archive_path: &Path, destination: &Path) -> Result<(), AstudiosError> {
        let temp_mount = tempfile::tempdir()?;
        let mount_point = temp_mount.path();

        println!("Attempting to mount DMG: {}", archive_path.display());

        let output = Command::new("hdiutil")
            .args([
                "attach",
                archive_path
                    .to_str()
                    .ok_or(AstudiosError::Path("Invalid path".to_string()))?,
                "-mountpoint",
                mount_point
                    .to_str()
                    .ok_or(AstudiosError::Path("Invalid path".to_string()))?,
                "-nobrowse",
                "-noverify", // Skip verification to avoid issues
            ])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            println!("DMG mount failed: {error_msg}");
            return Err(AstudiosError::Extraction(format!(
                "Failed to mount DMG: {}",
                error_msg.trim()
            )));
        }

        println!("DMG mounted successfully at: {}", mount_point.display());

        // Find and copy app bundles
        println!("Searching for .app bundles in DMG...");
        let mut app_paths = Vec::new();

        if let Ok(entries) = fs::read_dir(mount_point) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                println!("Found item: {name_str}");

                if name_str.ends_with(".app") {
                    println!("Found .app bundle: {name_str}");
                    app_paths.push(entry.path());
                }
            }
        }

        if app_paths.is_empty() {
            // Try to find Android Studio specifically
            let android_studio_paths: Vec<PathBuf> = fs::read_dir(mount_point)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    name_str.contains("Android") && name_str.ends_with(".app")
                })
                .map(|entry| entry.path())
                .collect();

            if !android_studio_paths.is_empty() {
                app_paths = android_studio_paths;
            } else {
                // List all contents for debugging
                println!("DMG contents:");
                if let Ok(entries) = fs::read_dir(mount_point) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        println!("  {}", entry.file_name().to_string_lossy());
                    }
                }

                self.detach_dmg(mount_point)?;
                return Err(AstudiosError::Extraction(
                    "No Android Studio .app bundle found in DMG".to_string(),
                ));
            }
        }

        for app_path in app_paths {
            let app_name = app_path.file_name().unwrap();
            let dest_path = destination.join(app_name);

            let status = Command::new("cp")
                .args([
                    "-R",
                    app_path.to_str().unwrap(),
                    dest_path.to_str().unwrap(),
                ])
                .status()?;

            if !status.success() {
                self.detach_dmg(mount_point)?;
                return Err(AstudiosError::Extraction(
                    "Failed to copy app bundle".to_string(),
                ));
            }
        }

        self.detach_dmg(mount_point)?;
        Ok(())
    }

    /// Detach DMG volume
    fn detach_dmg(&self, mount_point: &Path) -> Result<(), AstudiosError> {
        let output = Command::new("hdiutil")
            .args(["detach", mount_point.to_str().unwrap(), "-force"])
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    println!("Warning: Failed to detach DMG: {}", error_msg.trim());
                } else {
                    println!("DMG detached successfully");
                }
            }
            Err(e) => {
                println!("Warning: Could not detach DMG: {e}");
            }
        }
        Ok(())
    }

    /// Move extracted Android Studio to applications directory
    fn move_to_applications(
        &self,
        version: &str,
        extracted_path: &Path,
        custom_dir: Option<&str>,
    ) -> Result<PathBuf, AstudiosError> {
        let target_dir = if let Some(dir) = custom_dir {
            PathBuf::from(dir)
        } else {
            self.applications_dir.clone()
        };

        // Ensure target directory exists
        fs::create_dir_all(&target_dir)?;

        let app_path = target_dir.join(format!("Android Studio {version}.app"));

        // Find the actual app bundle in extracted directory
        let mut app_source = None;
        if let Ok(entries) = fs::read_dir(extracted_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.contains("Android Studio") && name_str.ends_with(".app") {
                    app_source = Some(entry.path());
                    break;
                }
            }
        }

        let source = app_source.ok_or(AstudiosError::Installation(
            "Android Studio.app not found in extracted files".to_string(),
        ))?;

        println!("Installing Android Studio to: {}", app_path.display());

        // Remove existing installation if it exists
        if app_path.exists() {
            println!("Removing existing installation...");
            fs::remove_dir_all(&app_path)?;
        }

        // Copy the app bundle using a more robust approach
        // Use ditto instead of cp for better macOS app bundle handling
        let output = Command::new("ditto")
            .args([
                source
                    .to_str()
                    .ok_or(AstudiosError::Path("Invalid source path".to_string()))?,
                app_path
                    .to_str()
                    .ok_or(AstudiosError::Path("Invalid target path".to_string()))?,
            ])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(AstudiosError::Installation(format!(
                "Failed to install app bundle: {}",
                error_msg.trim()
            )));
        }

        // Verify the installation was successful
        if !app_path.exists() {
            return Err(AstudiosError::Installation(
                "Installation completed but app bundle not found at target location".to_string(),
            ));
        }

        println!("Installation completed successfully!");
        Ok(app_path)
    }

    /// Clean up temporary files
    fn cleanup_files(
        &self,
        archive_path: &Path,
        extracted_path: &Path,
    ) -> Result<(), AstudiosError> {
        if archive_path.exists() {
            fs::remove_file(archive_path)?;
        }
        if extracted_path.exists() {
            fs::remove_dir_all(extracted_path)?;
        }
        Ok(())
    }

    /// Verify installation integrity
    fn verify_installation(&self, app_path: &Path) -> Result<(), AstudiosError> {
        if !app_path.exists() {
            return Err(AstudiosError::Installation(format!(
                "Installation not found at: {}",
                app_path.display()
            )));
        }

        // Check for required directories
        let required_dirs = ["Contents", "Contents/MacOS", "Contents/Resources"];
        for dir in required_dirs {
            let path = app_path.join(dir);
            if !path.exists() {
                return Err(AstudiosError::Installation(format!(
                    "Required directory missing: {}",
                    path.display()
                )));
            }
        }

        // Verify code signing
        let status = Command::new("codesign")
            .args(["-v", app_path.to_str().unwrap()])
            .status()?;

        if !status.success() {
            return Err(AstudiosError::Installation(
                "Code signing verification failed".to_string(),
            ));
        }

        Ok(())
    }

    /// Create application symlink for version switching
    fn create_symlink(&self, app_path: &Path) -> Result<(), AstudiosError> {
        let symlink_path = self.applications_dir.join("Android Studio.app");

        println!(
            "Creating symlink: {} -> {}",
            symlink_path.display(),
            app_path.display()
        );

        // Remove existing symlink or file/directory
        if symlink_path.exists() || symlink_path.is_symlink() {
            // Check if it's a symlink (including broken symlinks)
            match fs::symlink_metadata(&symlink_path) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        println!("Removing existing symlink...");
                        fs::remove_file(&symlink_path)?;
                    } else if metadata.is_dir() {
                        println!("Removing existing directory...");
                        fs::remove_dir_all(&symlink_path)?;
                    } else {
                        println!("Removing existing file...");
                        fs::remove_file(&symlink_path)?;
                    }
                }
                Err(_) => {
                    // If we can't get metadata but the path exists, try to remove it as a file first
                    if fs::remove_file(&symlink_path).is_err() {
                        // If removing as file fails, try as directory
                        fs::remove_dir_all(&symlink_path)?;
                    }
                }
            }
        }

        // Ensure the target exists before creating symlink
        if !app_path.exists() {
            return Err(AstudiosError::Installation(format!(
                "Cannot create symlink: target does not exist: {}",
                app_path.display()
            )));
        }

        // Create new symlink (macOS/Unix)
        match std::os::unix::fs::symlink(app_path, &symlink_path) {
            Ok(_) => {
                println!("Symlink created successfully!");
                Ok(())
            }
            Err(e) => Err(AstudiosError::Installation(format!(
                "Failed to create symlink from {} to {}: {}",
                symlink_path.display(),
                app_path.display(),
                e
            ))),
        }
    }

    /// Uninstall a specific version
    pub fn uninstall_version(&self, version: &str) -> Result<(), AstudiosError> {
        let installations = self.list_installed_studios()?;

        // Find matching installations by version query
        let matching_installations: Vec<_> = installations
            .iter()
            .filter(|install| {
                // Match by short version (e.g., "2025.1")
                install.version.short_version == version ||
                // Match by build version (e.g., "AI-251.26094.121.2512.13840223")
                install.version.build_version == version ||
                // Match by identifier (same as build version)
                install.identifier() == version ||

                // Match by API version (e.g., "2025.1.3.7")
                install.get_full_version_from_api().unwrap_or(None).as_ref() == Some(&version.to_string()) ||
                // Partial match for short version (e.g., "2025.1" matches "2025.1.2")
                install.version.short_version.starts_with(version) ||

                // Partial match for API version (e.g., "2025.1.3" matches "2025.1.3.7")
                install.get_full_version_from_api().unwrap_or(None).as_ref().is_some_and(|v| v.starts_with(version))
            })
            .collect();

        if matching_installations.is_empty() {
            return Err(AstudiosError::VersionNotFound(format!(
                "Android Studio version '{version}' is not installed. Use 'astudios installed' to see available versions."
            )));
        }

        // If multiple matches, show them and ask for more specific input
        if matching_installations.len() > 1 {
            let mut error_msg = format!(
                "Multiple Android Studio installations match '{version}'. Please be more specific:\n"
            );
            for install in &matching_installations {
                let detailed_version = install.extract_detailed_version();
                error_msg.push_str(&format!(
                    "  - {}\n    Version: {} | Build: {}\n    Path: {}\n",
                    install.enhanced_display_name(),
                    detailed_version,
                    install.identifier(),
                    install.path.display()
                ));
            }
            error_msg.push_str("\nUse the full build version (e.g., 'AI-251.26094.121.2512.13840223') for exact matching.");

            return Err(AstudiosError::General(error_msg));
        }

        // Uninstall the matched version
        let installation = &matching_installations[0];
        let app_path = &installation.path;

        let detailed_version = installation.extract_detailed_version();

        println!(
            "Uninstalling {} from {}...",
            installation.enhanced_display_name().green(),
            app_path.display().to_string().dimmed()
        );
        println!(
            "Version: {} | Build: {}",
            detailed_version.cyan(),
            installation.identifier().blue()
        );

        // Check if this is the currently active version
        if let Ok(Some(active)) = self.get_active_studio()
            && active.path == *app_path
        {
            println!("Removing symlink for currently active version...");
            let symlink_path = self.applications_dir.join("Android Studio.app");
            if symlink_path.exists() || symlink_path.is_symlink() {
                fs::remove_file(&symlink_path)?;
            }
        }

        // Remove the application bundle
        if app_path.exists() {
            fs::remove_dir_all(app_path)?;
            println!("Removed application bundle: {}", app_path.display());
        }

        // Remove from install directory if it exists
        // Try to match by short version first, then by build version
        let possible_version_dirs = vec![
            self.install_dir.join(&installation.version.short_version),
            self.install_dir.join(&installation.version.build_version),
            self.install_dir.join(version), // Original input
        ];

        for version_dir in possible_version_dirs {
            if version_dir.exists() {
                fs::remove_dir_all(&version_dir)?;
                println!("Removed installation files: {}", version_dir.display());
                break;
            }
        }

        Ok(())
    }

    /// List all installed Android Studio instances
    pub fn list_installed_studios(&self) -> Result<Vec<InstalledAndroidStudio>, AstudiosError> {
        let mut installations = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.applications_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();

                // Check if it's an Android Studio app bundle
                if name.contains("Android Studio") && name.ends_with(".app") {
                    // Skip any other symlinks to avoid duplicates
                    if path.is_symlink() {
                        continue;
                    }

                    if let Ok(Some(installed)) = InstalledAndroidStudio::new(path) {
                        installations.push(installed);
                    }
                }
            }
        }

        // Sort by version (newest first)
        installations.sort_by(|a, b| b.cmp(a));
        Ok(installations)
    }

    /// List all installed versions (legacy compatibility)
    pub fn list_installed_versions(&self) -> Result<Vec<String>, AstudiosError> {
        let installations = self.list_installed_studios()?;
        Ok(installations
            .into_iter()
            .map(|install| install.version.short_version)
            .collect())
    }

    /// Get currently active Android Studio installation
    pub fn get_active_studio(&self) -> Result<Option<InstalledAndroidStudio>, AstudiosError> {
        let symlink_path = self.applications_dir.join("Android Studio.app");

        if symlink_path.exists()
            && symlink_path.is_symlink()
            && let Ok(target) = fs::read_link(&symlink_path)
            && let Ok(Some(installed)) = InstalledAndroidStudio::new(target)
        {
            return Ok(Some(installed));
        }

        Ok(None)
    }

    /// Get currently active version (legacy compatibility)
    pub fn get_active_version(&self) -> Result<Option<String>, AstudiosError> {
        if let Some(active) = self.get_active_studio()? {
            Ok(Some(active.version.short_version))
        } else {
            Ok(None)
        }
    }

    /// Switch to a different Android Studio installation by identifier
    pub fn switch_to_studio(&self, identifier: &str) -> Result<(), AstudiosError> {
        let installations = self.list_installed_studios()?;

        // Find installation by various version identifiers
        let target_installation = installations
            .iter()
            .find(|install| {
                // Match by build version (e.g., "AI-251.26094.121.2513.14007798")
                install.identifier() == identifier ||
                // Match by short version (e.g., "2025.1")
                install.version.short_version == identifier ||

                // Match by API version (e.g., "2025.1.3.7")
                install.get_full_version_from_api().unwrap_or(None).as_ref() == Some(&identifier.to_string()) ||
                // Partial match for short version (e.g., "2025.1" matches "2025.1.2")
                install.version.short_version.starts_with(identifier) ||

                // Partial match for API version (e.g., "2025.1.3" matches "2025.1.3.7")
                install.get_full_version_from_api().unwrap_or(None).as_ref().is_some_and(|v| v.starts_with(identifier))
            })
            .ok_or_else(|| {
                AstudiosError::VersionNotFound(format!(
                    "Android Studio with identifier '{identifier}' is not installed.\nUse 'astudios installed' to see installed versions or 'astudios install {identifier}' to install it."
                ))
            })?;

        self.create_symlink(&target_installation.path)?;
        Ok(())
    }

    /// Switch to a different version (legacy compatibility)
    pub fn switch_to_version(&self, version: &str) -> Result<(), AstudiosError> {
        self.switch_to_studio(version)
    }
}
