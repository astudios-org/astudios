use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub enum Downloader {
    Reqwest,
    Aria2(PathBuf),
}

impl Downloader {
    /// Detect the best available downloader
    pub fn detect_best() -> Self {
        // Check if aria2 is available in PATH
        if let Ok(aria2_path) = Self::find_aria2() {
            Downloader::Aria2(aria2_path)
        } else {
            Downloader::Reqwest
        }
    }

    /// Find aria2 in system PATH
    pub fn find_aria2() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Try common locations for aria2
        let possible_paths = if cfg!(target_os = "windows") {
            vec![
                "C:\\Program Files\\aria2\\aria2c.exe",
                "C:\\Program Files (x86)\\aria2\\aria2c.exe",
                "aria2c.exe",
            ]
        } else {
            vec![
                "/usr/local/bin/aria2c",
                "/opt/homebrew/bin/aria2c", // macOS Apple Silicon
                "/usr/bin/aria2c",
                "/bin/aria2c",
                "aria2c",
            ]
        };

        for path_str in possible_paths {
            // Try to execute the command to see if it exists
            if let Ok(status) = std::process::Command::new(path_str)
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
            {
                if status.success() {
                    return Ok(PathBuf::from(path_str));
                }
            }

            // Also check if it's in PATH
            if let Ok(output) = std::process::Command::new("which").arg(path_str).output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout);
                    return Ok(PathBuf::from(path.trim()));
                }
            }
        }

        Err("aria2 not found".into())
    }

    /// Download a file from URL to destination
    pub fn download(
        &self,
        url: &str,
        destination: &Path,
        progress_name: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Downloader::Reqwest => self.download_with_reqwest(url, destination, progress_name),
            Downloader::Aria2(path) => {
                self.download_with_aria2(path, url, destination, progress_name)
            }
        }
    }

    /// Download using reqwest (built-in HTTP client) with enhanced progress
    fn download_with_reqwest(
        &self,
        url: &str,
        destination: &Path,
        progress_name: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use reqwest::blocking::Client;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        let mut response = client.get(url).send()?;
        let total_size = response.content_length().unwrap_or(0);

        let pb = if let Some(name) = progress_name {
            if total_size > 0 {
                let pb = ProgressBar::new(total_size);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template(&format!("(1/6) Downloading {name}:\n{{spinner:.green}} [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}}) @ {{bytes_per_sec}}"))
                        .unwrap()
                        .progress_chars("█▉▊▋▌▍▎▏ "),
                );
                pb
            } else {
                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .template(&format!("(1/6) Downloading {name}:\n{{spinner:.green}} {{bytes}} bytes @ {{bytes_per_sec}}"))
                        .unwrap(),
                );
                pb
            }
        } else if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "[{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) @ {bytes_per_sec}",
                    )
                    .unwrap()
                    .progress_chars("█▉▊▋▌▍▎▏ "),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} Downloading... {bytes} bytes @ {bytes_per_sec}")
                    .unwrap(),
            );
            pb
        };

        let mut file = fs::File::create(destination)?;
        let mut downloaded = 0u64;
        let mut buf = [0u8; 8192];

        let start_time = std::time::Instant::now();
        let mut last_update = start_time;

        loop {
            let n = response.read(&mut buf)?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])?;
            downloaded += n as u64;

            // Update progress with rate limiting to avoid flickering
            let now = std::time::Instant::now();
            if now.duration_since(last_update) > std::time::Duration::from_millis(100) {
                pb.set_position(downloaded);
                last_update = now;
            }
        }

        pb.finish_with_message("✅ Download complete");

        Ok(())
    }

    /// Download using aria2 for faster downloads with enhanced progress
    fn download_with_aria2(
        &self,
        aria2_path: &Path,
        url: &str,
        destination: &Path,
        progress_name: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure destination directory exists
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        // Build aria2 command with controlled output
        let mut cmd = Command::new(aria2_path);

        cmd.arg(url)
            .arg("--dir")
            .arg(destination.parent().unwrap_or_else(|| Path::new(".")))
            .arg("--out")
            .arg(
                destination
                    .file_name()
                    .ok_or("Invalid destination filename")?,
            )
            .arg("--max-connection-per-server=16")
            .arg("--split=16")
            .arg("--min-split-size=1M")
            .arg("--continue=true")
            .arg("--max-tries=3")
            .arg("--retry-wait=5")
            .arg("--human-readable=false") // Use machine-readable format
            .arg("--console-log-level=error")
            .arg("--no-color=true") // Disable ANSI color codes
            .stderr(Stdio::piped());

        // Get file size for progress bar (try HEAD request first)
        let file_size = reqwest::blocking::Client::new()
            .head(url)
            .send()
            .ok()
            .and_then(|r| {
                
                r
                    .headers()
                    .get("content-length")?
                    .to_str()
                    .ok()?
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(0);

        let pb = if let Some(name) = progress_name {
            if file_size > 0 {
                let pb = ProgressBar::new(file_size);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template(&format!("(1/6) Downloading {name} with aria2:\n{{spinner:.green}} [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}}) @ {{bytes_per_sec}}"))
                        .unwrap()
                        .progress_chars("█▉▊▋▌▍▎▏ "),
                );
                pb
            } else {
                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .template(&format!(
                            "(1/6) Downloading {name} with aria2:\n{{spinner:.green}} {{msg}}"
                        ))
                        .unwrap(),
                );
                pb
            }
        } else if file_size > 0 {
            let pb = ProgressBar::new(file_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "[{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) @ {bytes_per_sec}",
                    )
                    .unwrap()
                    .progress_chars("█▉▊▋▌▍▎▏ "),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} Downloading with aria2... {msg}")
                    .unwrap(),
            );
            pb
        };

        let mut child = cmd.spawn()?;

        // Let aria2 run and we'll check file size periodically
        let pb_clone = pb.clone();
        let destination_clone = destination.to_path_buf();
        let file_size_clone = file_size;

        // Start monitoring thread for progress
        let handle = std::thread::spawn(move || {
            let mut last_size = 0u64;
            let mut last_update = std::time::Instant::now();

            loop {
                std::thread::sleep(std::time::Duration::from_millis(200));

                if let Ok(metadata) = fs::metadata(&destination_clone) {
                    let current_size = metadata.len();

                    // Rate limit updates
                    let now = std::time::Instant::now();
                    if now.duration_since(last_update) > std::time::Duration::from_millis(100) {
                        if file_size_clone > 0 {
                            pb_clone.set_position(current_size);
                        } else if current_size > last_size {
                            pb_clone.set_message(format!("{current_size} bytes"));
                        }
                        last_update = now;
                    }

                    last_size = current_size;

                    // Stop if file seems complete (based on expected size or no growth)
                    if file_size_clone > 0 && current_size >= file_size_clone {
                        break;
                    }
                }
            }
        });

        let status = child.wait()?;

        // Wait for monitoring thread to complete
        let _ = handle.join();

        // Final update to ensure progress bar shows completion
        if let Ok(metadata) = fs::metadata(destination) {
            let final_size = metadata.len();
            if file_size > 0 {
                pb.set_position(final_size.min(file_size));
            }
        }

        if status.success() {
            pb.finish_with_message("✅ Download complete");
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

            pb.finish_with_message("❌ Download failed");
            Err(format!("aria2 download failed: {}", stderr.trim()).into())
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
