use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Attach to a running process by PID and profile its resource usage")]
pub struct PidArgs {
    pub pid: u32,

    #[arg(short, long, value_name = "ms", default_value = "100")]
    pub interval: u64,

    #[arg(long, value_name = "time")]
    pub duration: Option<String>,

    #[arg(long, value_name = "path")]
    pub json: Option<String>,
}
