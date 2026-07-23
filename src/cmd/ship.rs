use crate::cli::ShipArgs;
use crate::workspace::Workspace;

pub fn run(args: ShipArgs) -> anyhow::Result<()> {
    let ws = Workspace::open(args.path.as_deref())?;

    // Run on-ship hook before shipping
    let hooks = crate::plugin::HookConfig::from_config(&ws.config);
    if let Some(ref path) = hooks.on_ship {
        let result = crate::plugin::run_hook(&ws.root, path);
        match result.status {
            crate::plugin::HookStatus::Blocked => {
                anyhow::bail!("on-ship hook blocked: {} ({})", result.hook, result.message);
            }
            _ => {
                if !args.json {
                    let icon = if result.status == crate::plugin::HookStatus::Ok { "✓" } else { "–" };
                    println!("  {icon} hook {}: {}", result.hook, result.message);
                }
            }
        }
    }

    let result = ws.ship(args.bump.as_deref())?;

    if args.json {
        let output = serde_json::json!({
            "status": "ok",
            "old_version": result.old_version,
            "new_version": result.new_version,
            "tag": result.tag_name,
            "changelog": result.changelog,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("  ✓ Version bumped: {} → {}", result.old_version, result.new_version);
        println!("  ✓ Tag created: {}", result.tag_name);
        println!();
        println!("  Changelog:");
        for line in result.changelog.lines() {
            println!("    {line}");
        }
        println!();
        println!("  Next step: kodehold ship --push to push to remote");
        println!("  (or: git push && git push --tags)");
    }

    // Push if not skipped
    if !args.no_push {
        if args.json {
            // In JSON mode, push is handled separately
        } else {
            push_to_remote(&ws)?;
            println!("  ✓ Pushed to remote (commits + tags)");
        }
    }

    Ok(())
}

fn push_to_remote(ws: &Workspace) -> anyhow::Result<()> {
    let output = std::process::Command::new("git")
        .args(["-C", &ws.root.to_string_lossy(), "push"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run git push: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if no remote is configured
        if !stderr.contains("No remote configured") {
            eprintln!("  ⚠ git push output: {}", stderr.trim());
        }
    }

    // Also push tags
    let tag_output = std::process::Command::new("git")
        .args(["-C", &ws.root.to_string_lossy(), "push", "--tags"])
        .output()?;

    if !tag_output.status.success() {
        let stderr = String::from_utf8_lossy(&tag_output.stderr);
        if !stderr.contains("No remote configured") {
            eprintln!("  ⚠ git push --tags output: {}", stderr.trim());
        }
    }

    Ok(())
}
