use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Run a command with real-time resource usage overlay")]
pub struct LiveArgs {
    pub command: Vec<String>,
}
