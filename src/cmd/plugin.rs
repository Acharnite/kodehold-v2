use crate::cli::{
    PluginArgs, PluginCommand, PluginInitArgs, PluginListArgs, PluginRunArgs,
};
use crate::plugin;
use crate::workspace::Workspace;

pub fn run(args: PluginArgs) -> anyhow::Result<()> {
    match args.command {
        PluginCommand::List(a) => list(a),
        PluginCommand::Run(a) => run_hook(a),
        PluginCommand::Init(a) => init_hook(a),
    }
}

fn list(args: PluginListArgs) -> anyhow::Result<()> {
    let ws = Workspace::open(args.path.as_deref())?;
    let hooks = plugin::HookConfig::from_config(&ws.config);
    let available = hooks.list_available(&ws.root);

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "hooks": available,
                "all_configured": {
                    "pre-generate": hooks.pre_generate,
                    "post-verify": hooks.post_verify,
                    "on-ship": hooks.on_ship,
                }
            }))?
        );
    } else {
        println!("━━━ Hooks ({}) ━━━", available.len());
        for h in &available {
            println!("  ✓ {h}");
        }
        println!();
        if let Some(pg) = &hooks.pre_generate {
            println!("  pre-generate: {pg}");
        }
        if let Some(pv) = &hooks.post_verify {
            println!("  post-verify: {pv}");
        }
        if let Some(os) = &hooks.on_ship {
            println!("  on-ship: {os}");
        }
    }
    Ok(())
}

fn run_hook(args: PluginRunArgs) -> anyhow::Result<()> {
    let ws = Workspace::open(args.path.as_deref())?;
    let result = plugin::run_hook(&ws.root, &args.hook);

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "hook": result.hook,
                "status": format!("{:?}", result.status),
                "message": result.message,
            }))?
        );
    } else {
        let icon = match result.status {
            plugin::HookStatus::Ok => "✓",
            plugin::HookStatus::Blocked => "✗",
            plugin::HookStatus::Skipped => "–",
        };
        println!("  {icon} {}: {}", result.hook, result.message);
    }
    Ok(())
}

fn init_hook(args: PluginInitArgs) -> anyhow::Result<()> {
    let ws = Workspace::open(args.path.as_deref())?;
    let lang = args.language.as_deref().unwrap_or("bash");

    let path = plugin::scaffold_hook(&ws.root, &args.name, lang)?;

    // Update kodehold.yaml to register the hook
    let mut config = ws.config;
    let hook_path = format!("hooks/{}", args.name);
    match args.name.as_str() {
        "pre-generate" => config.hooks.pre_generate = Some(hook_path),
        "post-verify" => config.hooks.post_verify = Some(hook_path),
        "on-ship" => config.hooks.on_ship = Some(hook_path),
        _ => {}
    }
    config.save(&ws.root)?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "ok",
                "path": path,
                "language": lang,
            }))?
        );
    } else {
        println!("  ✓ Created hook: {path}");
    }
    Ok(())
}
