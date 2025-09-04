use std::path::PathBuf;

/// Application configuration constants and utilities
pub struct Config;

impl Config {
    /// Application name used for directory creation
    pub const APP_NAME: &'static str = "astudios";

    /// Default cache expiration time (24 hours)
    pub const CACHE_DURATION_SECS: u64 = 60 * 60 * 24;

    /// Default network timeout for API requests (30 seconds)
    pub const NETWORK_TIMEOUT_SECS: u64 = 30;

    /// Default download timeout (5 minutes)
    pub const DOWNLOAD_TIMEOUT_SECS: u64 = 300;

    /// Maximum download retry attempts
    pub const MAX_DOWNLOAD_RETRIES: u32 = 3;

    /// Retry wait time between download attempts (5 seconds)
    pub const DOWNLOAD_RETRY_WAIT_SECS: u64 = 5;

    /// Number of parallel connections for aria2
    pub const ARIA2_MAX_CONNECTIONS: u32 = 16;

    /// Minimum split size for aria2 downloads
    pub const ARIA2_MIN_SPLIT_SIZE: &'static str = "1M";

    /// Minimum disk space required for Android Studio installation (in GB)
    pub const MIN_DISK_SPACE_GB: u64 = 8;

    /// Minimum RAM recommended for Android Studio (in GB)
    pub const MIN_RAM_GB: u64 = 8;

    /// Timeout for system detection checks (in seconds)
    pub const DETECTION_TIMEOUT_SECS: u64 = 10;

    /// Default download directory (now points to versions directory)
    pub fn default_download_dir() -> PathBuf {
        Self::versions_dir()
    }

    /// Application cache directory
    pub fn cache_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".")
            .join(Self::APP_NAME)
            .join("cache")
    }

    /// Application versions directory
    pub fn versions_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".")
            .join(Self::APP_NAME)
            .join("versions")
    }

    /// Default applications directory (macOS)
    pub fn default_applications_dir() -> PathBuf {
        PathBuf::from("/Applications")
    }

    /// JetBrains API endpoint for Android Studio releases
    pub const RELEASES_FEED_URL: &'static str = "https://teamcity.jetbrains.com/guestAuth/repository/download/AndroidStudioReleasesList/.lastSuccessful/android-studio-releases-list.xml";

    /// User agent string for HTTP requests
    pub fn user_agent() -> String {
        format!("{}/0.1.0", Self::APP_NAME)
    }

    /// Get minimum disk space requirement in GB
    pub fn min_disk_space_gb() -> u64 {
        Self::MIN_DISK_SPACE_GB
    }

    /// Get minimum RAM requirement in GB
    pub fn min_ram_gb() -> u64 {
        Self::MIN_RAM_GB
    }

    /// Common aria2 executable paths for macOS
    pub fn aria2_search_paths() -> &'static [&'static str] {
        &[
            "/usr/local/bin/aria2c",
            "/opt/homebrew/bin/aria2c",
            "/usr/bin/aria2c",
            "/bin/aria2c",
            "aria2c",
        ]
    }
}
