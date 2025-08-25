use crate::model::Content;
use reqwest::blocking::Client;
use std::time::Duration;

const FEED_URL: &str = "https://teamcity.jetbrains.com/guestAuth/repository/download/AndroidStudioReleasesList/.lastSuccessful/android-studio-releases-list.xml";

pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    pub fn new() -> reqwest::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("as-man/0.1.0")
            .build()?;

        Ok(Self { client })
    }

    pub fn fetch_releases(&self) -> Result<Content, Box<dyn std::error::Error>> {
        let response = self.client.get(FEED_URL).send()?;
        let bytes = response.bytes()?;

        let text = std::str::from_utf8(&bytes)?;
        let content: Content = quick_xml::de::from_str(text)?;

        Ok(content)
    }
}
