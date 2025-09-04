use crate::error::AstudiosError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Root structure for Android Studio releases list
#[derive(Debug, Deserialize, Serialize)]
pub struct AndroidStudioReleasesList {
    /// XML version attribute
    #[serde(rename = "@version")]
    pub version: String,
    /// List of Android Studio releases
    #[serde(rename = "item")]
    pub items: Vec<AndroidStudio>,
}

/// Android Studio release information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AndroidStudio {
    /// Human-readable name (e.g., "Android Studio Hedgehog")
    pub name: String,
    /// Build number (e.g., "AI-231.9392.1")
    pub build: String,
    /// Version string (e.g., "2023.3.1")
    pub version: String,
    /// Release channel (Release, Beta, Canary, RC, Patch)
    pub channel: String,
    /// Platform-specific build identifier
    #[serde(rename = "platformBuild")]
    pub platform_build: String,
    /// Platform version identifier
    #[serde(rename = "platformVersion")]
    pub platform_version: String,
    /// Release date in YYYY-MM-DD format
    pub date: String,
    /// Available downloads for different platforms
    #[serde(rename = "download")]
    pub downloads: Vec<Download>,
}

/// Download information for a specific platform
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Download {
    /// Download URL
    pub link: String,
    /// Human-readable file size
    pub size: String,
    /// File checksum for integrity verification
    pub checksum: String,
}

/// Release channel types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReleaseChannel {
    Release,
    Beta,
    Canary,
    ReleaseCandidate,
    Patch,
}

impl AndroidStudio {
    /// Check if this is a stable release
    pub fn is_release(&self) -> bool {
        self.channel == "Release"
    }

    /// Check if this is a beta release
    pub fn is_beta(&self) -> bool {
        self.channel == "Beta"
    }

    /// Check if this is a canary release
    pub fn is_canary(&self) -> bool {
        self.channel == "Canary"
    }

    /// Check if this is a release candidate
    pub fn is_rc(&self) -> bool {
        self.channel == "RC"
    }

    /// Check if this is a patch release
    pub fn is_patch(&self) -> bool {
        self.channel == "Patch"
    }

    /// Get the release channel as an enum
    pub fn channel_type(&self) -> ReleaseChannel {
        match self.channel.as_str() {
            "Release" => ReleaseChannel::Release,
            "Beta" => ReleaseChannel::Beta,
            "Canary" => ReleaseChannel::Canary,
            "RC" => ReleaseChannel::ReleaseCandidate,
            "Patch" => ReleaseChannel::Patch,
            _ => ReleaseChannel::Release, // Default fallback
        }
    }

    /// Get macOS download URL
    pub fn get_macos_download(&self) -> Option<&Download> {
        self.downloads.iter().find(|d| d.link.contains("mac"))
    }

    /// Get download URL for macOS
    pub fn get_platform_download(&self) -> Option<&Download> {
        self.get_macos_download()
    }

    /// Get display name with channel indicator
    pub fn display_name(&self) -> String {
        let channel_indicator = match self.channel_type() {
            ReleaseChannel::Release => "",
            ReleaseChannel::Beta => " (Beta)",
            ReleaseChannel::Canary => " (Canary)",
            ReleaseChannel::ReleaseCandidate => " (RC)",
            ReleaseChannel::Patch => " (Patch)",
        };
        format!("{}{}", self.name, channel_indicator)
    }
}

/// Version information extracted from Android Studio metadata
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AndroidStudioVersion {
    /// Short version string from CFBundleShortVersionString (e.g., "2025.1")
    pub short_version: String,
    /// Full build version from CFBundleVersion (e.g., "AI-251.26094.121.2513.14007798")
    pub build_version: String,
    /// Product code (e.g., "AI")
    pub product_code: String,
    /// Build number without product code (e.g., "251.26094.121.2513.14007798")
    pub build_number: String,
    /// Human-readable name from product-info.json (e.g., "Android Studio")
    pub product_name: String,
}

impl AndroidStudioVersion {
    /// Create a new version from components
    pub fn new(
        short_version: String,
        build_version: String,
        product_code: String,
        build_number: String,
        product_name: String,
    ) -> Self {
        Self {
            short_version,
            build_version,
            product_code,
            build_number,
            product_name,
        }
    }

    /// Get a display-friendly version string
    pub fn display_version(&self) -> String {
        format!("{} ({})", self.short_version, self.build_version)
    }

    /// Get a unique identifier for this version
    pub fn identifier(&self) -> String {
        self.build_version.clone()
    }

    /// Check if this is a stable release (not beta, canary, etc.)
    pub fn is_stable(&self) -> bool {
        !self.build_version.contains("Beta")
            && !self.build_version.contains("Canary")
            && !self.build_version.contains("RC")
    }
}

impl std::fmt::Display for AndroidStudioVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_version())
    }
}

/// Represents an installed Android Studio instance
/// Similar to InstalledXcode in the Xcodes project
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledAndroidStudio {
    /// Path to the Android Studio.app bundle
    pub path: PathBuf,
    /// Parsed version information from metadata
    pub version: AndroidStudioVersion,
}

impl InstalledAndroidStudio {
    /// Create a new InstalledAndroidStudio from a path
    /// Returns None if the path doesn't contain a valid Android Studio installation
    pub fn new(path: PathBuf) -> Result<Option<Self>, AstudiosError> {
        if !path.exists() {
            return Ok(None);
        }

        // Check if it's an app bundle
        if path.extension().is_none_or(|ext| ext != "app") {
            return Ok(None);
        }

        // Try to parse version information
        match Self::parse_version_info(&path) {
            Ok(version) => Ok(Some(Self { path, version })),
            Err(_) => Ok(None), // Not a valid Android Studio installation
        }
    }

