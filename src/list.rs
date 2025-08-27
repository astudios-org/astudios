use crate::{
    api::ApiClient,
    config::Config,
    error::AsManError,
    model::{AndroidStudio, AndroidStudioReleasesList},
    progress::ProgressReporter,
};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

/// Manages Android Studio releases listing with caching support
pub struct AndroidStudioLister {
    cache_dir: PathBuf,
}

impl AndroidStudioLister {
    /// Create a new Android Studio lister with default cache directory
    pub fn new() -> Result<Self, AsManError> {
        let cache_dir = Config::cache_dir();
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Create a new Android Studio lister with custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self, AsManError> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Get Android Studio releases with caching
    pub fn get_releases(&self) -> Result<AndroidStudioReleasesList, AsManError> {
        let cache_path = self.cache_dir.join("releases.json");

        // Check if cache exists and is valid
        if let Some(cached) = self.load_cached_releases(&cache_path)? {
            return Ok(cached);
        }

        // Fetch fresh data
        let mut reporter = ProgressReporter::new(true);
        let pb = reporter.create_spinner("Fetching latest releases from JetBrains...");

        let client = ApiClient::new()?;
        let content = client.fetch_releases()?;

        // Cache the data
        self.save_releases_to_cache(&cache_path, &content)?;
        reporter.finish_with_success("Releases fetched successfully");

        Ok(content)
    }

    /// Get the latest stable release
    pub fn get_latest_release(&self) -> Result<AndroidStudio, AsManError> {
        let releases = self.get_releases()?;
        releases.items
            .into_iter()
            .find(|item| item.is_release())
            .ok_or_else(|| AsManError::VersionNotFound("No release versions available".to_string()))
    }

    /// Get the latest pre-release (beta or canary)
    pub fn get_latest_prerelease(&self) -> Result<AndroidStudio, AsManError> {
        let releases = self.get_releases()?;
        releases.items
            .into_iter()
            .find(|item| item.is_beta() || item.is_canary())
            .ok_or_else(|| AsManError::VersionNotFound("No pre-release versions available".to_string()))
    }

    /// Find a version by query string (supports partial matches)
    pub fn find_version_by_query(&self, query: &str) -> Result<AndroidStudio, AsManError> {
        let releases = self.get_releases()?;
        let query = query.to_lowercase();

        // Try exact version match first
        if let Some(item) = releases.items.iter().find(|item| item.version == query) {
            return Ok(item.clone());
        }

        // Try partial version match
        if let Some(item) = releases.items.iter().find(|item| item.version.to_lowercase().contains(&query)) {
            return Ok(item.clone());
        }

        // Try name match
        if let Some(item) = releases.items.iter().find(|item| item.name.to_lowercase().contains(&query)) {
            return Ok(item.clone());
        }

        // Try build number match
        if let Some(item) = releases.items.iter().find(|item| item.build.to_lowercase().contains(&query)) {
            return Ok(item.clone());
        }

        // Try channel-based search
        self.find_by_channel_query(&releases.items, &query)
    }

    /// Load releases from cache if valid
    fn load_cached_releases(&self, cache_path: &PathBuf) -> Result<Option<AndroidStudioReleasesList>, AsManError> {
        if !cache_path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(cache_path)?;
        let modified = metadata.modified()?;
        let age = SystemTime::now().duration_since(modified)?;

        if age < Duration::from_secs(Config::CACHE_DURATION_SECS) {
            let data = fs::read_to_string(cache_path)?;
            let content: AndroidStudioReleasesList = serde_json::from_str(&data)?;
            return Ok(Some(content));
        }

        Ok(None)
    }

    /// Save releases to cache
    fn save_releases_to_cache(&self, cache_path: &PathBuf, content: &AndroidStudioReleasesList) -> Result<(), AsManError> {
        let data = serde_json::to_string_pretty(content)?;
        fs::write(cache_path, data)?;
        Ok(())
    }

    /// Find version by channel-based query (e.g., "2023.3.1 Canary 8")
    fn find_by_channel_query(&self, items: &[AndroidStudio], query: &str) -> Result<AndroidStudio, AsManError> {
        let parts: Vec<&str> = query.split_whitespace().collect();
        
        if parts.len() >= 2 {
            let version_part = parts[0];
            let channel_part = parts[1].to_lowercase();

            // Try to find matching version and channel
            if let Some(item) = items.iter().find(|item| {
                item.version.contains(version_part) && 
                item.channel.to_lowercase() == channel_part
            }) {
                return Ok(item.clone());
            }
        }

        Err(AsManError::VersionNotFound(format!(
            "Version '{}' not found. Use 'as-man list' to see available versions.", 
            query
        )))
    }

    /// Filter releases by channel
    pub fn filter_by_channel(
        &self,
        releases: AndroidStudioReleasesList,
        release_only: bool,
        beta_only: bool,
        canary_only: bool,
    ) -> Vec<AndroidStudio> {
        let mut items = releases.items;

        if release_only {
            items.retain(|item| item.is_release());
        }
        if beta_only {
            items.retain(|item| item.is_beta());
        }
        if canary_only {
            items.retain(|item| item.is_canary());
        }

        items
    }
}
