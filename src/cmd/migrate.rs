use crate::cli::{BridgeArgs, BridgeCommand, MigrateArgs};
use crate::state;

pub fn run_migrate(args: MigrateArgs) -> anyhow::Result<()> {
    let path = args
        .path
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or(std::env::current_dir().unwrap_or_default());

    let mut results = Vec::new();

    // Check for v1.0 .kodehold-state
    let v1_state = path.join(".kodehold-state");
    if v1_state.exists() {
        results.push(migrate_v1_state(&path, &v1_state, args.dry_run)?);
    } else {
        results.push(MigrateItem {
            item: "v1 state file".into(),
            status: "skipped".into(),
            detail: "no .kodehold-state found".into(),
        });
    }

    // Check for v1.0 lifecycle markers
    let markers = [
        ".design_reviewed",
        ".second_opinion_done",
        ".testers_done",
        ".code_reviewed",
        ".impact_analysis_done",
    ];
    let found_markers: Vec<&str> = markers.iter().filter(|m| path.join(m).exists()).copied().collect();
    if !found_markers.is_empty() {
        results.push(migrate_v1_markers(&path, &found_markers, args.dry_run)?);
    }

    // Check if v2.0 kodehold.yaml exists
    let v2_config = path.join("kodehold.yaml");
    if !v2_config.exists() {
        results.push(create_v2_config(&path, args.dry_run)?);
    }

    // Check for v1.0 workspace.py
    let v1_scripts = path.join("scripts").join("workspace.py");
    if v1_scripts.exists() {
        results.push(MigrateItem {
            item: "v1.0 scripts".into(),
            status: "info".into(),
            detail: format!("found v1.0 scripts at {}", v1_scripts.display()),
        });
    }

    // Report
    if args.json {
        let items: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "item": r.item,
                    "status": r.status,
                    "detail": r.detail,
                })
            })
            .collect();
        let status = if results.iter().any(|r| r.status == "error") {
            "fail"
        } else {
            "ok"
        };
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "status": status,
            "migrations": items,
        }))?);
    } else {
        println!("━━━ Migration Report ━━━");
        for r in &results {
            let icon = match r.status.as_str() {
                "ok" => "✓",
                "skipped" => "–",
                "info" => "i",
                "error" => "✗",
                _ => "?",
            };
            println!("  {icon} {}: {}", r.item, r.detail);
        }
        println!();
        if results.iter().any(|r| r.status == "error") {
            println!("  Migration completed with errors");
        } else if args.dry_run {
            println!("  Dry-run complete. Run without --dry-run to apply.");
        } else {
            println!("  ✓ Migration complete");
        }
    }

    Ok(())
}

pub fn run_bridge(args: BridgeArgs) -> anyhow::Result<()> {
    match args.command {
        BridgeCommand::Gate { transition } => bridge_gate(&args.path, &transition, args.dry_run),
    }
}

struct MigrateItem {
    item: String,
    status: String,
    detail: String,
}

fn migrate_v1_state(path: &std::path::Path, state_file: &std::path::Path, dry_run: bool) -> anyhow::Result<MigrateItem> {
    let content = std::fs::read_to_string(state_file)?;

    // Parse v1.0 state
    let v1_state = content
        .lines()
        .find(|l| l.starts_with("STATE="))
        .and_then(|l| l.split('=').nth(1))
        .unwrap_or("INIT");

    let state = match v1_state {
        "INIT" => state::LifecycleState::Init,
        "ACTIVE" => state::LifecycleState::Active,
        "REVIEW" => state::LifecycleState::Review,
        "CLOSED" => state::LifecycleState::Closed,
        "REOPEN" => state::LifecycleState::Reopen(1),
        _ => state::LifecycleState::Init,
    };

    if dry_run {
        return Ok(MigrateItem {
            item: ".kodehold-state".into(),
            status: "info".into(),
            detail: format!("would create tag {} ({})", state.tag_name(), v1_state),
        });
    }

    // Ensure we have at least one commit
    let repo = git2::Repository::open(path);
    let has_commits = repo.is_ok() && repo.unwrap().revparse_single("HEAD").is_ok();
    if !has_commits {
        // Create an initial commit before tagging
        if let Ok(repo) = git2::Repository::init(path) {
            let sig = repo.signature()?;
            let tree = {
                let mut index = repo.index()?;
                let tree_id = index.write_tree()?;
                repo.find_tree(tree_id)?
            };
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit (migrated from v1.0)", &tree, &[])?;
        }
    }

    // Create git tag
    match state::tag_project(path, &state, &format!("Migrated from v1.0 state: {v1_state}")) {
        Ok(_) => Ok(MigrateItem {
            item: ".kodehold-state".into(),
            status: "ok".into(),
            detail: format!("tag {} created ({})", state.tag_name(), v1_state),
        }),
        Err(e) => Ok(MigrateItem {
            item: ".kodehold-state".into(),
            status: "error".into(),
            detail: format!("failed to create tag: {e}"),
        }),
    }
}

