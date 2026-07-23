use crate::cli::{PatternAddArgs, PatternListArgs, PatternSearchArgs, PatternCommand};
use crate::solver::pattern_db::PatternDatabase;

pub fn run(args: crate::cli::PatternArgs) -> anyhow::Result<()> {
    match args.command {
        PatternCommand::List(a) => list(a),
        PatternCommand::Search(a) => search(a),
        PatternCommand::Add(a) => add(a),
    }
}

fn list(args: PatternListArgs) -> anyhow::Result<()> {
    let db = PatternDatabase::open();
    let patterns = db.list(args.language.as_deref());

    if args.json {
        let output: Vec<serde_json::Value> = patterns.iter().map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "signature": p.signature,
                "language": p.language,
                "tags": p.tags,
                "usage_count": p.usage_count,
                "success_rate": p.success_rate,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("━━━ Patterns ({}) ━━━", patterns.len());
        for p in &patterns {
            println!("  {} ({})", p.name, p.signature);
            println!("    id: {} | lang: {} | used: {}x", p.id, p.language, p.usage_count);
            if !p.tags.is_empty() {
                println!("    tags: {}", p.tags.join(", "));
            }
            println!();
        }
    }
    Ok(())
}

fn search(args: PatternSearchArgs) -> anyhow::Result<()> {
    let db = PatternDatabase::open();
    let results = db.search(&args.query);

    if args.json {
        let output: Vec<serde_json::Value> = results.iter().map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "signature": p.signature,
                "language": p.language,
                "tags": p.tags,
                "usage_count": p.usage_count,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("━━━ Search results for '{query}' ({count}) ━━━",
            query = args.query, count = results.len());
        for p in &results {
            println!("  {:<20} {:<30} {}x", p.name, p.signature, p.usage_count);
        }
    }
    Ok(())
}

fn add(args: PatternAddArgs) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(&args.path)?;
    let pattern: crate::solver::pattern_db::Pattern = serde_json::from_str(&content)?;

    let mut db = PatternDatabase::open();
    db.add(pattern)?;

    if args.json {
        println!("{}", serde_json::json!({ "status": "ok", "count": db.count() }));
    } else {
        println!("✓ Pattern added. Total: {}", db.count());
    }
    Ok(())
}
