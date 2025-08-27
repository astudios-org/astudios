use serde::{Deserialize, Serialize};

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

    /// Get Windows download URL
    pub fn get_windows_download(&self) -> Option<&Download> {
        self.downloads.iter().find(|d| d.link.contains("windows"))
    }

    /// Get Linux download URL
    pub fn get_linux_download(&self) -> Option<&Download> {
        self.downloads.iter().find(|d| d.link.contains("linux"))
    }

    /// Get download URL for current platform
    pub fn get_platform_download(&self) -> Option<&Download> {
        if cfg!(target_os = "macos") {
            self.get_macos_download()
        } else if cfg!(target_os = "windows") {
            self.get_windows_download()
        } else if cfg!(target_os = "linux") {
            self.get_linux_download()
        } else {
            None
        }
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
