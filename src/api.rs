use crate::{config::Config, error::AstudiosError, model::AndroidStudioReleasesList};
use reqwest::blocking::Client;
use std::time::Duration;

/// HTTP client for interacting with JetBrains API
pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    /// Create a new API client with default configuration
    pub fn new() -> Result<Self, AstudiosError> {
        Self::with_timeout(Config::NETWORK_TIMEOUT_SECS)
    }

    /// Create a new API client with custom timeout
    pub fn with_timeout(seconds: u64) -> Result<Self, AstudiosError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(seconds))
            .user_agent(Config::user_agent())
            .build()?;

        Ok(Self { client })
    }

    /// Fetch Android Studio releases from JetBrains API
    pub fn fetch_releases(&self) -> Result<AndroidStudioReleasesList, AstudiosError> {
        let response = self.client.get(Config::RELEASES_FEED_URL).send()?;
        let bytes = response.bytes()?;

        let text = std::str::from_utf8(&bytes)?;
        let content: AndroidStudioReleasesList = quick_xml::de::from_str(text)?;

        Ok(content)
    }
}
