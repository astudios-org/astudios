use crate::{config::Config, error::AstudiosError};
use std::{fs, path::Path, process::Command};

/// System detection and validation for pre-installation checks
pub struct SystemDetector;

/// Results of system detection checks
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub platform_compatible: bool,
    pub disk_space_sufficient: bool,
    pub permissions_valid: bool,
    pub network_available: bool,
    pub dependencies_available: bool,
    pub java_runtime_available: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for DetectionResult {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectionResult {
    pub fn new() -> Self {
        Self {
            platform_compatible: false,
            disk_space_sufficient: false,
            permissions_valid: false,
            network_available: false,
            dependencies_available: false,
            java_runtime_available: false,
            issues: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.platform_compatible
            && self.disk_space_sufficient
            && self.permissions_valid
            && self.network_available
            && self.dependencies_available
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn add_issue(&mut self, issue: String) {
        self.issues.push(issue);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

impl SystemDetector {
    /// Run comprehensive system detection checks
    pub fn detect_system_requirements(
        install_dir: &Path,
        applications_dir: &Path,
    ) -> Result<DetectionResult, AstudiosError> {
        let mut result = DetectionResult::new();

        // Check platform compatibility
        result.platform_compatible = Self::check_platform_compatibility(&mut result)?;

        // Check disk space
        result.disk_space_sufficient =
            Self::check_disk_space(install_dir, applications_dir, &mut result)?;

        // Check permissions
        result.permissions_valid =
            Self::check_permissions(install_dir, applications_dir, &mut result)?;

        // Check network connectivity
        result.network_available = Self::check_network_connectivity(&mut result)?;

        // Check dependencies
        result.dependencies_available = Self::check_dependencies(&mut result)?;

        // Check Java runtime (optional - add warning if not found)
        result.java_runtime_available = Self::check_java_runtime(&mut result)?;

        Ok(result)
    }

    /// Check if the current platform is supported (macOS only)
    fn check_platform_compatibility(result: &mut DetectionResult) -> Result<bool, AstudiosError> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        if os != "macos" {
            result.add_issue(format!(
                "Unsupported operating system: {os}. This tool only supports macOS."
            ));
            return Ok(false);
        }

        if arch != "x86_64" && arch != "aarch64" {
            result.add_issue(format!(
                "Unsupported architecture: {arch}. Android Studio requires x86_64 or aarch64 on macOS."
            ));
            return Ok(false);
        }

        // Check macOS version
        if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
            let version = String::from_utf8_lossy(&output.stdout);
            let version = version.trim();

            // Parse version and check if it's at least macOS 10.14
            if let Some(major_minor) = version.split('.').take(2).collect::<Vec<_>>().get(0..2)
                && let (Ok(major), Ok(minor)) =
                    (major_minor[0].parse::<u32>(), major_minor[1].parse::<u32>())
                && (major < 10 || (major == 10 && minor < 14))
            {
                result.add_issue(format!(
                            "macOS version {version} is not supported. Android Studio requires macOS 10.14 or later."
                        ));
                return Ok(false);
            }
        } else {
            result.add_warning("Could not determine macOS version".to_string());
        }

        Ok(true)
    }

    /// Check available disk space
    fn check_disk_space(
        install_dir: &Path,
        applications_dir: &Path,
        result: &mut DetectionResult,
    ) -> Result<bool, AstudiosError> {
        let required_space = Config::min_disk_space_gb()
            .checked_mul(1024)
            .and_then(|x| x.checked_mul(1024))
            .and_then(|x| x.checked_mul(1024))
            .unwrap_or(8 * 1024 * 1024 * 1024); // Default to 8GB if overflow

        let mut space_check_failed = false;
        let mut insufficient_space = false;

        // Check space in install directory
        match Self::get_available_space(install_dir) {
            Ok(space) => {
                if space < required_space {
                    result.add_issue(format!(
                        "Insufficient disk space in {}. Required: {} GB, Available: {:.1} GB",
                        install_dir.display(),
                        Config::min_disk_space_gb(),
                        space as f64 / (1024.0 * 1024.0 * 1024.0)
                    ));
                    insufficient_space = true;
                }
            }
            Err(_) => {
                space_check_failed = true;
            }
        }

        // Check space in applications directory (only if different from install directory)
        if install_dir != applications_dir {
            match Self::get_available_space(applications_dir) {
                Ok(space) => {
                    if space < required_space {
                        result.add_issue(format!(
                            "Insufficient disk space in {}. Required: {} GB, Available: {:.1} GB",
                            applications_dir.display(),
                            Config::min_disk_space_gb(),
                            space as f64 / (1024.0 * 1024.0 * 1024.0)
                        ));
                        insufficient_space = true;
                    }
                }
                Err(_) => {
                    space_check_failed = true;
                }
            }
        }

        // Only add a warning if we couldn't check space at all and there's no other issue
        if space_check_failed && !insufficient_space {
            result.add_warning(
                "Could not verify available disk space. Ensure you have sufficient space for installation.".to_string()
            );
        }

        Ok(!insufficient_space)
    }

    /// Get available disk space for a given path (macOS/Unix)
    fn get_available_space(path: &Path) -> Result<u64, AstudiosError> {
        // Create the directory if it doesn't exist to check space
        if !path.exists() {
            fs::create_dir_all(path)?;
        }

        // Try multiple methods to get disk space, starting with the most reliable

        // Method 1: Use df command (most reliable on macOS)
        if let Ok(space) = Self::get_space_via_df(path) {
            return Ok(space);
        }

        // Method 2: Use statvfs system call
        if let Ok(space) = Self::get_space_via_statvfs(path) {
            return Ok(space);
        }

        // Method 3: Try with parent directory if the path itself fails
        if let Some(parent) = path.parent() {
            if let Ok(space) = Self::get_space_via_df(parent) {
                return Ok(space);
            }
            if let Ok(space) = Self::get_space_via_statvfs(parent) {
                return Ok(space);
            }
        }

        Err(AstudiosError::General(
            "Could not determine available disk space using any method".to_string(),
        ))
    }

    /// Get disk space using df command (most reliable on macOS)
    fn get_space_via_df(path: &Path) -> Result<u64, AstudiosError> {
        let output = Command::new("df")
            .args(["-k", path.to_str().unwrap_or(".")])
            .output()
            .map_err(|_| AstudiosError::General("df command failed".to_string()))?;

        if !output.status.success() {
            return Err(AstudiosError::General(
                "df command returned error".to_string(),
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        // df output format: Filesystem 1K-blocks Used Available Capacity Mounted on
        // We want the "Available" column (index 3) from the second line
        if lines.len() >= 2 {
            let fields: Vec<&str> = lines[1].split_whitespace().collect();
            if fields.len() >= 4
                && let Ok(available_kb) = fields[3].parse::<u64>() {
                    // Convert from KB to bytes
                    return Ok(available_kb * 1024);
                }
        }

        Err(AstudiosError::General(
            "Could not parse df output".to_string(),
        ))
    }

    /// Get disk space using statvfs system call (fallback method)
    fn get_space_via_statvfs(path: &Path) -> Result<u64, AstudiosError> {
        // Canonicalize the path to resolve any relative components like "./"
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // If canonicalization fails, try to use the path as-is
                // but ensure it's absolute
                if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    std::env::current_dir()?.join(path)
                }
            }
        };

        use std::ffi::CString;
        use std::mem;
        use std::os::raw::{c_char, c_int};

        #[repr(C)]
        struct Statvfs {
            f_bsize: u64,
            f_frsize: u64,
            f_blocks: u64,
            f_bfree: u64,
            f_bavail: u64,
            f_files: u64,
            f_ffree: u64,
            f_favail: u64,
            f_fsid: u64,
            f_flag: u64,
            f_namemax: u64,
        }

        unsafe extern "C" {
            fn statvfs(path: *const c_char, buf: *mut Statvfs) -> c_int;
        }

        // Convert path to string and handle potential null bytes
        let path_str = canonical_path.to_string_lossy();
        let path_bytes: Vec<u8> = path_str.bytes().filter(|&b| b != 0).collect();
        let path_cstring = CString::new(path_bytes)
            .map_err(|_| AstudiosError::General("Invalid path for disk space check".to_string()))?;

        let mut stat: Statvfs = unsafe { mem::zeroed() };

        let result = unsafe { statvfs(path_cstring.as_ptr(), &mut stat) };

        if result == 0 {
            stat.f_bavail.checked_mul(stat.f_frsize).ok_or_else(|| {
                AstudiosError::General("Disk space calculation overflow".to_string())
            })
        } else {
            Err(AstudiosError::General(
                "statvfs system call failed".to_string(),
            ))
        }
    }

    /// Check directory permissions
    fn check_permissions(
        install_dir: &Path,
        applications_dir: &Path,
        result: &mut DetectionResult,
    ) -> Result<bool, AstudiosError> {
        let mut permissions_ok = true;

        // Check install directory permissions
        if !Self::check_directory_permissions(install_dir)? {
            result.add_issue(format!(
                "No write permission for install directory: {}",
                install_dir.display()
            ));
            permissions_ok = false;
        }

        // Check applications directory permissions
        if !Self::check_directory_permissions(applications_dir)? {
            result.add_issue(format!(
                "No write permission for applications directory: {}. You may need administrator privileges.",
                applications_dir.display()
            ));
            permissions_ok = false;
        }

        Ok(permissions_ok)
    }

    /// Check if we have write permissions for a directory
    fn check_directory_permissions(dir: &Path) -> Result<bool, AstudiosError> {
        // Create directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        // Try to create a temporary file to test write permissions
        let test_file = dir.join(".astudios-permission-test");
        match fs::write(&test_file, "test") {
            Ok(_) => {
                // Clean up test file
                let _ = fs::remove_file(&test_file);
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Check network connectivity
    fn check_network_connectivity(result: &mut DetectionResult) -> Result<bool, AstudiosError> {
        // Try to make a simple HEAD request to the JetBrains API
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(Config::NETWORK_TIMEOUT_SECS))
            .user_agent(Config::user_agent())
            .build()?;

        match client.head(Config::RELEASES_FEED_URL).send() {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(true)
                } else {
                    result.add_issue(format!(
                        "JetBrains API is not accessible (HTTP {}). Check your internet connection.",
                        response.status()
                    ));
                    Ok(false)
                }
            }
            Err(e) => {
                result.add_issue(format!(
                    "Network connectivity check failed: {e}. Check your internet connection and firewall settings."
                ));
                Ok(false)
            }
        }
    }

    /// Check required dependencies
    fn check_dependencies(result: &mut DetectionResult) -> Result<bool, AstudiosError> {
        let mut dependencies_ok = true;

        // Check for required system tools
        let required_tools = Self::get_required_tools();
        for tool in required_tools {
            if !Self::check_tool_available(tool) {
                result.add_issue(format!(
                    "Required tool '{tool}' not found in PATH. Please install it and try again."
                ));
                dependencies_ok = false;
            }
        }

        // Check for archive extraction tools
        if !Self::check_archive_tools(result)? {
            dependencies_ok = false;
        }

        // Check for download tools (aria2 is optional, but warn if not available)
        if crate::downloader::Downloader::find_aria2().is_err() {
            result.add_warning(
                "aria2 not found. Downloads will use the built-in HTTP client, which may be slower. Install aria2 for faster downloads.".to_string()
            );
        }

        Ok(dependencies_ok)
    }

    /// Get list of required system tools for macOS
    fn get_required_tools() -> Vec<&'static str> {
        vec!["ditto", "rm", "hdiutil", "codesign"]
    }

    /// Check if a tool is available in PATH
    fn check_tool_available(tool: &str) -> bool {
        Command::new("which")
            .arg(tool)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check for archive extraction tools (macOS)
    fn check_archive_tools(result: &mut DetectionResult) -> Result<bool, AstudiosError> {
        // On macOS, we need hdiutil for DMG files
        if !Self::check_tool_available("hdiutil") {
            result.add_issue(
                "hdiutil not found. This tool is required for extracting DMG files on macOS."
                    .to_string(),
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Check for Java runtime (optional but recommended)
    fn check_java_runtime(result: &mut DetectionResult) -> Result<bool, AstudiosError> {
        // Check for Java in common locations
        let java_commands = ["java", "javac"];
        let mut java_found = false;

        for cmd in java_commands {
            if let Ok(output) = Command::new(cmd).arg("-version").output()
                && output.status.success()
            {
                java_found = true;
                break;
            }
        }

        if !java_found {
            result.add_warning(
                "Java runtime not found. While Android Studio includes its own JDK, having Java installed system-wide is recommended for development.".to_string()
            );
        }

        Ok(java_found)
    }
}

impl From<std::ffi::NulError> for AstudiosError {
    fn from(err: std::ffi::NulError) -> Self {
        AstudiosError::General(format!("String conversion error: {err}"))
    }
}
