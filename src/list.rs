use std::{
    error::Error,
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use colored::Colorize;

use crate::{
    api::ApiClient,
    model::{AndroidStudio, AndroidStudioReleasesList},
};

pub struct AndroidStudioLister {
    cache_dir: PathBuf,
}

impl AndroidStudioLister {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let cache_dir = home_dir.join(".as-man").join("cache");

        fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    pub fn get_releases(&self) -> Result<AndroidStudioReleasesList, Box<dyn Error>> {
        let cache_path = self.cache_dir.join("releases.json");
        let cache_duration = Duration::from_secs(60 * 60 * 24); // 1 day cache

        // Check if cache exists and is valid
        if cache_path.exists() {
            let metadata = fs::metadata(&cache_path)?;
            let modified = metadata.modified()?;
            let age = SystemTime::now().duration_since(modified)?;

            if age < cache_duration {
                // Use cached data
                println!("{} Using cached version list", "💾".blue());
                let data = fs::read_to_string(&cache_path)?;
                let content: AndroidStudioReleasesList = serde_json::from_str(&data)?;
                return Ok(content);
            }
        }

        // Fetch fresh data
        println!("{} Fetching latest releases from JetBrains...", "🔄".blue());
        let client = ApiClient::new()?;
        let content = client.fetch_releases()?;

        // Cache the data
        let data = serde_json::to_string_pretty(&content)?;
        fs::write(&cache_path, data)?;

        Ok(content)
    }

    pub fn get_latest_release(&self) -> Result<AndroidStudio, Box<dyn Error>> {
        let releases = self.get_releases()?;
        let items = releases.items;

        items.iter()
            .find(|item| item.is_release())
            .cloned()
            .ok_or("No release versions available".into())
    }

    pub fn get_latest_prerelease(&self) -> Result<AndroidStudio, Box<dyn Error>> {
        let releases = self.get_releases()?;
        let items = releases.items;

        items.iter()
            .find(|item| item.is_beta() || item.is_canary())
            .cloned()
            .ok_or("No pre-release versions available".into())
    }

    pub fn find_version_by_query(&self, query: &str) -> Result<AndroidStudio, Box<dyn Error>> {
        let releases = self.get_releases()?;
        let items = releases.items;

        // First try exact version match
        if let Some(item) = items.iter().find(|item| item.version == query) {
            return Ok(item.clone());
        }

        // Try partial version match
        if let Some(item) = items.iter().find(|item| item.version.contains(query)) {
            return Ok(item.clone());
        }

        // Try name match
        if let Some(item) = items
            .iter()
            .find(|item| item.name.to_lowercase().contains(&query.to_lowercase()))
        {
            return Ok(item.clone());
        }

        // Try channel-based search (e.g., "2023.3.1 Canary 8")
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.len() >= 3 {
            let version_part = parts[0];
            let channel_part = parts[1].to_lowercase();
            let build_part = parts[2];

            if let Some(item) = items.iter().find(|item| {
                item.version.contains(version_part)
                    && item.channel.to_lowercase() == channel_part
                    && item.build.contains(build_part)
            }) {
                return Ok(item.clone());
            }
        };

        Err(
            format!("Version '{query}' not found. Use 'as-man list' to see available versions.")
                .into(),
        )
    }
}
