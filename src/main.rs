use clap::Parser;

mod cli;
mod cmd;
mod config;
mod dsl;
mod events;
mod pipeline;
mod plugin;
mod solver;
mod state;
mod util;
mod workspace;

fn main() {
    let cli = cli::Cli::parse();
    let result = match cli.command {
        cli::Command::Init(cmd) => cmd::init::run(cmd),
        cli::Command::Verify(cmd) => cmd::verify::run(cmd),
        cli::Command::Ship(cmd) => cmd::ship::run(cmd),
        cli::Command::State(cmd) => cmd::state::run(cmd),
        cli::Command::Pipeline(cmd) => cmd::pipeline::run(cmd),
        cli::Command::Pattern(cmd) => cmd::pattern::run(cmd),
        cli::Command::Listen(cmd) => cmd::listen::run(cmd),
        cli::Command::Plugin(cmd) => cmd::plugin::run(cmd),
        cli::Command::Spec(cmd) => cmd::spec::run(cmd),
        cli::Command::Migrate(cmd) => cmd::migrate::run_migrate(cmd),
        cli::Command::Bridge(cmd) => cmd::migrate::run_bridge(cmd),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
