use assert_cmd::Command;
use insta::assert_snapshot;

/// Test the main help output
#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.arg("--help").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("help_output", stdout);
}

/// Test version output
#[test]
fn test_version_output() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.arg("--version").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Version output should contain the package name and version
    assert!(stdout.contains("astudios"));
    assert_snapshot!("version_output", stdout);
}

/// Test list command help
#[test]
fn test_list_help() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.args(["list", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("list_help_output", stdout);
}

/// Test download command help
#[test]
fn test_download_help() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.args(["download", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("download_help_output", stdout);
}

/// Test install command help
#[test]
fn test_install_help() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.args(["install", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("install_help_output", stdout);
}

/// Test installed command help
#[test]
fn test_installed_help() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.args(["installed", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("installed_help_output", stdout);
}

/// Test use command help
#[test]
fn test_use_help() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.args(["use", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("use_help_output", stdout);
}

/// Test uninstall command help
#[test]
fn test_uninstall_help() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.args(["uninstall", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("uninstall_help_output", stdout);
}

/// Test which command help
#[test]
fn test_which_help() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.args(["which", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("which_help_output", stdout);
}

/// Test update command help
#[test]
fn test_update_help() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.args(["update", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_snapshot!("update_help_output", stdout);
}

/// Test invalid command error
#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("astudios").unwrap();
    let output = cmd.arg("invalid-command").output().unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_snapshot!("invalid_command_error", stderr);
}
