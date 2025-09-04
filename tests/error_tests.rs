use astudios::error::AstudiosError;
use insta::assert_yaml_snapshot;

/// Test AstudiosError variants and their display messages
#[test]
fn test_error_variants() {
    let errors = vec![
        AstudiosError::General("General error message".to_string()),
        AstudiosError::VersionNotFound("2023.1.1".to_string()),
        AstudiosError::Download("Download failed".to_string()),
        AstudiosError::Installation("Installation failed".to_string()),
        AstudiosError::Extraction("Extraction failed".to_string()),
        AstudiosError::Config("Configuration error".to_string()),
        AstudiosError::PrerequisiteNotMet("Prerequisite check failed".to_string()),
        AstudiosError::InsufficientResources("Not enough resources".to_string()),
        AstudiosError::PermissionDenied("Permission denied".to_string()),
        AstudiosError::NetworkUnavailable("Network timeout".to_string()),
        AstudiosError::Path("Invalid path".to_string()),
        AstudiosError::Platform("Platform error".to_string()),
        AstudiosError::Cache("Cache error".to_string()),
        AstudiosError::Parse("Parse error".to_string()),
        AstudiosError::DownloaderNotFound("aria2 not found".to_string()),
    ];

    let error_messages: Vec<(String, String)> = errors
        .into_iter()
        .map(|e| {
            let variant_name = match &e {
                AstudiosError::General(_) => "General",
                AstudiosError::VersionNotFound(_) => "VersionNotFound",
                AstudiosError::Download(_) => "Download",
                AstudiosError::Installation(_) => "Installation",
                AstudiosError::Extraction(_) => "Extraction",
                AstudiosError::Network(_) => "Network",
                AstudiosError::Config(_) => "Config",
                AstudiosError::PrerequisiteNotMet(_) => "PrerequisiteNotMet",
                AstudiosError::InsufficientResources(_) => "InsufficientResources",
                AstudiosError::PermissionDenied(_) => "PermissionDenied",
                AstudiosError::NetworkUnavailable(_) => "NetworkUnavailable",
                AstudiosError::Path(_) => "Path",
                AstudiosError::Platform(_) => "Platform",
                AstudiosError::Cache(_) => "Cache",
                AstudiosError::Parse(_) => "Parse",
                AstudiosError::DownloaderNotFound(_) => "DownloaderNotFound",
                AstudiosError::Io(_) => "Io",
                AstudiosError::Utf8(_) => "Utf8",
                AstudiosError::Zip(_) => "Zip",
                AstudiosError::SystemTime(_) => "SystemTime",
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
    let astudios_error = AstudiosError::from(io_error);
    let error_string = astudios_error.to_string();

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
    let astudios_error = AstudiosError::from(json_error);
    let error_string = astudios_error.to_string();

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
            AstudiosError::Download(
                "Failed to download Android Studio 2023.1.1: Connection timeout".to_string(),
            ),
        ),
        (
            "installation_failure",
            AstudiosError::Installation(
                "Failed to install to /Applications: Permission denied".to_string(),
            ),
        ),
        (
            "extraction_failure",
            AstudiosError::Extraction("Failed to extract DMG: Invalid archive format".to_string()),
        ),
        (
            "prerequisite_failure",
            AstudiosError::PrerequisiteNotMet(
                "System requirements not met: macOS 10.14+ required".to_string(),
            ),
        ),
        (
            "version_not_found",
            AstudiosError::VersionNotFound("2023.1.1".to_string()),
        ),
        (
            "permission_denied",
            AstudiosError::PermissionDenied("Cannot write to /Applications directory".to_string()),
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
        AstudiosError::NetworkUnavailable("Timeout".to_string()),
        AstudiosError::Download("Download failed".to_string()),
        AstudiosError::Installation("Installation failed".to_string()),
        AstudiosError::PermissionDenied("Permission denied".to_string()),
        AstudiosError::InsufficientResources("Disk full".to_string()),
        AstudiosError::Config("Config error".to_string()),
        AstudiosError::General("General error".to_string()),
        AstudiosError::VersionNotFound("1.0.0".to_string()),
        AstudiosError::PrerequisiteNotMet("Missing dependency".to_string()),
    ];

    let categorized_errors: Vec<(String, bool, bool, bool)> = errors
        .into_iter()
        .map(|e| {
            let is_network_related = matches!(
                e,
                AstudiosError::Network(_) | AstudiosError::NetworkUnavailable(_)
            );
            let is_user_actionable = matches!(
                e,
                AstudiosError::PermissionDenied(_)
                    | AstudiosError::InsufficientResources(_)
                    | AstudiosError::Config(_)
                    | AstudiosError::PrerequisiteNotMet(_)
            );
            let is_retryable = matches!(
                e,
                AstudiosError::Network(_)
                    | AstudiosError::NetworkUnavailable(_)
                    | AstudiosError::Download(_)
            );

            let variant_name = format!("{e:?}")
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
    let astudios_error = AstudiosError::from(utf8_error);
    let error_string = astudios_error.to_string();

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
    let astudios_error = AstudiosError::from(time_error);
    let error_string = astudios_error.to_string();

    let error_info = vec![
        ("type".to_string(), "SystemTimeError".to_string()),
        (
            "contains_time".to_string(),
            error_string.contains("time").to_string(),
        ),
    ];

    assert_yaml_snapshot!("system_time_error_conversion", error_info);
}
