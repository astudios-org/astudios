use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Content {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "item")]
    pub items: Vec<AndroidStudio>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AndroidStudio {
    pub name: String,
    pub build: String,
    pub version: String,
    pub channel: String,
    #[serde(rename = "platformBuild")]
    pub platform_build: String,
    #[serde(rename = "platformVersion")]
    pub platform_version: String,
    pub date: String,
    #[serde(rename = "download")]
    pub downloads: Vec<Download>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Download {
    pub link: String,
    pub size: String,
    pub checksum: String,
}

impl AndroidStudio {
    pub fn is_release(&self) -> bool {
        self.channel == "Release"
    }

    pub fn is_beta(&self) -> bool {
        self.channel == "Beta"
    }

    pub fn is_canary(&self) -> bool {
        self.channel == "Canary"
    }

    pub fn is_rc(&self) -> bool {
        self.channel == "RC"
    }

    pub fn is_patch(&self) -> bool {
        self.channel == "Patch"
    }

    pub fn get_macos_download(&self) -> Option<&Download> {
        self.downloads.iter().find(|d| d.link.contains("mac"))
    }

    pub fn get_windows_download(&self) -> Option<&Download> {
        self.downloads.iter().find(|d| d.link.contains("windows"))
    }

    pub fn get_linux_download(&self) -> Option<&Download> {
        self.downloads.iter().find(|d| d.link.contains("linux"))
    }
}
