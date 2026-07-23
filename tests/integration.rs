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

#[test]
fn test_pipeline_runs_all_phases() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    Command::new(kodehold_bin())
        .args(["init", "test-pl"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    let project = dir.join("test-pl");
    let output = Command::new(kodehold_bin())
        .args(["pipeline", "--path", project.to_str().unwrap()])
        .output()
        .expect("failed to run pipeline");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "pipeline failed: {stdout}");
    assert!(stdout.contains("parse"), "no parse phase: {stdout}");
    assert!(stdout.contains("analyze"), "no analyze phase: {stdout}");
    assert!(stdout.contains("generate"), "no generate phase: {stdout}");
    assert!(stdout.contains("verify"), "no verify phase: {stdout}");
    assert!(stdout.contains("optimize"), "no optimize phase: {stdout}");
}

#[test]
fn test_pipeline_specific_phases() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    Command::new(kodehold_bin())
        .args(["init", "test-pl2"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    let project = dir.join("test-pl2");
    let output = Command::new(kodehold_bin())
        .args(["pipeline", "--path", project.to_str().unwrap(), "--phases", "parse,analyze"])
        .output()
        .expect("failed to run pipeline");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "pipeline failed: {stdout}");
    assert!(stdout.contains("parse"), "no parse: {stdout}");
    assert!(stdout.contains("analyze"), "no analyze: {stdout}");
    assert!(!stdout.contains("verify"), "should not have verify: {stdout}");
}

#[test]
fn test_pipeline_json_output() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    Command::new(kodehold_bin())
        .args(["init", "test-pl3"])
        .current_dir(dir)
        .output()
        .expect("failed to run init");

    let project = dir.join("test-pl3");
    let output = Command::new(kodehold_bin())
        .args(["pipeline", "--path", project.to_str().unwrap(), "--json"])
        .output()
        .expect("failed to run pipeline");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
    let pipeline = parsed["pipeline"].as_array().expect("pipeline should be array");
    assert!(pipeline.len() >= 4, "expected 4+ phases, got {}", pipeline.len());
}

#[test]
fn test_pattern_list_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let patterns_path = tmp.path().join("patterns.json");

    let output = Command::new(kodehold_bin())
        .env("KODEHOLD_PATTERNS_PATH", patterns_path.to_str().unwrap())
        .args(["pattern", "list"])
        .output()
        .expect("failed to run pattern list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Patterns"), "should show pattern count: {stdout}");
}

#[test]
fn test_pattern_add_and_list() {
    let tmp = tempfile::tempdir().unwrap();
    let patterns_path = tmp.path().join("patterns.json");
    let pattern_json = tmp.path().join("test-pattern.json");
    std::fs::write(&pattern_json, r#"{
        "id": "pat-test-001",
        "name": "test-classifier",
        "signature": "fn(Vec<u64>) -> String",
        "tags": ["test", "classification"],
        "language": "rust",
        "description": "A test pattern",
        "template": "fn classify(data: Vec<u64>) -> String {\n    \"test\".to_string()\n}"
    }"#).unwrap();

    let output = Command::new(kodehold_bin())
        .env("KODEHOLD_PATTERNS_PATH", patterns_path.to_str().unwrap())
        .args(["pattern", "add", pattern_json.to_str().unwrap()])
        .output()
        .expect("failed to add pattern");

    assert!(output.status.success(), "add failed: {}", String::from_utf8_lossy(&output.stdout));

    let output = Command::new(kodehold_bin())
        .env("KODEHOLD_PATTERNS_PATH", patterns_path.to_str().unwrap())
        .args(["pattern", "list"])
        .output()
        .expect("failed to list patterns");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-classifier"), "pattern not listed: {stdout}");
}

#[test]
fn test_pattern_search() {
    let tmp = tempfile::tempdir().unwrap();
    let patterns_path = tmp.path().join("patterns.json");

    // First add a pattern
    let pattern_json = tmp.path().join("search-pattern.json");
    std::fs::write(&pattern_json, r#"{
        "id": "pat-search-001",
        "name": "market-regime-classifier",
        "signature": "fn(Vec<OHLCV>) -> Regime",
        "tags": ["trading", "classification"],
        "language": "rust",
        "description": "Classifies market regime from OHLCV data",
        "template": ""
    }"#).unwrap();

    Command::new(kodehold_bin())
        .env("KODEHOLD_PATTERNS_PATH", patterns_path.to_str().unwrap())
        .args(["pattern", "add", pattern_json.to_str().unwrap()])
        .output()
        .expect("failed to add pattern");

    // Search for it
    let output = Command::new(kodehold_bin())
        .env("KODEHOLD_PATTERNS_PATH", patterns_path.to_str().unwrap())
        .args(["pattern", "search", "regime"])
        .output()
        .expect("failed to search patterns");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("regime-classifier"), "should find pattern: {stdout}");
}

