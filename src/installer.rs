use crate::{config::Config, downloader::Downloader, error::AsManError};
use colored::Colorize;
use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
    process::Command,
};

/// Archive extraction support
#[derive(Debug, Clone, Copy)]
pub enum ArchiveType {
    Zip,
    Tar,
    TarGz,
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
    pub fn new() -> Result<Self, AsManError> {
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
    ) -> Result<Self, AsManError> {
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
    ) -> Result<(), AsManError> {
        let download_path = self.download_version(version, full_name)?;
        let extracted_path = self.extract_archive(&download_path, version)?;
        let app_path = self.move_to_applications(version, &extracted_path, custom_dir)?;
        self.cleanup_files(&download_path, &extracted_path)?;
        self.verify_installation(&app_path)?;
        self.create_symlink(version, &app_path)?;
        Ok(())
    }

    /// Download a specific version
    fn download_version(&self, version: &str, full_name: &str) -> Result<PathBuf, AsManError> {
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
            .ok_or_else(|| AsManError::VersionNotFound(format!("Version {} not found", version)))?;

        let download = target_item
            .get_platform_download()
            .ok_or(AsManError::Download(
                "No download available for current platform".to_string(),
            ))?;

        let default_filename = format!("android-studio-{}.dmg", version);
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

        println!("Downloading Android Studio {}...", version);

        // Use the downloader to actually download the file
        let downloader = Downloader::detect_best();
        downloader.download(&download.link, &download_path, Some(full_name))?;

        println!("Download completed: {}", download_path.display());

        Ok(download_path)
    }

    /// Extract archive based on type
    fn extract_archive(&self, archive_path: &Path, version: &str) -> Result<PathBuf, AsManError> {
        let extract_dir = self.install_dir.join(version).join("extracted");
        fs::create_dir_all(&extract_dir)?;

        let archive_type = self.detect_archive_type(archive_path);

        match archive_type {
            ArchiveType::Zip => self.extract_zip(archive_path, &extract_dir)?,
            ArchiveType::Tar => self.extract_tar(archive_path, &extract_dir)?,
            ArchiveType::TarGz => self.extract_tar_gz(archive_path, &extract_dir)?,
            ArchiveType::Dmg => self.extract_dmg(archive_path, &extract_dir)?,
            ArchiveType::Unsupported => {
                return Err(AsManError::Extraction(format!(
                    "Unsupported archive format: {}",
                    archive_path.display()
                )));
            }
        }

        Ok(extract_dir)
    }

