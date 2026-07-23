use std::process::Command;
use std::path::Path;

fn kodehold_bin() -> std::path::PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("debug");
    path.push("kodehold");
    path
}

#[test]
fn test_init_creates_project() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let output = Command::new(kodehold_bin())
        .args(["init", "test-project", "--language", "rust"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    assert!(output.status.success(), "init failed: {}", String::from_utf8_lossy(&output.stderr));

    let project = dir.join("test-project");
    assert!(project.join("kodehold.yaml").exists(), "kodehold.yaml not created");
    assert!(project.join(".gitignore").exists(), ".gitignore not created");
    assert!(project.join(".kodehold").exists(), ".kodehold dir not created");
    assert!(Path::new(&project.join(".git")).join("HEAD").exists(), "git repo not initialized");
}

#[test]
fn test_init_creates_with_correct_config() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let output = Command::new(kodehold_bin())
        .args(["init", "my-api", "--language", "python"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    assert!(output.status.success());

    let yaml_path = dir.join("my-api").join("kodehold.yaml");
    let content = std::fs::read_to_string(&yaml_path).unwrap();
    assert!(content.contains("language: python"), "wrong language in config");
    assert!(content.contains("name: my-api"), "wrong name in config");
}

#[test]
fn test_init_refuses_existing_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    std::fs::create_dir(dir.join("exists")).unwrap();

    let output = Command::new(kodehold_bin())
        .args(["init", "exists"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    assert!(!output.status.success(), "should have failed for existing dir");
}

#[test]
fn test_state_shows_init_after_creation() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    Command::new(kodehold_bin())
        .args(["init", "test-state"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    let output = Command::new(kodehold_bin())
        .args(["state", "--path", dir.join("test-state").to_str().unwrap()])
        .output()
        .expect("failed to run state");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("INIT"), "state should show INIT, got: {stdout}");
}

#[test]
fn test_verify_on_init_project() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    Command::new(kodehold_bin())
        .args(["init", "test-verify"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    let output = Command::new(kodehold_bin())
        .args(["verify", "--path", dir.join("test-verify").to_str().unwrap()])
        .output()
        .expect("failed to run verify");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "verify failed: {stdout}");
    assert!(stdout.to_uppercase().contains("PASSED"), "no pass message: {stdout}");
}

#[test]
fn test_ship_bumps_version() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    Command::new(kodehold_bin())
        .args(["init", "test-ship"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    let ship_path = dir.join("test-ship");
    let output = Command::new(kodehold_bin())
        .args(["ship", "--bump", "minor", "--path", ship_path.to_str().unwrap()])
        .output()
        .expect("failed to run ship");

    assert!(output.status.success(), "ship failed: {}", String::from_utf8_lossy(&output.stderr));

    let yaml_content = std::fs::read_to_string(ship_path.join("kodehold.yaml")).unwrap();
    assert!(yaml_content.contains("version: 0.2.0"), "version not bumped: {yaml_content}");
}

#[test]
fn test_ship_creates_tag() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    Command::new(kodehold_bin())
        .args(["init", "ship-tag"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    let ship_path = dir.join("ship-tag");
    Command::new(kodehold_bin())
        .args(["ship", "--bump", "patch", "--path", ship_path.to_str().unwrap()])
        .output()
        .expect("failed to run ship");

    let tag_output = Command::new("git")
        .args(["-C", ship_path.to_str().unwrap(), "tag", "-l"])
        .output()
        .expect("failed to list tags");

    let tags = String::from_utf8_lossy(&tag_output.stdout);
    assert!(tags.contains("v2-init"), "missing init tag");
    assert!(tags.contains("v0.1.1"), "missing version tag, got: {tags}");
}

#[test]
fn test_json_output() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let output = Command::new(kodehold_bin())
        .args(["init", "json-test", "--json"])
        .current_dir(dir)
        .output()
        .expect("failed to run init with json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["state"], "INIT");
}
