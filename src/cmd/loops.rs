use crate::cli::{
    LoopArgs, LoopCommand, LoopFeatureArgs, LoopListArgs, LoopMaintainArgs, LoopPruneArgs,
    LoopStartArgs,
};
use crate::loops::{LoopConfig, LoopEngine};

pub fn run(args: LoopArgs) -> anyhow::Result<()> {
    match args.command {
        LoopCommand::Start(a) => run_start(a, args.json),
        LoopCommand::Feature(a) => run_feature(a, args.json),
        LoopCommand::List(a) => run_list(a, args.json),
        LoopCommand::Maintain(a) => run_maintain(a, args.json),
        LoopCommand::Prune(a) => run_prune(a, args.json),
    }
}

fn resolve_path(path: Option<String>) -> std::path::PathBuf {
    path.as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

fn load_loop_config(root: &std::path::Path) -> LoopConfig {
    // Try to load from kodehold.yaml first, fall back to defaults
    let yaml_path = root.join("kodehold.yaml");
    if let Ok(content) = std::fs::read_to_string(&yaml_path) {
        if let Ok(config_val) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(loops) = config_val.get("loops") {
                if let Ok(config) = serde_yaml::from_value::<LoopConfig>(loops.clone()) {
                    return config;
                }
            }
        }
    }
    LoopConfig::default()
}

fn run_start(args: LoopStartArgs, json: bool) -> anyhow::Result<()> {
    let root = resolve_path(args.path);
    let config = load_loop_config(&root);

    if !root.join("kodehold.yaml").exists() {
        anyhow::bail!("not a kodehold workspace (no kodehold.yaml found in {})", root.display());
    }

    if json {
        // Single cycle with JSON output
        let engine = LoopEngine::new(&root, config);
        let report = engine.run_cycle()?;
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let interval = args.interval.unwrap_or(60);
        let single_shot = args.once;

        if single_shot {
            eprintln!("━━━ KodeHold Loop (single cycle) ━━━");
            eprintln!("  workspace: {}", root.display());
        } else {
            eprintln!("━━━ KodeHold Loop Daemon ━━━");
            eprintln!("  workspace: {}", root.display());
            eprintln!("  interval: {}s", interval);
            eprintln!("  Press Ctrl+C to stop\n");
        }

        LoopEngine::run_daemon(&root, config, interval, single_shot)?;
    }

    Ok(())
}

fn run_feature(args: LoopFeatureArgs, json: bool) -> anyhow::Result<()> {
    let root = resolve_path(args.path);
    let store = crate::loops::feature::FeatureStore::new(&root);

    let feature = crate::loops::feature::FeatureRequest {
        name: args.name,
        title: args.title.clone(),
        description: args.description.unwrap_or_default(),
        acceptance: args.accept,
        source: crate::loops::feature::FeatureSource::Cli,
        status: crate::loops::feature::FeatureStatus::Pending,
        labels: args.label,
        priority: args.priority.unwrap_or_else(|| "P2".to_string()),
    };

    let path = store.save(&feature)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "ok",
                "feature": feature.name,
                "path": path.to_string_lossy(),
                "title": feature.title,
                "priority": feature.priority,
            }))?
        );
    } else {
        println!("✓ Created feature request '{}'", feature.name);
        println!("  Title: {}", feature.title);
        println!("  Priority: {}", feature.priority);
        println!("  Path: {}", path.display());
        println!();
        println!("  Next: kodehold loop start --once to process");
    }

    Ok(())
}

fn run_list(args: LoopListArgs, json: bool) -> anyhow::Result<()> {
    let root = resolve_path(args.path);
    let engine = LoopEngine::new(&root, load_loop_config(&root));

    // Get features
    let features = engine.features.list(None)?;
    let worktrees = engine.worktrees.list_active()?;

    if json {
        let feat_list: Vec<serde_json::Value> = features
            .iter()
            .map(|f| {
                serde_json::json!({
                    "name": f.name,
                    "title": f.title,
                    "status": f.status.to_string(),
                    "priority": f.priority,
                    "source": format!("{:?}", f.source),
                    "labels": f.labels,
                })
            })
            .collect();

        let wt_list: Vec<serde_json::Value> = worktrees
            .iter()
            .map(|w| {
                serde_json::json!({
                    "name": w.name,
                    "branch": w.branch,
                    "path": w.path.to_string_lossy(),
                    "status": format!("{:?}", w.status),
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "features": feat_list,
                "worktrees": wt_list,
                "summary": {
                    "total_features": features.len(),
                    "active_worktrees": worktrees.len(),
                }
            }))?
        );
    } else {
        println!("━━━ Features ({}) ━━━", features.len());
        if features.is_empty() {
            println!("  (no feature requests found)");
        } else {
            for f in &features {
                let icon = match f.status {
                    crate::loops::feature::FeatureStatus::Pending => "○",
                    crate::loops::feature::FeatureStatus::InProgress => "◷",
                    crate::loops::feature::FeatureStatus::ReadyForReview => "◉",
                    crate::loops::feature::FeatureStatus::Approved => "✓",
                    crate::loops::feature::FeatureStatus::Merged => "✔",
                    crate::loops::feature::FeatureStatus::Cancelled => "✕",
                };
                println!("  {icon} {} [{}] {}", f.name, f.priority, f.title);
                if let crate::loops::feature::FeatureSource::GitHubIssue { issue_number, .. } = f.source {
                    println!("       (from issue #{issue_number})");
                }
            }
        }

        println!();
        println!("━━━ Worktrees ({}) ━━━", worktrees.len());
        if worktrees.is_empty() {
            println!("  (no active worktrees)");
        } else {
            for wt in &worktrees {
                let icon = match wt.status {
                    crate::loops::worktree::WorktreeStatus::Active => "◷",
                    crate::loops::worktree::WorktreeStatus::Merged => "✓",
                    crate::loops::worktree::WorktreeStatus::Abandoned => "✕",
                };
                println!("  {icon} {} ({})", wt.name, wt.branch);
                println!("       path: {}", wt.path.display());
            }
        }
    }

    Ok(())
}

fn run_maintain(args: LoopMaintainArgs, json: bool) -> anyhow::Result<()> {
    let root = resolve_path(args.path);
    let engine = LoopEngine::new(&root, load_loop_config(&root));

    if json {
        engine.run_maintenance()?;
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"status": "ok"}))?);
    } else {
        eprintln!("━━━ KodeHold Maintenance ━━━");
        engine.run_maintenance()?;
        println!("  ✓ maintenance complete");
    }

    Ok(())
}

fn run_prune(args: LoopPruneArgs, json: bool) -> anyhow::Result<()> {
    let root = resolve_path(args.path);
    let engine = LoopEngine::new(&root, load_loop_config(&root));

    if json {
        engine.prune_abandoned()?;
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"status": "ok"}))?);
    } else {
        eprintln!("━━━ Pruning Abandoned Worktrees ━━━");
        engine.prune_abandoned()?;
        println!("  ✓ prune complete");
    }

    Ok(())
}
