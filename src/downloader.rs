use crate::{config::Config, error::AstudiosError};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Supported download methods with different performance characteristics
#[derive(Debug, Clone)]
pub enum Downloader {
    /// Built-in HTTP client (reqwest)
    Reqwest,
    /// High-performance downloader (aria2) with path to executable
    Aria2(PathBuf),
}

impl Downloader {
    /// Detect the best available downloader based on system availability
    pub fn detect_best() -> Self {
        if let Ok(aria2_path) = Self::find_aria2() {
            Downloader::Aria2(aria2_path)
        } else {
            Downloader::Reqwest
        }
    }

    /// Find aria2 executable in system PATH or common locations
    pub fn find_aria2() -> Result<PathBuf, AstudiosError> {
        for path_str in Config::aria2_search_paths() {
            // Try to execute the command to verify it exists
            if let Ok(status) = Command::new(path_str)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
            {
                if status.success() {
                    return Ok(PathBuf::from(path_str));
                }
            }

            // Check if it's in PATH using 'which' command
            if let Ok(output) = Command::new("which").arg(path_str).output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout);
                    return Ok(PathBuf::from(path.trim()));
                }
            }
        }

        Err(AstudiosError::DownloaderNotFound(
            "aria2 not found in system PATH".to_string(),
        ))
    }

    /// Download a file from URL to destination
    pub fn download(
        &self,
        url: &str,
        destination: &Path,
        progress_name: Option<&str>,
    ) -> Result<(), AstudiosError> {
        // Ensure destination directory exists
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        match self {
            Downloader::Reqwest => self.download_with_reqwest(url, destination, progress_name),
            Downloader::Aria2(path) => {
                self.download_with_aria2(path, url, destination, progress_name)
            }
        }
    }

    /// Download using reqwest (built-in HTTP client)
    fn download_with_reqwest(
        &self,
        url: &str,
        destination: &Path,
        _progress_name: Option<&str>,
    ) -> Result<(), AstudiosError> {
        use reqwest::blocking::Client;

        let client = Client::builder()
            .timeout(Duration::from_secs(Config::DOWNLOAD_TIMEOUT_SECS))
            .build()?;

        let mut response = client.get(url).send()?;
        let mut file = fs::File::create(destination)?;

        std::io::copy(&mut response, &mut file)?;

        Ok(())
    }

    /// Download using aria2 for high-performance downloads
    fn download_with_aria2(
        &self,
        aria2_path: &Path,
        url: &str,
        destination: &Path,
        _progress_name: Option<&str>,
    ) -> Result<(), AstudiosError> {
        let mut cmd = Command::new(aria2_path);

        cmd.arg(url)
            .arg("--dir")
            .arg(destination.parent().unwrap_or_else(|| Path::new(".")))
            .arg("--out")
            .arg(
                destination
                    .file_name()
                    .ok_or(AstudiosError::Path("Invalid destination filename".to_string()))?,
            )
            .arg(format!(
                "--max-connection-per-server={}",
                Config::ARIA2_MAX_CONNECTIONS
            ))
            .arg(format!("--split={}", Config::ARIA2_MAX_CONNECTIONS))
            .arg(format!("--min-split-size={}", Config::ARIA2_MIN_SPLIT_SIZE))
            .arg("--continue=true")
            .arg(format!("--max-tries={}", Config::MAX_DOWNLOAD_RETRIES))
            .arg(format!("--retry-wait={}", Config::DOWNLOAD_RETRY_WAIT_SECS))
            .arg("--human-readable=true")
            .arg("--console-log-level=error")
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let status = child.wait()?;

        if status.success() {
            Ok(())
        } else {
            let stderr = child
                .stderr
                .take()
                .and_then(|mut e| {
                    let mut buffer = String::new();
                    e.read_to_string(&mut buffer).ok().map(|_| buffer)
                })
                .unwrap_or_else(|| "Unknown error".to_string());

            Err(AstudiosError::Download(format!(
                "aria2 download failed: {}",
                stderr.trim()
            )))
        }
    }

    /// Get a human-readable description of the downloader
    pub fn description(&self) -> String {
        match self {
            Downloader::Reqwest => "reqwest (built-in HTTP client)".to_string(),
            Downloader::Aria2(path) => format!("aria2 ({})", path.display()),
        }
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::detect_best()
    }
}

impl std::fmt::Display for Downloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}
