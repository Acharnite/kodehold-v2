use crate::cli::InitArgs;
use crate::workspace::Workspace;

pub fn run(args: InitArgs) -> anyhow::Result<()> {
    let ws = Workspace::init(&args.name, args.language.as_deref(), &std::env::current_dir()?)?;

    if args.json {
        let output = serde_json::json!({
            "status": "ok",
            "name": args.name,
            "language": ws.config.language,
            "path": ws.root.display().to_string(),
            "state": "INIT",
            "tag": "v2-init",
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("✓ Created project '{}' in {}", args.name, ws.root.display());
        println!("  Language: {}", ws.config.language);
        println!("  State: INIT (tag: v2-init)");
        println!();
        println!("  Next steps:");
        println!("    cd {}", args.name);
        println!("    kodehold verify");
    }

    Ok(())
}