    /// Detect archive type from file extension
    fn detect_archive_type(&self, path: &Path) -> ArchiveType {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            ArchiveType::TarGz
        } else if file_name.ends_with(".dmg") {
            ArchiveType::Dmg
        } else if extension == "zip" {
            ArchiveType::Zip
        } else if extension == "tar" {
            ArchiveType::Tar
        } else {
            ArchiveType::Unsupported
        }
    }

    /// Extract ZIP archive
    fn extract_zip(&self, archive_path: &Path, destination: &Path) -> Result<(), AsManError> {
        let file = fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => destination.join(path),
                None => continue,
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = fs::File::create(outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    /// Extract TAR archive
    fn extract_tar(&self, archive_path: &Path, destination: &Path) -> Result<(), AsManError> {
        let file = fs::File::open(archive_path)?;
        let mut archive = tar::Archive::new(file);
        archive.unpack(destination)?;
        Ok(())
    }

    /// Extract TAR.GZ archive
    fn extract_tar_gz(&self, archive_path: &Path, destination: &Path) -> Result<(), AsManError> {
        let file = fs::File::open(archive_path)?;
        let tar = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(destination)?;
        Ok(())
    }

    /// Extract DMG archive (macOS only)
    #[cfg(target_os = "macos")]
    fn extract_dmg(&self, archive_path: &Path, destination: &Path) -> Result<(), AsManError> {
        let temp_mount = tempfile::tempdir()?;
        let mount_point = temp_mount.path();

        println!("Attempting to mount DMG: {}", archive_path.display());

        let output = Command::new("hdiutil")
            .args([
                "attach",
                archive_path
                    .to_str()
                    .ok_or(AsManError::Path("Invalid path".to_string()))?,
                "-mountpoint",
                mount_point
                    .to_str()
                    .ok_or(AsManError::Path("Invalid path".to_string()))?,
                "-nobrowse",
                "-noverify", // Skip verification to avoid issues
            ])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            println!("DMG mount failed: {}", error_msg);
            return Err(AsManError::Extraction(format!(
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
                println!("Found item: {}", name_str);

                if name_str.ends_with(".app") {
                    println!("Found .app bundle: {}", name_str);
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
                return Err(AsManError::Extraction(
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
                return Err(AsManError::Extraction(
                    "Failed to copy app bundle".to_string(),
                ));
            }
        }

        self.detach_dmg(mount_point)?;
        Ok(())
    }

    /// Extract DMG on non-macOS platforms (placeholder)
    #[cfg(not(target_os = "macos"))]
    fn extract_dmg(&self, _archive_path: &Path, _destination: &Path) -> Result<(), AsManError> {
        Err(AsManError::Platform(
            "DMG extraction only supported on macOS".to_string(),
        ))
    }

    /// Detach DMG volume
    #[cfg(target_os = "macos")]
    fn detach_dmg(&self, mount_point: &Path) -> Result<(), AsManError> {
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
                println!("Warning: Could not detach DMG: {}", e);
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
    ) -> Result<PathBuf, AsManError> {
        let target_dir = if let Some(dir) = custom_dir {
            PathBuf::from(dir)
        } else {
            self.applications_dir.clone()
        };

        let app_path = target_dir.join(format!("Android Studio {}.app", version));

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

        let source = app_source.ok_or(AsManError::Installation(
            "Android Studio.app not found in extracted files".to_string(),
        ))?;

        // Remove existing installation
        if app_path.exists() {
            fs::remove_dir_all(&app_path)?;
        }

        // Copy the app bundle
        let status = Command::new("cp")
            .args(["-R", source.to_str().unwrap(), app_path.to_str().unwrap()])
            .status()?;

        if !status.success() {
            return Err(AsManError::Installation(
                "Failed to move app to applications".to_string(),
            ));
        }

        Ok(app_path)
    }

    /// Clean up temporary files
    fn cleanup_files(&self, archive_path: &Path, extracted_path: &Path) -> Result<(), AsManError> {
        if archive_path.exists() {
            fs::remove_file(archive_path)?;
        }
        if extracted_path.exists() {
            fs::remove_dir_all(extracted_path)?;
        }
        Ok(())
    }

    /// Verify installation integrity
    fn verify_installation(&self, app_path: &Path) -> Result<(), AsManError> {
        if !app_path.exists() {
            return Err(AsManError::Installation(format!(
                "Installation not found at: {}",
                app_path.display()
            )));
        }

        // Check for required directories
        let required_dirs = ["Contents", "Contents/MacOS", "Contents/Resources"];
        for dir in required_dirs {
            let path = app_path.join(dir);
            if !path.exists() {
                return Err(AsManError::Installation(format!(
                    "Required directory missing: {}",
                    path.display()
                )));
            }
        }

        // Verify code signing on macOS
        #[cfg(target_os = "macos")]
        {
            let status = Command::new("codesign")
                .args(["-v", app_path.to_str().unwrap()])
                .status()?;

            if !status.success() {
                return Err(AsManError::Installation(
                    "Code signing verification failed".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Create application symlink for version switching
    fn create_symlink(&self, version: &str, app_path: &Path) -> Result<(), AsManError> {
        let symlink_path = self.applications_dir.join("Android Studio.app");

        // Remove existing symlink
        if symlink_path.exists() {
            if symlink_path.is_symlink() {
                fs::remove_file(&symlink_path)?;
            } else {
                fs::remove_dir_all(&symlink_path)?;
            }
        }

        // Create new symlink
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(app_path, &symlink_path)?;
        }

        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(app_path, &symlink_path)?;
        }

        Ok(())
    }

    /// Uninstall a specific version
    pub fn uninstall_version(&self, version: &str) -> Result<(), AsManError> {
        let app_path = self
            .applications_dir
            .join(format!("Android Studio-{}.app", version));

        if app_path.exists() {
            fs::remove_dir_all(&app_path)?;
        }

        // Remove from install directory
        let version_dir = self.install_dir.join(version);
        if version_dir.exists() {
            fs::remove_dir_all(&version_dir)?;
        }

        Ok(())
    }

    /// List all installed versions
    pub fn list_installed_versions(&self) -> Result<Vec<String>, AsManError> {
        let mut versions = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.applications_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("Android Studio-") && name_str.ends_with(".app") {
                    if let Some(version) = name_str
                        .strip_prefix("Android Studio-")
                        .and_then(|s| s.strip_suffix(".app"))
                    {
                        versions.push(version.to_string());
                    }
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Get currently active version
    pub fn get_active_version(&self) -> Result<Option<String>, AsManError> {
        let symlink_path = self.applications_dir.join("Android Studio.app");

        if symlink_path.exists() && symlink_path.is_symlink() {
            if let Ok(target) = fs::read_link(&symlink_path) {
                let target_name = target.file_name().and_then(|n| n.to_str()).unwrap_or("");

                return Ok(target_name
                    .strip_prefix("Android Studio-")
                    .and_then(|s| s.strip_suffix(".app"))
                    .map(|s| s.to_string()));
            }
        }

        Ok(None)
    }

    /// Switch to a different version
    pub fn switch_to_version(&self, version: &str) -> Result<(), AsManError> {
        let versions = self.list_installed_versions()?;
        if !versions.contains(&version.to_string()) {
            return Err(AsManError::VersionNotFound(format!(
                "Version {} is not installed",
                version
            )));
        }

        let app_path = self
            .applications_dir
            .join(format!("Android Studio-{}.app", version));
        self.create_symlink(version, &app_path)?;

        Ok(())
    }

    /// Sanitize application name for filesystem
    fn sanitize_app_name(&self, name: &str) -> String {
        name.replace(" | ", " ")
            .replace(" ", "-")
            .replace(".", "")
            .replace("(", "")
            .replace(")", "")
            .to_lowercase()
    }
}
