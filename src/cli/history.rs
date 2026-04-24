use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "List past profiling runs stored locally")]
pub struct HistoryArgs {
    #[arg(short = 'n', long, value_name = "N")]
    pub limit: Option<usize>,

    #[arg(long, value_name = "keyword")]
    pub filter: Option<String>,

    #[arg(long)]
    pub json: bool,
}
