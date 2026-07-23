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
}

#[derive(clap::Args)]
pub struct InitArgs {
    /// Project name
    pub name: String,
    /// Project language (auto-detect if not set)
    #[arg(short, long)]
    pub language: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct VerifyArgs {
    /// Project path (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct ShipArgs {
    /// Version bump type
    #[arg(short, long)]
    pub bump: Option<String>,
    /// Project path (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<String>,
    /// Skip push
    #[arg(long)]
    pub no_push: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct StateArgs {
    /// Project path (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}
