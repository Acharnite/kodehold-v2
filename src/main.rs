use clap::Parser;

mod cli;
mod cmd;
mod config;
mod state;
mod workspace;

fn main() {
    let cli = cli::Cli::parse();
    let result = match cli.command {
        cli::Command::Init(cmd) => cmd::init::run(cmd),
        cli::Command::Verify(cmd) => cmd::verify::run(cmd),
        cli::Command::Ship(cmd) => cmd::ship::run(cmd),
        cli::Command::State(cmd) => cmd::state::run(cmd),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
