use std::fmt;

/// Custom error types for astudios application
#[derive(Debug)]
pub enum AstudiosError {
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

impl fmt::Display for AstudiosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstudiosError::Io(e) => write!(f, "IO error: {e}"),
            AstudiosError::Network(e) => write!(f, "Network error: {e}"),
            AstudiosError::Parse(msg) => write!(f, "Parse error: {msg}"),
            AstudiosError::Config(msg) => write!(f, "Configuration error: {msg}"),
            AstudiosError::VersionNotFound(version) => write!(f, "Version '{version}' not found"),
            AstudiosError::Platform(msg) => write!(f, "Platform error: {msg}"),
            AstudiosError::Download(msg) => write!(f, "Download error: {msg}"),
            AstudiosError::Installation(msg) => write!(f, "Installation error: {msg}"),
            AstudiosError::Extraction(msg) => write!(f, "Extraction error: {msg}"),
            AstudiosError::DownloaderNotFound(msg) => write!(f, "Downloader not found: {msg}"),
            AstudiosError::Path(msg) => write!(f, "Path error: {msg}"),
            AstudiosError::Cache(msg) => write!(f, "Cache error: {msg}"),
            AstudiosError::General(msg) => write!(f, "Error: {msg}"),
            AstudiosError::Utf8(e) => write!(f, "UTF-8 error: {e}"),
            AstudiosError::Zip(e) => write!(f, "ZIP error: {e}"),
            AstudiosError::SystemTime(e) => write!(f, "System time error: {e}"),
            AstudiosError::PrerequisiteNotMet(msg) => write!(f, "Prerequisite not met: {msg}"),
            AstudiosError::InsufficientResources(msg) => write!(f, "Insufficient resources: {msg}"),
            AstudiosError::PermissionDenied(msg) => write!(f, "Permission denied: {msg}"),
            AstudiosError::NetworkUnavailable(msg) => write!(f, "Network unavailable: {msg}"),
        }
    }
}

impl std::error::Error for AstudiosError {}

impl From<std::io::Error> for AstudiosError {
    fn from(err: std::io::Error) -> Self {
        AstudiosError::Io(err)
    }
}

impl From<reqwest::Error> for AstudiosError {
    fn from(err: reqwest::Error) -> Self {
        AstudiosError::Network(err)
    }
}

impl From<serde_json::Error> for AstudiosError {
    fn from(err: serde_json::Error) -> Self {
        AstudiosError::Parse(err.to_string())
    }
}

impl From<quick_xml::DeError> for AstudiosError {
    fn from(err: quick_xml::DeError) -> Self {
        AstudiosError::Parse(err.to_string())
    }
}

impl From<std::str::Utf8Error> for AstudiosError {
    fn from(err: std::str::Utf8Error) -> Self {
        AstudiosError::Utf8(err)
    }
}

impl From<zip::result::ZipError> for AstudiosError {
    fn from(err: zip::result::ZipError) -> Self {
        AstudiosError::Zip(err)
    }
}

impl From<std::time::SystemTimeError> for AstudiosError {
    fn from(err: std::time::SystemTimeError) -> Self {
        AstudiosError::SystemTime(err)
    }
}
