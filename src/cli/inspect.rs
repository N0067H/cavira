use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Show full profiling data for a specific run")]
pub struct InspectArgs {
    pub run_id: String,
}