fn migrate_v1_markers(path: &std::path::Path, markers: &[&str], dry_run: bool) -> anyhow::Result<MigrateItem> {
    if dry_run {
        return Ok(MigrateItem {
            item: "lifecycle markers".into(),
            status: "info".into(),
            detail: format!("would archive {} marker file(s)", markers.len()),
        });
    }

    let archive_dir = path.join(".kodehold").join("v1-markers");
    std::fs::create_dir_all(&archive_dir)?;

    for marker in markers {
        let src = path.join(marker);
        let dst = archive_dir.join(marker.trim_start_matches('.'));
        if src.exists() {
            std::fs::rename(&src, &dst)?;
        }
    }

    Ok(MigrateItem {
        item: "lifecycle markers".into(),
        status: "ok".into(),
        detail: format!("{} marker(s) archived to .kodehold/v1-markers/", markers.len()),
    })
}

fn create_v2_config(path: &std::path::Path, dry_run: bool) -> anyhow::Result<MigrateItem> {
    // Try to detect the project language
    let language = if path.join("Cargo.toml").exists() {
        "rust"
    } else if path.join("requirements.txt").exists() || path.join("setup.py").exists() {
        "python"
    } else {
        "rust"
    };

    if dry_run {
        return Ok(MigrateItem {
            item: "kodehold.yaml".into(),
            status: "info".into(),
            detail: format!("would create kodehold.yaml (language: {language})"),
        });
    }

    let config = crate::config::Config::default_for(
        path.file_name().and_then(|n| n.to_str()).unwrap_or("project"),
        Some(language),
    );
    config.save(path)?;

    Ok(MigrateItem {
        item: "kodehold.yaml".into(),
        status: "ok".into(),
        detail: format!("created kodehold.yaml (language: {language})"),
    })
}

fn bridge_gate(path: &Option<String>, transition: &str, dry_run: bool) -> anyhow::Result<()> {
    let workspace_path = path
        .as_ref()
        .cloned()
        .unwrap_or(std::env::current_dir().unwrap_or_default().to_string_lossy().to_string());

    // Look for v1.0 gate.py
    let gate_script = std::path::Path::new(&workspace_path)
        .join("scripts")
        .join("gate.py");

    if !gate_script.exists() {
        anyhow::bail!("v1.0 gate.py not found at {}", gate_script.display());
    }

    if dry_run {
        println!("  Would run: python3 {} --transition {} --project-path {} --yes",
            gate_script.display(), transition, workspace_path);
        return Ok(());
    }

    // Map v2.0 transition names to v1.0
    let v1_transition = match transition {
        "INIT_TO_ACTIVE" => "INIT_TO_ACTIVE",
        "ACTIVE_TO_REVIEW" => "ACTIVE_TO_REVIEW",
        "REVIEW_TO_CLOSED" => "REVIEW_TO_CLOSED",
        "CLOSED_TO_REOPEN" => "CLOSED_TO_REOPEN",
        "REOPEN_TO_ACTIVE" => "REOPEN_TO_ACTIVE",
        other => anyhow::bail!("Unknown transition: {other}"),
    };

    let output = std::process::Command::new("python3")
        .arg(&gate_script)
        .arg("--transition")
        .arg(v1_transition)
        .arg("--project-path")
        .arg(&workspace_path)
        .arg("--yes")
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run v1.0 gate.py: {e}"))?;

    if output.status.success() {
        println!("  ✓ v1.0 gate {transition} passed");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("v1.0 gate {transition} failed:\n{stderr}");
    }

    Ok(())
}
