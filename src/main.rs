mod cli;
mod commands;
mod store;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run(args) => commands::run::execute(args),
        Commands::Pid(args) => commands::pid::execute(args),
        Commands::Compare(args) => commands::compare::execute(args),
        Commands::History(args) => commands::history::execute(args),
        _ => eprintln!("not yet implemented"),
    }
}
