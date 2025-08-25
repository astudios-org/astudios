use crate::api::ApiClient;
use crate::model::Item;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let install_dir = home_dir.join(".as-man").join("versions");

        // Create directory if it doesn't exist
        fs::create_dir_all(&install_dir)?;

        Ok(Self { install_dir })
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
            .ok_or_else(|| format!("Version {} not found", version))?;

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
            return Err(format!("Version {} is already installed", version).into());
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

    fn get_platform_download<'a>(
        &self,
        item: &'a Item,
    ) -> Result<&'a crate::model::Download, Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        let download = item
            .get_macos_download()
            .ok_or("macOS download not available for this version")?;

        #[cfg(target_os = "windows")]
        let download = item
            .get_windows_download()
            .ok_or("Windows download not available for this version")?;

        #[cfg(target_os = "linux")]
        let download = item
            .get_linux_download()
            .ok_or("Linux download not available for this version")?;

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let download = return Err("Unsupported platform".into());

        Ok(download)
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

    fn extract_archive(
        &self,
        archive_path: &Path,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_name = archive_path.file_name()
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
            _ => return Err(format!("Unsupported archive format: {}", archive_type).into()),
        }
        
        // Verify installation
        self.verify_installation(destination)?;
        
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
                "-quiet"
            ])
            .status()?;
            
        if !status.success() {
            return Err("Failed to mount DMG".into());
        }
        
        // Find the app bundle in the mounted DMG
        let app_paths: Vec<PathBuf> = fs::read_dir(mount_point)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().ends_with(".app")
            })
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
            
            println!("{} Copying {:?} to {:?}", "📁".blue(), app_name, destination);
            
            // Use rsync or cp for copying app bundles
            let status = std::process::Command::new("cp")
                .args([
                    "-R",
                    app_path.to_str().unwrap(),
                    dest_path.to_str().unwrap()
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

    fn verify_installation(
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
            println!("{} Android Studio installation files may need manual arrangement", "⚠".yellow());
        }
        
        Ok(())
    }
}
