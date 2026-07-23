use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kodehold", version, about = "KodeHold v2.0 — code specification and generation tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new project with git-native state
    Init(InitArgs),
    /// Run project verification (tests + lint)
    Verify(VerifyArgs),
    /// Bump version, tag, push
    Ship(ShipArgs),
    /// Show workspace state from git tags
    State(StateArgs),
    /// Run the full compilation pipeline
    Pipeline(PipelineArgs),
    /// Manage the pattern database
    Pattern(PatternArgs),
    /// Start the event daemon (webhooks + file watcher)
    Listen(ListenArgs),
}

#[derive(clap::Args)]
pub struct InitArgs {
    pub name: String,
    #[arg(short, long)]
    pub language: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct VerifyArgs {
    #[arg(short, long)]
    pub path: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct ShipArgs {
    #[arg(short, long)]
    pub bump: Option<String>,
    #[arg(short, long)]
    pub path: Option<String>,
    #[arg(long)]
    pub no_push: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct StateArgs {
    #[arg(short, long)]
    pub path: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct PipelineArgs {
    /// Project path (defaults to current directory)
    #[arg(short = 'P', long)]
    pub path: Option<String>,
    /// Run only specific phases (comma-separated: parse,analyze,generate,verify,optimize)
    #[arg(short = 'f', long = "phases")]
    pub phases: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct PatternArgs {
    #[command(subcommand)]
    pub command: PatternCommand,
}

#[derive(Subcommand)]
pub enum PatternCommand {
    /// List all patterns in the database
    List(PatternListArgs),
    /// Search patterns by query
    Search(PatternSearchArgs),
    /// Add a pattern from a JSON file
    Add(PatternAddArgs),
}

#[derive(clap::Args)]
pub struct PatternListArgs {
    /// Language filter
    #[arg(short, long)]
    pub language: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct PatternSearchArgs {
    /// Search query (matches name, signature, tags)
    pub query: String,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct PatternAddArgs {
    /// Path to JSON pattern file
    pub path: String,
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct ListenArgs {
    /// Project path (defaults to current directory)
    #[arg(short = 'P', long)]
    pub path: Option<String>,
    /// HTTP port for webhook server
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
    /// GitHub webhook secret (for signature verification)
    #[arg(short = 's', long)]
    pub webhook_secret: Option<String>,
    /// Paths to watch for file changes
    #[arg(short = 'w', long)]
    pub watch: Vec<String>,
    /// Disable file watcher
    #[arg(long)]
    pub no_watch: bool,
}
