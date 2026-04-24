use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(about = "Compare two recorded runs side by side")]
pub struct CompareArgs {
    pub run1: String,
    pub run2: String,

    #[arg(long, value_name = "type")]
    pub metric: Option<MetricType>,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum MetricType {
    Cpu,
    Mem,
    Time,
}
