use as_man::error::AsManError;
use insta::assert_yaml_snapshot;

/// Test AsManError variants and their display messages
#[test]
fn test_error_variants() {
    let errors = vec![
        AsManError::General("General error message".to_string()),
        AsManError::VersionNotFound("2023.1.1".to_string()),
        AsManError::Download("Download failed".to_string()),
        AsManError::Installation("Installation failed".to_string()),
        AsManError::Extraction("Extraction failed".to_string()),
        AsManError::Config("Configuration error".to_string()),
        AsManError::PrerequisiteNotMet("Prerequisite check failed".to_string()),
        AsManError::InsufficientResources("Not enough resources".to_string()),
        AsManError::PermissionDenied("Permission denied".to_string()),
        AsManError::NetworkUnavailable("Network timeout".to_string()),
        AsManError::Path("Invalid path".to_string()),
        AsManError::Platform("Platform error".to_string()),
        AsManError::Cache("Cache error".to_string()),
        AsManError::Parse("Parse error".to_string()),
        AsManError::DownloaderNotFound("aria2 not found".to_string()),
    ];

    let error_messages: Vec<(String, String)> = errors
        .into_iter()
        .map(|e| {
            let variant_name = match &e {
                AsManError::General(_) => "General",
                AsManError::VersionNotFound(_) => "VersionNotFound",
                AsManError::Download(_) => "Download",
                AsManError::Installation(_) => "Installation",
                AsManError::Extraction(_) => "Extraction",
                AsManError::Network(_) => "Network",
                AsManError::Config(_) => "Config",
                AsManError::PrerequisiteNotMet(_) => "PrerequisiteNotMet",
                AsManError::InsufficientResources(_) => "InsufficientResources",
                AsManError::PermissionDenied(_) => "PermissionDenied",
                AsManError::NetworkUnavailable(_) => "NetworkUnavailable",
                AsManError::Path(_) => "Path",
                AsManError::Platform(_) => "Platform",
                AsManError::Cache(_) => "Cache",
                AsManError::Parse(_) => "Parse",
                AsManError::DownloaderNotFound(_) => "DownloaderNotFound",
                AsManError::Io(_) => "Io",
                AsManError::Utf8(_) => "Utf8",
                AsManError::Zip(_) => "Zip",
                AsManError::SystemTime(_) => "SystemTime",
            };
            (variant_name.to_string(), e.to_string())
        })
        .collect();

    assert_yaml_snapshot!("error_variants", error_messages);
}

/// Test error conversion from std::io::Error
#[test]
fn test_io_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let as_man_error = AsManError::from(io_error);
    let error_string = as_man_error.to_string();

    let error_info = vec![
        ("type".to_string(), "IoError".to_string()),
        (
            "contains_io".to_string(),
            error_string.contains("IO error").to_string(),
        ),
        (
            "contains_not_found".to_string(),
            error_string.contains("not found").to_string(),
        ),
    ];

    assert_yaml_snapshot!("io_error_conversion", error_info);
}

/// Test error conversion from serde_json::Error
#[test]
fn test_json_error_conversion() {
    let json_str = r#"{"invalid": json"#;
    let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let as_man_error = AsManError::from(json_error);
    let error_string = as_man_error.to_string();

    let error_info = vec![
        ("type".to_string(), "ParseError".to_string()),
        (
            "contains_parse".to_string(),
            error_string.contains("Parse error").to_string(),
        ),
    ];

    assert_yaml_snapshot!("json_error_conversion", error_info);
}

/// Test error chaining and context
#[test]
fn test_error_context() {
    // Test that errors provide meaningful context
    let errors_with_context = vec![
        (
            "download_failure",
            AsManError::Download(
                "Failed to download Android Studio 2023.1.1: Connection timeout".to_string(),
            ),
        ),
        (
            "installation_failure",
            AsManError::Installation(
                "Failed to install to /Applications: Permission denied".to_string(),
            ),
        ),
        (
            "extraction_failure",
            AsManError::Extraction("Failed to extract DMG: Invalid archive format".to_string()),
        ),
        (
            "prerequisite_failure",
            AsManError::PrerequisiteNotMet(
                "System requirements not met: macOS 10.14+ required".to_string(),
            ),
        ),
        (
            "version_not_found",
            AsManError::VersionNotFound("2023.1.1".to_string()),
        ),
        (
            "permission_denied",
            AsManError::PermissionDenied("Cannot write to /Applications directory".to_string()),
        ),
    ];

    let context_info: Vec<(String, String)> = errors_with_context
        .into_iter()
        .map(|(context, error)| (context.to_string(), error.to_string()))
        .collect();

    assert_yaml_snapshot!("error_context", context_info);
}

/// Test error categorization for user-friendly handling
#[test]
fn test_error_categorization() {
    let errors = vec![
        AsManError::NetworkUnavailable("Timeout".to_string()),
        AsManError::Download("Download failed".to_string()),
        AsManError::Installation("Installation failed".to_string()),
        AsManError::PermissionDenied("Permission denied".to_string()),
        AsManError::InsufficientResources("Disk full".to_string()),
        AsManError::Config("Config error".to_string()),
        AsManError::General("General error".to_string()),
        AsManError::VersionNotFound("1.0.0".to_string()),
        AsManError::PrerequisiteNotMet("Missing dependency".to_string()),
    ];

    let categorized_errors: Vec<(String, bool, bool, bool)> = errors
        .into_iter()
        .map(|e| {
            let is_network_related = matches!(
                e,
                AsManError::Network(_) | AsManError::NetworkUnavailable(_)
            );
            let is_user_actionable = matches!(
                e,
                AsManError::PermissionDenied(_)
                    | AsManError::InsufficientResources(_)
                    | AsManError::Config(_)
                    | AsManError::PrerequisiteNotMet(_)
            );
            let is_retryable = matches!(
                e,
                AsManError::Network(_)
                    | AsManError::NetworkUnavailable(_)
                    | AsManError::Download(_)
            );

            let variant_name = format!("{:?}", e)
                .split('(')
                .next()
                .unwrap_or("Unknown")
                .to_string();

            (
                variant_name,
                is_network_related,
                is_user_actionable,
                is_retryable,
            )
        })
        .collect();

    assert_yaml_snapshot!("error_categorization", categorized_errors);
}

/// Test UTF-8 error conversion
#[test]
fn test_utf8_error_conversion() {
    let invalid_utf8 = vec![0, 159, 146, 150]; // Invalid UTF-8 sequence
    let utf8_error = std::str::from_utf8(&invalid_utf8).unwrap_err();
    let as_man_error = AsManError::from(utf8_error);
    let error_string = as_man_error.to_string();

    let error_info = vec![
        ("type".to_string(), "Utf8Error".to_string()),
        (
            "contains_utf8".to_string(),
            error_string.contains("UTF-8").to_string(),
        ),
    ];

    assert_yaml_snapshot!("utf8_error_conversion", error_info);
}

/// Test system time error conversion
#[test]
fn test_system_time_error_conversion() {
    use std::time::{Duration, SystemTime};

    // Create a system time error by trying to subtract more time than available
    let now = SystemTime::UNIX_EPOCH;
    let future_time = now + Duration::from_secs(1);
    let time_error = now.duration_since(future_time).unwrap_err();
    let as_man_error = AsManError::from(time_error);
    let error_string = as_man_error.to_string();

    let error_info = vec![
        ("type".to_string(), "SystemTimeError".to_string()),
        (
            "contains_time".to_string(),
            error_string.contains("time").to_string(),
        ),
    ];

    assert_yaml_snapshot!("system_time_error_conversion", error_info);
}
