use std::fmt;

/// Custom error types for as-man application
#[derive(Debug)]
pub enum AsManError {
    /// IO-related errors
    Io(std::io::Error),
    /// Network-related errors
    Network(reqwest::Error),
    /// XML/JSON parsing errors
    Parse(String),
    /// Configuration errors
    Config(String),
    /// Version not found errors
    VersionNotFound(String),
    /// Platform-specific errors
    Platform(String),
    /// Download errors
    Download(String),
    /// Installation errors
    Installation(String),
    /// Archive extraction errors
    Extraction(String),
    /// Downloader not found errors
    DownloaderNotFound(String),
    /// Path-related errors
    Path(String),
    /// Cache-related errors
    Cache(String),
    /// General application errors
    General(String),
    /// UTF-8 conversion errors
    Utf8(std::str::Utf8Error),
    /// ZIP archive errors
    Zip(zip::result::ZipError),
    /// System time errors
    SystemTime(std::time::SystemTimeError),
    /// Prerequisite not met errors
    PrerequisiteNotMet(String),
    /// Insufficient resources errors
    InsufficientResources(String),
    /// Permission denied errors
    PermissionDenied(String),
    /// Network unavailable errors
    NetworkUnavailable(String),
}

impl fmt::Display for AsManError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsManError::Io(e) => write!(f, "IO error: {e}"),
            AsManError::Network(e) => write!(f, "Network error: {e}"),
            AsManError::Parse(msg) => write!(f, "Parse error: {msg}"),
            AsManError::Config(msg) => write!(f, "Configuration error: {msg}"),
            AsManError::VersionNotFound(version) => write!(f, "Version '{version}' not found"),
            AsManError::Platform(msg) => write!(f, "Platform error: {msg}"),
            AsManError::Download(msg) => write!(f, "Download error: {msg}"),
            AsManError::Installation(msg) => write!(f, "Installation error: {msg}"),
            AsManError::Extraction(msg) => write!(f, "Extraction error: {msg}"),
            AsManError::DownloaderNotFound(msg) => write!(f, "Downloader not found: {msg}"),
            AsManError::Path(msg) => write!(f, "Path error: {msg}"),
            AsManError::Cache(msg) => write!(f, "Cache error: {msg}"),
            AsManError::General(msg) => write!(f, "Error: {msg}"),
            AsManError::Utf8(e) => write!(f, "UTF-8 error: {e}"),
            AsManError::Zip(e) => write!(f, "ZIP error: {e}"),
            AsManError::SystemTime(e) => write!(f, "System time error: {e}"),
            AsManError::PrerequisiteNotMet(msg) => write!(f, "Prerequisite not met: {msg}"),
            AsManError::InsufficientResources(msg) => write!(f, "Insufficient resources: {msg}"),
            AsManError::PermissionDenied(msg) => write!(f, "Permission denied: {msg}"),
            AsManError::NetworkUnavailable(msg) => write!(f, "Network unavailable: {msg}"),
        }
    }
}

impl std::error::Error for AsManError {}

impl From<std::io::Error> for AsManError {
    fn from(err: std::io::Error) -> Self {
        AsManError::Io(err)
    }
}

impl From<reqwest::Error> for AsManError {
    fn from(err: reqwest::Error) -> Self {
        AsManError::Network(err)
    }
}

impl From<serde_json::Error> for AsManError {
    fn from(err: serde_json::Error) -> Self {
        AsManError::Parse(err.to_string())
    }
}

impl From<quick_xml::DeError> for AsManError {
    fn from(err: quick_xml::DeError) -> Self {
        AsManError::Parse(err.to_string())
    }
}

impl From<std::str::Utf8Error> for AsManError {
    fn from(err: std::str::Utf8Error) -> Self {
        AsManError::Utf8(err)
    }
}

impl From<zip::result::ZipError> for AsManError {
    fn from(err: zip::result::ZipError) -> Self {
        AsManError::Zip(err)
    }
}

impl From<std::time::SystemTimeError> for AsManError {
    fn from(err: std::time::SystemTimeError) -> Self {
        AsManError::SystemTime(err)
    }
}
