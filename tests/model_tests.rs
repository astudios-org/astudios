use astudios::model::*;
use insta::assert_yaml_snapshot;

/// Test AndroidStudio model serialization
#[test]
fn test_android_studio_model() {
    let android_studio = AndroidStudio {
        name: "Android Studio Hedgehog".to_string(),
        version: "2023.1.1".to_string(),
        build: "AI-231.9392.1.2311.11076708".to_string(),
        date: "2023-07-26".to_string(),
        channel: "Release".to_string(),
        platform_build: "231.9392.1".to_string(),
        platform_version: "2023.1.1".to_string(),
        downloads: vec![Download {
            link: "https://example.com/android-studio-mac.dmg".to_string(),
            size: "1024000000".to_string(),
            checksum: "abc123def456".to_string(),
        }],
    };

    assert_yaml_snapshot!("android_studio_model", android_studio);
}

/// Test ReleaseChannel enum variants
#[test]
fn test_release_channels() {
    let channels = [
        ReleaseChannel::Release,
        ReleaseChannel::Beta,
        ReleaseChannel::Canary,
        ReleaseChannel::ReleaseCandidate,
        ReleaseChannel::Patch,
    ];

    // Since ReleaseChannel doesn't implement Serialize, we'll test the string representations
    let channel_strings: Vec<String> = channels.iter().map(|c| format!("{c:?}")).collect();
    assert_yaml_snapshot!("release_channels", channel_strings);
}

/// Test Download model
#[test]
fn test_download_model() {
    let download = Download {
        link: "https://dl.google.com/dl/android/studio/install/2023.1.1.23/android-studio-2023.1.1.23-mac_arm.dmg".to_string(),
        size: "1073741824".to_string(),
        checksum: "sha256:abcdef1234567890".to_string(),
    };

    assert_yaml_snapshot!("download_model", download);
}

/// Test AndroidStudioReleasesList model
#[test]
fn test_android_studio_releases_list() {
    let studio_list = AndroidStudioReleasesList {
        version: "1.0".to_string(),
        items: vec![
            AndroidStudio {
                name: "Android Studio Hedgehog".to_string(),
                version: "2023.1.1".to_string(),
                build: "AI-231.9392.1.2311.11076708".to_string(),
                date: "2023-07-26".to_string(),
                channel: "Release".to_string(),
                platform_build: "231.9392.1".to_string(),
                platform_version: "2023.1.1".to_string(),
                downloads: vec![Download {
                    link: "https://example.com/android-studio-mac.dmg".to_string(),
                    size: "1024000000".to_string(),
                    checksum: "abc123def456".to_string(),
                }],
            },
            AndroidStudio {
                name: "Android Studio Iguana".to_string(),
                version: "2023.2.1".to_string(),
                build: "AI-232.1234.5.2321.11234567".to_string(),
                date: "2023-08-15".to_string(),
                channel: "Beta".to_string(),
                platform_build: "232.1234.5".to_string(),
                platform_version: "2023.2.1".to_string(),
                downloads: vec![Download {
                    link: "https://example.com/android-studio-beta-mac.dmg".to_string(),
                    size: "1100000000".to_string(),
                    checksum: "def456ghi789".to_string(),
                }],
            },
        ],
    };

    assert_yaml_snapshot!("android_studio_releases_list", studio_list);
}

