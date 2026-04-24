pub mod compare;
pub mod history;
pub mod inspect;
pub mod live;
pub mod pid;
pub mod run;

use clap::{Parser, Subcommand};
use compare::CompareArgs;
use history::HistoryArgs;
use inspect::InspectArgs;
use live::LiveArgs;
use pid::PidArgs;
use run::RunArgs;

#[derive(Parser, Debug)]
#[command(name = "cavira", version, about = "Cavira process — execution analysis tool")]
pub struct Cli {
    #[arg(long, global = true)]
    pub no_color: bool,

    #[arg(long, global = true)]
    pub quiet: bool,

    #[arg(long, global = true, value_name = "path")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run(RunArgs),
    Pid(PidArgs),
    Compare(CompareArgs),
    History(HistoryArgs),
    Inspect(InspectArgs),
    Live(LiveArgs),
}
