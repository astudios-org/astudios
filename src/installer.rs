use crate::api::ApiClient;
use crate::model::AndroidStudio;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Installer {
    install_dir: PathBuf,
    applications_dir: PathBuf,
}

impl Installer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let install_dir = home_dir.join(".as-man").join("versions");
        let applications_dir = PathBuf::from("/Applications");

        // Create directories if they don't exist
        fs::create_dir_all(&install_dir)?;

        Ok(Self {
            install_dir,
            applications_dir,
        })
    }

    pub fn install_version(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = ApiClient::new()?;

        // Fetch releases
        let content = client.fetch_releases()?;

        // Find the requested version
        let item = content
            .items
            .iter()
            .find(|item| item.version == version)
            .ok_or_else(|| format!("Version {version} not found"))?;

        // Get appropriate download for current platform
        let download = self.get_platform_download(item)?;

        println!(
            "{} Installing Android Studio {}",
            "🚀".blue(),
            version.green().bold()
        );

        // Create version directory
        let version_dir = self.install_dir.join(version);
        if version_dir.exists() {
            return Err(format!("Version {version} is already installed").into());
        }

        fs::create_dir_all(&version_dir)?;

        // Download the file
        let filename = Path::new(&download.link)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("android-studio.zip");

        let download_path = version_dir.join(filename);

        self.download_file(&download.link, &download_path)?;

        println!("{} Download complete!", "✅".green());

        // Extract the downloaded file
        self.extract_archive(&download_path, &version_dir)?;

        // Clean up the archive file
        fs::remove_file(&download_path)?;

        println!(
            "{} Android Studio {} installed successfully!",
            "✅".green(),
            version
        );
        println!("  Location: {}", version_dir.display());

        Ok(())
    }

    pub fn install_version_with_progress(
        &self,
        version: &str,
        full_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = ApiClient::new()?;

        // Fetch releases
        let content = client.fetch_releases()?;

        // Find the requested version
        let item = content
            .items
            .iter()
            .find(|item| item.version == version)
            .ok_or_else(|| format!("Version {version} not found"))?;

        // Get appropriate download for current platform
        let download = self.get_platform_download(item)?;

        // Create version directory
        let version_dir = self.install_dir.join(version);
        if version_dir.exists() {
            return Err(format!("Version {version} is already installed").into());
        }

        fs::create_dir_all(&version_dir)?;

        // Download the file
        let filename = Path::new(&download.link)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("android-studio.zip");

        let download_path = version_dir.join(filename);

        self.download_file_with_progress(&download.link, &download_path, full_name)?;

        // Extract the downloaded file
        self.extract_archive(&download_path, &version_dir)?;

        // Keep the archive file for now - will be cleaned up later
        // Store path for cleanup
        self.store_archive_path(version, &download_path)?;

        Ok(())
    }

    fn get_platform_download<'a>(
        &self,
        item: &'a AndroidStudio,
    ) -> Result<&'a crate::model::Download, Box<dyn std::error::Error>> {
        item.get_macos_download()
            .ok_or("macOS download not available for this version".into())
    }

    fn download_file(
        &self,
        url: &str,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        let mut response = client.get(url).send()?;
        let total_size = response.content_length().unwrap_or(0);

        let pb = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("█▉▊▋▌▍▎▏ "),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} Downloading... {bytes} bytes")
                    .unwrap(),
            );
            pb
        };

        let mut file = fs::File::create(destination)?;
        let mut downloaded = 0u64;
        let mut buf = [0u8; 8192];

        loop {
            let n = response.read(&mut buf)?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])?;
            downloaded += n as u64;
            pb.set_position(downloaded);
        }

        pb.finish_and_clear();

        Ok(())
    }

    fn download_file_with_progress(
        &self,
        url: &str,
        destination: &Path,
        full_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        let mut response = client.get(url).send()?;
        let total_size = response.content_length().unwrap_or(0);

        let pb = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("█▉▊▋▌▍▎▏ "),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} Downloading... {bytes} bytes")
                    .unwrap(),
            );
            pb
        };

        let mut file = fs::File::create(destination)?;
        let mut downloaded = 0u64;
        let mut buf = [0u8; 8192];

        loop {
            let n = response.read(&mut buf)?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])?;
            downloaded += n as u64;
            pb.set_position(downloaded);

            // Print progress percentage for step format
            if total_size > 0 {
                let percentage = (downloaded as f64 / total_size as f64) * 100.0;
                print!("\r(1/6) Downloading {full_name}: {percentage:.0}%");
                if percentage >= 100.0 {
                    println!();
                }
            }
        }

        pb.finish_and_clear();

        Ok(())
    }

    fn store_archive_path(
        &self,
        version: &str,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let version_dir = self.install_dir.join(version);
        let archive_info_path = version_dir.join(".archive_path");
        fs::write(archive_info_path, path.to_string_lossy().as_bytes())?;
        Ok(())
    }

    pub fn cleanup_archive(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        let version_dir = self.install_dir.join(version);
        let archive_info_path = version_dir.join(".archive_path");

        if archive_info_path.exists() {
            let archive_path = fs::read_to_string(&archive_info_path)?;
            let path = PathBuf::from(archive_path.trim());

            if path.exists() {
                // Move to trash using system-specific commands
                #[cfg(target_os = "macos")]
                {
                    std::process::Command::new("osascript")
                        .args([
                            "-e",
                            &format!(
                                "tell app \"Finder\" to delete POSIX file \"{}\"",
                                path.display()
                            ),
                        ])
                        .output()?;
                }

                #[cfg(target_os = "linux")]
                {
                    // Try gio trash first, then trash-cli, then fallback to rm
                    let _ = std::process::Command::new("gio")
                        .args(&["trash", path.to_str().unwrap()])
                        .output()
                        .or_else(|_| {
                            std::process::Command::new("trash")
                                .arg(path.to_str().unwrap())
                                .output()
                        })
                        .or_else(|_| {
                            std::process::Command::new("rm")
                                .arg(path.to_str().unwrap())
                                .output()
                        });
                }

                #[cfg(target_os = "windows")]
                {
                    use std::process::Command;
                    let _ = Command::new("powershell")
                        .args(&["-Command", "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile('", path.to_str().unwrap(), "','OnlyErrorDialogs','SendToRecycleBin')"])
                        .output();
                }
            }

            fs::remove_file(archive_info_path)?;
        }

        Ok(())
    }

    fn extract_archive(
        &self,
        archive_path: &Path,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_name = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let extension = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // Determine archive type based on full filename
        let archive_type = if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            "tar.gz"
        } else if file_name.ends_with(".dmg") {
            "dmg"
        } else if extension == "zip" {
            "zip"
        } else if extension == "tar" {
            "tar"
        } else {
            extension
        };

        println!("{} Detected archive type: {}", "📦".blue(), archive_type);

        match archive_type {
            "zip" => self.extract_zip(archive_path, destination)?,
            "tar.gz" | "tgz" => self.extract_tar_gz(archive_path, destination)?,
            "tar" => self.extract_tar(archive_path, destination)?,
            "dmg" => self.extract_dmg(archive_path, destination)?,
            _ => return Err(format!("Unsupported archive format: {archive_type}").into()),
        }

        // Verify installation
        self.verify_extracted_installation(destination)?;

        Ok(())
    }

    fn extract_zip(
        &self,
        archive_path: &Path,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{} Extracting ZIP archive...", "📦".blue());

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

    fn extract_dmg(
        &self,
        archive_path: &Path,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{} Extracting DMG archive...", "📦".blue());

        // For macOS, we'll use the system tools to extract DMG
        // First, try using hdiutil to mount the DMG
        let temp_mount = tempfile::tempdir()?;
        let mount_point = temp_mount.path();

        let status = std::process::Command::new("hdiutil")
            .args([
                "attach",
                archive_path.to_str().unwrap(),
                "-mountpoint",
                mount_point.to_str().unwrap(),
                "-nobrowse",
                "-quiet",
            ])
            .status()?;

        if !status.success() {
            return Err("Failed to mount DMG".into());
        }

        // Find the app bundle in the mounted DMG
        let app_paths: Vec<PathBuf> = fs::read_dir(mount_point)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".app"))
            .map(|entry| entry.path())
            .collect();

        if app_paths.is_empty() {
            // Unmount the DMG
            std::process::Command::new("hdiutil")
                .args(["detach", mount_point.to_str().unwrap(), "-quiet"])
                .status()?;
            return Err("No .app bundle found in DMG".into());
        }

        // Copy the app bundle to destination
        for app_path in app_paths {
            let app_name = app_path.file_name().unwrap();
            let dest_path = destination.join(app_name);

            println!(
                "{} Copying {:?} to {:?}",
                "📁".blue(),
                app_name,
                destination
            );

            // Use rsync or cp for copying app bundles
            let status = std::process::Command::new("cp")
                .args([
                    "-R",
                    app_path.to_str().unwrap(),
                    dest_path.to_str().unwrap(),
                ])
                .status()?;

            if !status.success() {
                return Err("Failed to copy app bundle".into());
            }
        }

        // Unmount the DMG
        std::process::Command::new("hdiutil")
            .args(["detach", mount_point.to_str().unwrap(), "-quiet"])
            .status()?;

        Ok(())
    }

    fn extract_tar(
        &self,
        archive_path: &Path,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{} Extracting TAR archive...", "📦".blue());

        let file = fs::File::open(archive_path)?;
        let mut archive = tar::Archive::new(file);

        archive.unpack(destination)?;

        Ok(())
    }

    fn extract_tar_gz(
        &self,
        archive_path: &Path,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{} Extracting TAR.GZ archive...", "📦".blue());

        let file = fs::File::open(archive_path)?;
        let tar = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);

        archive.unpack(destination)?;

        Ok(())
    }

    fn verify_extracted_installation(
        &self,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{} Verifying installation...", "🔍".blue());

        let mut found = false;

        if let Ok(entries) = fs::read_dir(destination) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Check for Android Studio app bundle (macOS)
                if name_str.contains("Android Studio.app") {
                    println!("{} Found Android Studio.app", "✅".green());
                    found = true;
                    break;
                }

                // Check for android-studio directory (Linux/Windows)
                if name_str.contains("android-studio") {
                    let studio_dir = entry.path();
                    if studio_dir.join("bin").exists() {
                        println!("{} Found android-studio with bin directory", "✅".green());
                        found = true;
                        break;
                    }
                }
            }
        }

        if !found {
            // List what was actually extracted
            if let Ok(entries) = fs::read_dir(destination) {
                let items: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();

                if items.is_empty() {
                    return Err("No files were extracted".into());
                } else {
                    println!("{} Extracted files: {}", "📋".yellow(), items.join(", "));
                }
            }

            // Don't fail, just warn - the user might need to move files manually
            println!(
                "{} Android Studio installation files may need manual arrangement",
                "⚠".yellow()
            );
        }

        Ok(())
    }

    // macOS-specific installation and management methods
    pub fn install_macos(
        &self,
        version: &str,
        full_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let version_dir = self.install_dir.join(version);

        if !version_dir.exists() {
            return Err(format!("Version {version} not found in ~/.as-man/versions/").into());
        }

        // Find the Android Studio bundle (either Android Studio.app or Android Studio Preview.app)
        let mut app_bundle = version_dir.join("Android Studio.app");

        if !app_bundle.exists() {
            app_bundle = version_dir.join("Android Studio Preview.app");
        }

        if !app_bundle.exists() {
            // Check if it's in a subdirectory
            let app_bundles: Vec<PathBuf> = self.find_files(&version_dir, ".app");
            if app_bundles.is_empty() {
                return Err("No Android Studio.app bundle found".into());
            }
            app_bundle = app_bundles[0].clone();
        }

        // Create versioned app name with full name
        let safe_name = full_name.replace(" | ", " ").replace(" ", " ");
        let versioned_dest_path = self.applications_dir.join(format!("{safe_name}.app"));
        let symlink_path = self.applications_dir.join("Android Studio.app");

        // Remove existing versioned installation if it exists
        if versioned_dest_path.exists() {
            fs::remove_dir_all(&versioned_dest_path)?;
        }

        // Use ditto to install the app bundle
        let status = Command::new("ditto")
            .arg(&app_bundle)
            .arg(&versioned_dest_path)
            .status()?;

        if !status.success() {
            return Err("Failed to install Android Studio using ditto".into());
        }

        // Create/update symlink to point to this version
        if symlink_path.exists() {
            if symlink_path.is_symlink() {
                fs::remove_file(&symlink_path)?;
            } else {
                // If it's a real directory, remove it
                fs::remove_dir_all(&symlink_path)?;
            }
        }

        std::os::unix::fs::symlink(&versioned_dest_path, &symlink_path)?;

        Ok(())
    }

    pub fn install_macos_to_directory(
        &self,
        version: &str,
        full_name: &str,
        custom_dir: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let version_dir = self.install_dir.join(version);

        if !version_dir.exists() {
            return Err(format!("Version {version} not found in ~/.as-man/versions/").into());
        }

        let custom_path = PathBuf::from(custom_dir);

        // Find the Android Studio bundle (either Android Studio.app or Android Studio Preview.app)
        let mut app_bundle = version_dir.join("Android Studio.app");

        if !app_bundle.exists() {
            app_bundle = version_dir.join("Android Studio Preview.app");
        }

        if !app_bundle.exists() {
            // Check if it's in a subdirectory
            let app_bundles: Vec<PathBuf> = self.find_files(&version_dir, ".app");
            if app_bundles.is_empty() {
                return Err("No Android Studio.app bundle found".into());
            }
            app_bundle = app_bundles[0].clone();
        }

        // Create custom directory if it doesn't exist
        fs::create_dir_all(&custom_path)?;

        // Create versioned app name with full name
        let safe_name = full_name.replace(" | ", " ").replace(" ", " ");
        let versioned_dest_path = custom_path.join(format!("{safe_name}.app"));

        // Remove existing versioned installation if it exists
        if versioned_dest_path.exists() {
            fs::remove_dir_all(&versioned_dest_path)?;
        }

        // Use ditto to install the app bundle
        let status = Command::new("ditto")
            .arg(&app_bundle)
            .arg(&versioned_dest_path)
            .status()?;

        if !status.success() {
            return Err("Failed to install Android Studio using ditto".into());
        }

        Ok(())
    }

    pub fn uninstall_version(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        let versioned_path = self
            .applications_dir
            .join(format!("Android Studio-{version}.app"));

        // Check if the version is installed in Applications
        let installed_in_apps = versioned_path.exists();

        // Check if the version exists in ~/.as-man/versions
        let version_dir = self.install_dir.join(version);
        let installed_in_versions = version_dir.exists();

        if !installed_in_apps && !installed_in_versions {
            return Err(format!("Android Studio {version} is not installed").into());
        }

        println!("{} Removing Android Studio {}...", "🗑".yellow(), version);

        // Check if this is the currently active version
        let is_active = self.is_version_active(version)?;
        let symlink_path = self.applications_dir.join("Android Studio.app");

        // If this is the active version, remove the symlink
        if is_active {
            if symlink_path.exists() {
                if symlink_path.is_symlink() {
                    fs::remove_file(&symlink_path)?;
                    println!("{} Removed active version symlink", "ℹ".blue());
                } else {
                    fs::remove_dir_all(&symlink_path)?;
                }
            }
            println!("{} Android Studio is no longer available", "⚠".yellow());
        }

        // Remove the versioned installation from Applications
        if installed_in_apps {
            fs::remove_dir_all(&versioned_path)?;
            println!("{} Removed from Applications directory", "✅".green());
        }

        // Remove the downloaded version from ~/.as-man/versions
        if installed_in_versions {
            fs::remove_dir_all(&version_dir)?;
            println!("{} Removed downloaded version files", "✅".green());
        }

        println!(
            "{} Successfully uninstalled Android Studio {}",
            "✅".green(),
            version
        );

        Ok(())
    }

    pub fn switch_to_version(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        let versioned_path = self
            .applications_dir
            .join(format!("Android Studio-{version}.app"));

        if !versioned_path.exists() {
            return Err(format!("Android Studio {version} is not installed").into());
        }

        println!("{} Switching to Android Studio {}...", "🔀".blue(), version);

        let symlink_path = self.applications_dir.join("Android Studio.app");

        // Remove existing symlink
        if symlink_path.exists() {
            if symlink_path.is_symlink() {
                fs::remove_file(&symlink_path)?;
            } else {
                fs::remove_dir_all(&symlink_path)?;
            }
        }

        std::os::unix::fs::symlink(&versioned_path, &symlink_path)?;

        println!("{} Now using Android Studio {}", "✅".green(), version);
        println!("  Symlink: {}", symlink_path.display());
        println!("  Points to: {}", versioned_path.display());

        Ok(())
    }

    pub fn list_installed_versions(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{} Installed Android Studio versions:", "📋".green().bold());
        println!();

        let mut found = false;

        if let Ok(entries) = fs::read_dir(&self.applications_dir) {
            let mut versions = Vec::new();

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

            if !versions.is_empty() {
                found = true;
                versions.sort();

                for version in versions {
                    let is_active = self.is_version_active(&version)?;
                    let indicator = if is_active { "✅" } else { "  " };
                    println!("{indicator} Android Studio-{version}");
                }
            }
        }

        if !found {
            println!("{} No Android Studio versions installed", "⚠".yellow());
            println!();
            println!("Use 'as-man install <version>' to install a version");
        }

        Ok(())
    }

    pub fn show_current_version(&self) -> Result<(), Box<dyn std::error::Error>> {
        let symlink_path = self.applications_dir.join("Android Studio.app");

        if symlink_path.exists() {
            if symlink_path.is_symlink() {
                if let Ok(target) = fs::read_link(&symlink_path) {
                    let target_name = target
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    let version = target_name
                        .strip_prefix("Android Studio-")
                        .and_then(|s| s.strip_suffix(".app"));

                    if let Some(version) = version {
                        println!(
                            "{} Currently using Android Studio {}",
                            "✅".green(),
                            version
                        );
                        println!("  Symlink: {}", symlink_path.display());
                        println!("  Points to: {}", target.display());
                    } else {
                        println!("{} Symlink points to: {}", "ℹ".blue(), target.display());
                    }
                } else {
                    println!("{} Could not read symlink target", "⚠".yellow());
                }
            } else {
                println!(
                    "{} Android Studio is installed as a regular directory (not symlink)",
                    "ℹ".blue()
                );
            }
        } else {
            println!(
                "{} Android Studio is not installed or symlink is missing",
                "⚠".yellow()
            );
            println!();
            println!("Use 'as-man install <version>' to install a version");
        }

        Ok(())
    }

    pub fn verify_installation(
        &self,
        version: &str,
        full_name: &str,
        custom_dir: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let safe_name = full_name.replace(" | ", " ").replace(" ", " ");
        let target_dir = if let Some(dir) = custom_dir {
            PathBuf::from(dir)
        } else {
            self.applications_dir.clone()
        };

        let app_path = target_dir.join(format!("{safe_name}.app"));

        if !app_path.exists() {
            return Err(format!(
                "Failed to verify installation: {} not found",
                app_path.display()
            )
            .into());
        }

        // Check code signing
        let status = Command::new("codesign").arg("-v").arg(&app_path).status()?;

        if !status.success() {
            println!("⚠️  Code signing verification failed, but installation completed");
        }

        Ok(())
    }

    fn is_version_active(&self, version: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let symlink_path = self.applications_dir.join("Android Studio.app");

        if symlink_path.exists() && symlink_path.is_symlink() {
            if let Ok(target) = fs::read_link(&symlink_path) {
                let target_name = target.file_name().and_then(|n| n.to_str()).unwrap_or("");

                let expected_suffix = format!("Android Studio-{version}.app");
                return Ok(target_name == expected_suffix);
            }
        }

        Ok(false)
    }

    fn find_files(&self, dir: &Path, extension: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().contains(extension) {
                            files.push(path);
                        }
                    }
                } else if path.is_dir() {
                    files.extend(self.find_files(&path, extension));
                }
            }
        }
        files
    }
}