/// Test AndroidStudio utility methods
#[test]
fn test_android_studio_methods() {
    let stable_release = AndroidStudio {
        name: "Android Studio Hedgehog".to_string(),
        version: "2023.1.1".to_string(),
        build: "AI-231.9392.1.2311.11076708".to_string(),
        date: "2023-07-26".to_string(),
        channel: "Release".to_string(),
        platform_build: "231.9392.1".to_string(),
        platform_version: "2023.1.1".to_string(),
        downloads: vec![],
    };

    let beta_release = AndroidStudio {
        name: "Android Studio Iguana".to_string(),
        version: "2023.2.1".to_string(),
        build: "AI-232.1234.5.2321.11234567".to_string(),
        date: "2023-08-15".to_string(),
        channel: "Beta".to_string(),
        platform_build: "232.1234.5".to_string(),
        platform_version: "2023.2.1".to_string(),
        downloads: vec![],
    };

    let canary_release = AndroidStudio {
        name: "Android Studio Jellyfish".to_string(),
        version: "2023.3.1".to_string(),
        build: "AI-233.5678.9.2331.11345678".to_string(),
        date: "2023-09-01".to_string(),
        channel: "Canary".to_string(),
        platform_build: "233.5678.9".to_string(),
        platform_version: "2023.3.1".to_string(),
        downloads: vec![],
    };

    // Test channel detection methods
    assert!(stable_release.is_release());
    assert!(!stable_release.is_beta());
    assert!(!stable_release.is_canary());

    assert!(!beta_release.is_release());
    assert!(beta_release.is_beta());
    assert!(!beta_release.is_canary());

    assert!(!canary_release.is_release());
    assert!(!canary_release.is_beta());
    assert!(canary_release.is_canary());

    let channel_results = vec![
        ("stable_is_release", stable_release.is_release()),
        ("stable_is_beta", stable_release.is_beta()),
        ("beta_is_release", beta_release.is_release()),
        ("beta_is_beta", beta_release.is_beta()),
        ("canary_is_release", canary_release.is_release()),
        ("canary_is_canary", canary_release.is_canary()),
    ];

    assert_yaml_snapshot!("channel_detection_results", channel_results);

    // Test display names
    let display_names = vec![
        ("stable", stable_release.display_name()),
        ("beta", beta_release.display_name()),
        ("canary", canary_release.display_name()),
    ];

    assert_yaml_snapshot!("display_names", display_names);
}

/// Test AndroidStudioVersion model
#[test]
fn test_android_studio_version() {
    let version = AndroidStudioVersion::new(
        "2023.1.1".to_string(),
        "AI-231.9392.1.2311.11076708".to_string(),
        "AI".to_string(),
        "231.9392.1.2311.11076708".to_string(),
        "Android Studio".to_string(),
    );

    // Test individual methods
    let version_info = vec![
        ("display_version", version.display_version()),
        ("identifier", version.identifier()),
        ("is_stable", version.is_stable().to_string()),
        ("to_string", version.to_string()),
    ];

    assert_yaml_snapshot!("android_studio_version_methods", version_info);
}

/// Test InstalledAndroidStudio display methods
#[test]
fn test_installed_android_studio_display() {
    use std::path::PathBuf;

    let version = AndroidStudioVersion::new(
        "2023.1.1".to_string(),
        "AI-231.9392.1.2311.11076708".to_string(),
        "AI".to_string(),
        "231.9392.1.2311.11076708".to_string(),
        "Android Studio".to_string(),
    );

    let installed = InstalledAndroidStudio {
        path: PathBuf::from("/Applications/Android Studio Hedgehog.app"),
        version,
    };

    let display_info = vec![
        ("display_name", installed.display_name()),
        ("identifier", installed.identifier()),
        ("path", installed.path.to_string_lossy().to_string()),
    ];

    assert_yaml_snapshot!("installed_display_info", display_info);
}

/// Test AndroidStudio channel type conversion
#[test]
fn test_channel_type_conversion() {
    let test_cases = vec![
        ("Release", "Release"),
        ("Beta", "Beta"),
        ("Canary", "Canary"),
        ("RC", "RC"),
        ("Patch", "Patch"),
        ("Unknown", "Release"), // Default fallback
    ];

    let results: Vec<(String, String)> = test_cases
        .into_iter()
        .map(|(input, _expected)| {
            let studio = AndroidStudio {
                name: "Test Studio".to_string(),
                version: "1.0.0".to_string(),
                build: "AI-100.0.0".to_string(),
                date: "2023-01-01".to_string(),
                channel: input.to_string(),
                platform_build: "100.0.0".to_string(),
                platform_version: "1.0.0".to_string(),
                downloads: vec![],
            };
            (input.to_string(), format!("{:?}", studio.channel_type()))
        })
        .collect();

    assert_yaml_snapshot!("channel_type_conversion", results);
}
