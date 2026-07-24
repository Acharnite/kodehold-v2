use crate::cli::ListenArgs;
use crate::events::{webhook, watcher, Dispatcher};

pub fn run(args: ListenArgs) -> anyhow::Result<()> {
    let dispatcher = Dispatcher::new(
        args.path
            .as_deref()
            .unwrap_or(
                &std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            ),
    );

    // Default watch paths
    let watch_paths: Vec<String> = if args.watch.is_empty() {
        vec!["src".into(), "tests".into()]
    } else {
        args.watch
    };

    // Start file watcher
    if !args.no_watch {
        eprintln!("  starting file watcher...");
        watcher::Watcher::start(&watch_paths, dispatcher.clone())?;
    } else {
        eprintln!("  file watcher disabled (--no-watch)");
    }

    // Start webhook server
    let port = args.port;
    let addr = format!("0.0.0.0:{port}");

    eprintln!("  starting webhook server on http://{addr}");

    let web_dispatcher = dispatcher.clone();
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let app = webhook::router(
            web_dispatcher,
            args.webhook_secret.clone(),
        );

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        eprintln!("  ✓ listening on http://{addr}");
        eprintln!("  endpoints:");
        eprintln!("    POST /hooks/github  — GitHub webhook (push, PR, issues)");
        eprintln!("    POST /hooks/loop    — Trigger a loop cycle");
        eprintln!("    GET  /health        — health check");
        eprintln!();
        eprintln!("  Press Ctrl+C to stop");

        axum::serve(listener, app).await.unwrap();
    });

    Ok(())
}
