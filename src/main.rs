mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run(args) => commands::run::execute(args),
        _ => eprintln!("not yet implemented"),
    }
}
