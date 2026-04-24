use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Spawn a command and profile its resource usage until it exits")]
pub struct RunArgs {
    pub command: Vec<String>,

    #[arg(short, long, value_name = "ms", default_value = "100")]
    pub interval: u64,

    #[arg(short, long, value_name = "time")]
    pub timeout: Option<String>,

    #[arg(long, value_name = "path")]
    pub json: Option<String>,

    #[arg(long)]
    pub silent: bool,
}
