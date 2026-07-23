use crate::cli::StateArgs;
use crate::state;

pub fn run(args: StateArgs) -> anyhow::Result<()> {
    let path = args
        .path
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or(std::env::current_dir().unwrap_or_default());

    let snapshot = state::derive_state(&path)?;

    if args.json {
        let history: Vec<serde_json::Value> = snapshot
            .history
            .iter()
            .map(|(s, msg)| {
                serde_json::json!({
                    "state": s.to_string(),
                    "tag": s.tag_name(),
                    "message": msg,
                })
            })
            .collect();
        let output = serde_json::json!({
            "current": snapshot.current.to_string(),
            "current_tag": snapshot.current.tag_name(),
            "commit_count": snapshot.commit_count,
            "history": history,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("  State: {}", snapshot.current);
        println!("  Tag:   {}", snapshot.current.tag_name());
        println!("  Commits: {}", snapshot.commit_count);
        println!();
        if snapshot.history.is_empty() {
            println!("  No lifecycle tags found (project may not be initialized)");
        } else {
            println!("  Lifecycle:");
            for (s, msg) in &snapshot.history {
                let marker = if *s == snapshot.current { "→" } else { " " };
                println!("    {marker} {} ({})", s.tag_name(), s);
                if !msg.is_empty() {
                    println!("       {msg}");
                }
            }
        }
    }

    Ok(())
}