    /// Parse version information from Android Studio metadata files
    fn parse_version_info(app_path: &Path) -> Result<AndroidStudioVersion, AstudiosError> {
        let contents_path = app_path.join("Contents");

        // Parse Info.plist
        let info_plist_path = contents_path.join("Info.plist");
        let (short_version, build_version) = Self::parse_info_plist(&info_plist_path)?;

        // Parse product-info.json for additional details
        let product_info_path = contents_path.join("Resources").join("product-info.json");
        let (product_name, product_code, build_number) =
            Self::parse_product_info(&product_info_path)?;

        Ok(AndroidStudioVersion::new(
            short_version,
            build_version,
            product_code,
            build_number,
            product_name,
        ))
    }

    /// Parse Info.plist file for version information
    fn parse_info_plist(plist_path: &Path) -> Result<(String, String), AstudiosError> {
        use plist::Value;
        use std::fs::File;

        let file = File::open(plist_path)
            .map_err(|e| AstudiosError::General(format!("Failed to open Info.plist: {e}")))?;

        let plist: Value = plist::from_reader(file)
            .map_err(|e| AstudiosError::General(format!("Failed to parse Info.plist: {e}")))?;

        let dict = plist
            .as_dictionary()
            .ok_or_else(|| AstudiosError::General("Info.plist is not a dictionary".to_string()))?;

        // Extract CFBundleShortVersionString
        let short_version = dict
            .get("CFBundleShortVersionString")
            .and_then(|v| v.as_string())
            .ok_or_else(|| AstudiosError::General("CFBundleShortVersionString not found".to_string()))?
            .to_string();

        // Extract CFBundleVersion
        let build_version = dict
            .get("CFBundleVersion")
            .and_then(|v| v.as_string())
            .ok_or_else(|| AstudiosError::General("CFBundleVersion not found".to_string()))?
            .to_string();

        // Verify this is Android Studio
        let bundle_id = dict
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_string())
            .unwrap_or("");

        if !bundle_id.contains("android.studio") {
            return Err(AstudiosError::General(
                "Not an Android Studio application".to_string(),
            ));
        }

        Ok((short_version, build_version))
    }

    /// Parse product-info.json file for additional version information
    fn parse_product_info(
        product_info_path: &Path,
    ) -> Result<(String, String, String), AstudiosError> {
        use std::fs;

        let content = fs::read_to_string(product_info_path)
            .map_err(|e| AstudiosError::General(format!("Failed to read product-info.json: {e}")))?;

        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AstudiosError::General(format!("Failed to parse product-info.json: {e}")))?;

        let product_name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Android Studio")
            .to_string();

        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AstudiosError::General("Version not found in product-info.json".to_string())
            })?;

        let build_number = json
            .get("buildNumber")
            .and_then(|v| v.as_str())
            .unwrap_or(version);

        // Extract product code from version (e.g., "AI" from "AI-251.26094.121.2513.14007798")
        let product_code = version.split('-').next().unwrap_or("AI").to_string();

        Ok((product_name, product_code, build_number.to_string()))
    }

    /// Get the display name for this installation
    pub fn display_name(&self) -> String {
        format!(
            "{} {}",
            self.version.product_name, self.version.short_version
        )
    }

    /// Extract detailed version information, preferring API version over short version
    pub fn extract_detailed_version(&self) -> String {
        // Try to get full version from API first
        if let Ok(Some(api_version)) = self.get_full_version_from_api() {
            api_version
        } else {
            // Fallback to short version
            self.version.short_version.clone()
        }
    }

    /// Get the full version number by matching with API data
    pub fn get_full_version_from_api(&self) -> Result<Option<String>, AstudiosError> {
        use crate::list::AndroidStudioLister;

        let lister = AndroidStudioLister::new()?;
        let releases = lister.get_releases()?;

        // Try to find matching release by build version
        for release in &releases.items {
            if release.build == self.version.build_version {
                return Ok(Some(release.version.clone()));
            }
        }

        Ok(None)
    }

    /// Get enhanced display name with detailed version information
    pub fn enhanced_display_name(&self) -> String {
        let detailed_version = self.extract_detailed_version();
        let channel_info = self.detect_channel_from_name();

        if channel_info.is_empty() {
            format!("{} {}", self.version.product_name, detailed_version)
        } else {
            format!(
                "{} {} ({})",
                self.version.product_name, detailed_version, channel_info
            )
        }
    }

    /// Detect release channel information from app name
    fn detect_channel_from_name(&self) -> String {
        let app_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        if app_name.contains("Patch") {
            "Patch".to_string()
        } else if app_name.contains("Feature Drop") {
            "Feature Drop".to_string()
        } else if app_name.contains("Beta") {
            "Beta".to_string()
        } else if app_name.contains("Canary") {
            "Canary".to_string()
        } else if app_name.contains("RC") {
            "RC".to_string()
        } else {
            "Release".to_string()
        }
    }

    /// Get the unique identifier for this installation
    pub fn identifier(&self) -> String {
        self.version.identifier()
    }

    /// Check if this installation is valid and accessible
    pub fn is_valid(&self) -> bool {
        self.path.exists() && self.path.join("Contents").exists()
    }
}

impl PartialOrd for InstalledAndroidStudio {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InstalledAndroidStudio {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}
