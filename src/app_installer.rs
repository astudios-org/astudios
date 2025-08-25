use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct AppInstaller {
    applications_dir: PathBuf,
    versions_dir: PathBuf,
}

impl AppInstaller {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let applications_dir = PathBuf::from("/Applications");
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let versions_dir = home_dir.join(".as-man").join("versions");
        
        Ok(Self {
            applications_dir,
            versions_dir,
        })
    }

    pub fn install_application(
        &self,
        version: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let version_dir = self.versions_dir.join(version);
        
        if !version_dir.exists() {
            return Err(format!("Version {} not found in ~/.as-man/versions/", version).into());
        }
        
        #[cfg(target_os = "macos")]
        return self.install_macos(version, &version_dir);
        
        #[cfg(target_os = "linux")]
        return self.install_linux(version, &version_dir);
        
        #[cfg(target_os = "windows")]
        return self.install_windows(version, &version_dir);
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        Err("Unsupported platform".into())
    }

    #[cfg(target_os = "macos")]
    pub fn install_macos(
        &self,
        version: &str,
        version_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Find the Android Studio.app bundle
        let mut app_bundle = version_dir.join("Android Studio.app");
        
        if !app_bundle.exists() {
            // Check if it's in a subdirectory
            let app_bundles: Vec<PathBuf> = Self::find_files(version_dir, ".app");
            if app_bundles.is_empty() {
                return Err("No Android Studio.app bundle found".into());
            }
            app_bundle = app_bundles[0].clone();
        }
        
        // Create versioned app name
        let versioned_dest_path = self.applications_dir
            .join(format!("Android Studio-{}.app", version));
        let symlink_path = self.applications_dir.join("Android Studio.app");
        
        println!("{} Installing Android Studio {} to Applications...", "🚀".blue(), version);
        
        // Remove existing versioned installation if it exists
        if versioned_dest_path.exists() {
            println!("{} Removing existing Android Studio-{}.app...", "🗑".yellow(), version);
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
        
        #[cfg(unix)]
        std::os::unix::fs::symlink(&versioned_dest_path, &symlink_path)?;
        
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&versioned_dest_path, &symlink_path)?;
        
        println!("{} Android Studio {} installed to Applications!", "✅".green(), version);
        println!("  Versioned: {}", versioned_dest_path.display());
        println!("  Symlink: {}", symlink_path.display());
        
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn install_linux(
        &self,
        version: &str,
        version_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut studio_dir = version_dir.join("android-studio");
        
        if !studio_dir.exists() {
            // Check if it's in a subdirectory
            let studio_dirs: Vec<PathBuf> = Self::find_dirs(version_dir, "android-studio");
            if studio_dirs.is_empty() {
                return Err("No android-studio directory found".into());
            }
            studio_dir = studio_dirs[0].clone();
        }
        
        // Create Applications directory for user
        let user_apps_dir = dirs::home_dir()
            .unwrap()
            .join("Applications");
        
        fs::create_dir_all(&user_apps_dir)?;
        
        let versioned_dest_path = user_apps_dir
            .join(format!("android-studio-{}", version));
        let symlink_path = user_apps_dir.join("android-studio");
        
        println!("{} Installing Android Studio {} to ~/Applications...", "🚀".blue(), version);
        
        // Remove existing versioned installation if it exists
        if versioned_dest_path.exists() {
            println!("{} Removing existing android-studio-{}...", "🗑".yellow(), version);
            fs::remove_dir_all(&versioned_dest_path)?;
        }
        
        // Use cp -r to copy the directory
        let status = Command::new("cp")
            .args([
                "-R",
                studio_dir.to_str().unwrap(),
                versioned_dest_path.to_str().unwrap()
            ])
            .status()?;
        
        if !status.success() {
            return Err("Failed to install Android Studio".into());
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
        
        #[cfg(unix)]
        std::os::unix::fs::symlink(&versioned_dest_path, &symlink_path)?;
        
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&versioned_dest_path, &symlink_path)?;
        
        println!("{} Android Studio {} installed to ~/Applications!", "✅".green(), version);
        println!("  Versioned: {}", versioned_dest_path.display());
        println!("  Symlink: {}", symlink_path.display());
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn install_windows(
        &self,
        version: &str,
        version_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut studio_dir = version_dir.join("android-studio");
        
        if !studio_dir.exists() {
            // Check if it's in a subdirectory
            let studio_dirs: Vec<PathBuf> = Self::find_dirs(version_dir, "android-studio");
            if studio_dirs.is_empty() {
                return Err("No android-studio directory found".into());
            }
            studio_dir = studio_dirs[0].clone();
        }
        
        let user_apps_dir = dirs::home_dir()
            .unwrap()
            .join("Applications");
        
        fs::create_dir_all(&user_apps_dir)?;
        
        let versioned_dest_path = user_apps_dir
            .join(format!("Android Studio-{}", version));
        let symlink_path = user_apps_dir.join("Android Studio");
        
        println!("{} Installing Android Studio {} to ~/Applications...", "🚀".blue(), version);
        
        // Remove existing versioned installation if it exists
        if versioned_dest_path.exists() {
            println!("{} Removing existing Android Studio-{}...", "🗑".yellow(), version);
            fs::remove_dir_all(&versioned_dest_path)?;
        }
        
        // Use robocopy or xcopy on Windows
        let status = Command::new("robocopy")
            .args([
                studio_dir.to_str().unwrap(),
                versioned_dest_path.to_str().unwrap(),
                "/E",
                "/COPY:DAT"
            ])
            .status()?;
        
        if !status.success() {
            return Err("Failed to install Android Studio".into());
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
        
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&versioned_dest_path, &symlink_path)?;
        
        println!("{} Android Studio {} installed to ~/Applications!", "✅".green(), version);
        println!("  Versioned: {}", versioned_dest_path.display());
        println!("  Symlink: {}", symlink_path.display());
        
        Ok(())

    }

    pub fn switch_to_version(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        let versioned_path = self.applications_dir.join(format!("Android Studio-{}.app", version));
        
        #[cfg(target_os = "linux")]
        let versioned_path = dirs::home_dir()
            .unwrap()
            .join("Applications")
            .join(format!("android-studio-{}", version));
        
        #[cfg(target_os = "windows")]
        let versioned_path = dirs::home_dir()
            .unwrap()
            .join("Applications")
            .join(format!("Android Studio-{}", version));
        
        #[cfg(target_os = "macos")]
        let symlink_path = self.applications_dir.join("Android Studio.app");
        
        #[cfg(target_os = "linux")]
        let symlink_path = dirs::home_dir()
            .unwrap()
            .join("Applications")
            .join("android-studio");
        
        #[cfg(target_os = "windows")]
        let symlink_path = dirs::home_dir()
            .unwrap()
            .join("Applications")
            .join("Android Studio");
        
        if !versioned_path.exists() {
            return Err(format!("Android Studio {} is not installed", version).into());
        }
        
        println!("{} Switching to Android Studio {}...", "🔀".blue(), version);
        
        // Remove existing symlink
        if symlink_path.exists() {
            if symlink_path.is_symlink() {
                fs::remove_file(&symlink_path)?;
            } else {
                fs::remove_dir_all(&symlink_path)?;
            }
        }
        
        #[cfg(unix)]
        std::os::unix::fs::symlink(&versioned_path, &symlink_path)?;
        
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&versioned_path, &symlink_path)?;
        
        println!("{} Now using Android Studio {}", "✅".green(), version);
        println!("  Symlink: {}", symlink_path.display());
        println!("  Points to: {}", versioned_path.display());
        
        Ok(())
    }

    pub fn list_installed_versions(&self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{} Installed Android Studio versions:", "📋".green().bold());
        println!();
        
        let mut found = false;
        
        #[cfg(target_os = "macos")]
        let apps_dir = &self.applications_dir;
        
        #[cfg(not(target_os = "macos"))]
        let apps_dir = &dirs::home_dir().unwrap().join("Applications");
        
        if let Ok(entries) = fs::read_dir(apps_dir) {
            let mut versions = Vec::new();
            
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                #[cfg(target_os = "macos")]
                {
                    if name_str.starts_with("Android Studio-") && name_str.ends_with(".app") {
                        if let Some(version) = name_str.strip_prefix("Android Studio-").and_then(|s| s.strip_suffix(".app")) {
                            versions.push(version.to_string());
                        }
                    }
                }
                
                #[cfg(not(target_os = "macos"))]
                {
                    if cfg!(target_os = "linux") {
                        if name_str.starts_with("android-studio-") {
                            if let Some(version) = name_str.strip_prefix("android-studio-") {
                                versions.push(version.to_string());
                            }
                        }
                    } else if cfg!(target_os = "windows") {
                        if name_str.starts_with("Android Studio-") {
                            if let Some(version) = name_str.strip_prefix("Android Studio-") {
                                versions.push(version.to_string());
                            }
                        }
                    }
                }
            }
            
            if !versions.is_empty() {
                found = true;
                versions.sort();
                
                for version in versions {
                    let is_active = self.is_version_active(&version)?;
                    let indicator = if is_active { "✅" } else { "  " };
                    println!("{} Android Studio-{}" , indicator, version);
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

    pub fn show_current_version(&self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        let symlink_path = self.applications_dir.join("Android Studio.app");
        
        #[cfg(target_os = "linux")]
        let symlink_path = dirs::home_dir()
            .unwrap()
            .join("Applications")
            .join("android-studio");
        
        #[cfg(target_os = "windows")]
        let symlink_path = dirs::home_dir()
            .unwrap()
            .join("Applications")
            .join("Android Studio");
        
        if symlink_path.exists() {
            if symlink_path.is_symlink() {
                if let Ok(target) = fs::read_link(&symlink_path) {
                    let target_name = target.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    
                    #[cfg(target_os = "macos")]
                    let version = target_name.strip_prefix("Android Studio-").and_then(|s| s.strip_suffix(".app"));
                    
                    #[cfg(target_os = "linux")]
                    let version = target_name.strip_prefix("android-studio-");
                    
                    #[cfg(target_os = "windows")]
                    let version = target_name.strip_prefix("Android Studio-");
                    
                    if let Some(version) = version {
                        println!("{} Currently using Android Studio {}", "✅".green(), version);
                        println!("  Symlink: {}", symlink_path.display());
                        println!("  Points to: {}", target.display());
                    } else {
                        println!("{} Symlink points to: {}", "ℹ".blue(), target.display());
                    }
                } else {
                    println!("{} Could not read symlink target", "⚠".yellow());
                }
            } else {
                println!("{} Android Studio is installed as a regular directory (not symlink)", "ℹ".blue());
            }
        } else {
            println!("{} Android Studio is not installed or symlink is missing", "⚠".yellow());
            println!();
            println!("Use 'as-man install <version>' to install a version");
        }
        
        Ok(())
    }

    fn is_version_active(&self, version: &str) -> Result<bool, Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        let symlink_path = self.applications_dir.join("Android Studio.app");
        
        #[cfg(target_os = "linux")]
        let symlink_path = dirs::home_dir()
            .unwrap()
            .join("Applications")
            .join("android-studio");
        
        #[cfg(target_os = "windows")]
        let symlink_path = dirs::home_dir()
            .unwrap()
            .join("Applications")
            .join("Android Studio");
        
        if symlink_path.exists() && symlink_path.is_symlink() {
            if let Ok(target) = fs::read_link(&symlink_path) {
                let target_name = target.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                #[cfg(target_os = "macos")]
                let expected_suffix = format!("Android Studio-{}.app", version);
                
                #[cfg(target_os = "linux")]
                let expected_suffix = format!("android-studio-{}", version);
                
                #[cfg(target_os = "windows")]
                let expected_suffix = format!("Android Studio-{}", version);
                
                return Ok(target_name == expected_suffix);
            }
        }
        
        Ok(false)
    }

    fn find_files(dir: &Path, extension: &str) -> Vec<PathBuf> {
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
                    files.extend(Self::find_files(&path, extension));
                }
            }
        }
        files
    }

    fn find_dirs(dir: &Path, name: &str) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.contains(name))
                        .unwrap_or(false)
                    {
                        dirs.push(path);
                    } else {
                        dirs.extend(Self::find_dirs(&path, name));
                    }
                }
            }
        }
        dirs
    }
}