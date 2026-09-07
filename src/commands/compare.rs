use crate::cli::compare::{CompareArgs, MetricType};
use crate::store;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct RunRecord {
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    pid: Option<u32>,
    #[serde(default)]
    process_name: Option<String>,
    duration_ms: u64,
    peak_cpu: f32,
    avg_cpu: f32,
    peak_memory_bytes: u64,
    avg_memory_bytes: u64,
}

impl From<store::RunDetail> for RunRecord {
    fn from(detail: store::RunDetail) -> Self {
        Self {
            command: detail.command,
            pid: detail.pid,
            process_name: detail.process_name,
            duration_ms: detail.duration_ms,
            peak_cpu: detail.peak_cpu,
            avg_cpu: detail.avg_cpu,
            peak_memory_bytes: detail.peak_memory_bytes,
            avg_memory_bytes: detail.avg_memory_bytes,
        }
    }
}

fn load(input: &str) -> RunRecord {
    if Path::new(input).exists() {
        return load_path(input);
    }

    store::load_run(input).map(Into::into).unwrap_or_else(|| {
        eprintln!("error: failed to find run file or saved run id '{input}'");
        std::process::exit(1);
    })
}

fn load_path(path: &str) -> RunRecord {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: failed to read run file '{path}': {e}");
        std::process::exit(1);
    });
    serde_json::from_str(&content).unwrap_or_else(|e| {
        eprintln!("error: failed to parse run file '{path}': {e}");
        std::process::exit(1);
    })
}

fn record_label(r: &RunRecord) -> String {
    if let Some(cmd) = &r.command {
        cmd.clone()
    } else if let (Some(name), Some(pid)) = (&r.process_name, r.pid) {
        format!("{name} (pid: {pid})")
    } else {
        "unknown".to_string()
    }
}

fn fmt_bytes(b: u64) -> String {
    match b {
        b if b >= 1 << 30 => format!("{:.1} GB", b as f64 / (1u64 << 30) as f64),
        b if b >= 1 << 20 => format!("{:.1} MB", b as f64 / (1u64 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.1} KB", b as f64 / (1u64 << 10) as f64),
        b => format!("{b} B"),
    }
}

fn pct(a: f64, b: f64) -> f64 {
    if a == 0.0 { 0.0 } else { (b - a) / a * 100.0 }
}

fn delta_sec(a_ms: u64, b_ms: u64) -> String {
    let a = a_ms as f64;
    let b = b_ms as f64;
    let ds = (b - a) / 1000.0;
    let p = pct(a, b);
    let s = if ds >= 0.0 { "+" } else { "" };
    let ps = if p >= 0.0 { "+" } else { "" };
    format!("{s}{ds:.3}s ({ps}{p:.1}%)")
}

fn delta_pp(a: f32, b: f32) -> String {
    let a = a as f64;
    let b = b as f64;
    let d = b - a;
    let p = pct(a, b);
    let s = if d >= 0.0 { "+" } else { "" };
    let ps = if p >= 0.0 { "+" } else { "" };
    format!("{s}{d:.1}pp ({ps}{p:.1}%)")
}

fn delta_bytes(a: u64, b: u64) -> String {
    if a == b {
        return "±0 B (0.0%)".to_string();
    }
    let d = b as i64 - a as i64;
    let p = pct(a as f64, b as f64);
    let abs = d.unsigned_abs();
    let ps = if p >= 0.0 { "+" } else { "" };
    if d < 0 {
        format!("-{} ({ps}{p:.1}%)", fmt_bytes(abs))
    } else {
        format!("+{} ({ps}{p:.1}%)", fmt_bytes(abs))
    }
}

const W_LABEL: usize = 13;
const W_VAL: usize = 18;
const W_DELTA: usize = 22;

fn print_row(label: &str, v1: &str, v2: &str, delta: &str) {
    println!(
        "{:<W_LABEL$}  {:<W_VAL$}  {:<W_VAL$}  {}",
        label, v1, v2, delta
    );
}

fn print_sep() {
    println!(
        "{:─<W_LABEL$}  {:─<W_VAL$}  {:─<W_VAL$}  {:─<W_DELTA$}",
        "", "", "", ""
    );
}

pub fn execute(args: CompareArgs) {
    let r1 = load(&args.run1);
    let r2 = load(&args.run2);

    let name1 = Path::new(&args.run1)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| args.run1.clone());
    let name2 = Path::new(&args.run2)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| args.run2.clone());

    let show_all = args.metric.is_none();
    let show_time = show_all || matches!(args.metric, Some(MetricType::Time));
    let show_cpu = show_all || matches!(args.metric, Some(MetricType::Cpu));
    let show_mem = show_all || matches!(args.metric, Some(MetricType::Mem));

    print_row("metric", &name1, &name2, "delta");
    print_sep();

    if show_all {
        print_row("source", &record_label(&r1), &record_label(&r2), "");
    }

    if show_time {
        let v1 = format!("{:.3}s", r1.duration_ms as f64 / 1000.0);
        let v2 = format!("{:.3}s", r2.duration_ms as f64 / 1000.0);
        print_row(
            "duration",
            &v1,
            &v2,
            &delta_sec(r1.duration_ms, r2.duration_ms),
        );
    }

    if show_cpu {
        let v1 = format!("{:.1}%", r1.peak_cpu);
        let v2 = format!("{:.1}%", r2.peak_cpu);
        print_row("peak cpu", &v1, &v2, &delta_pp(r1.peak_cpu, r2.peak_cpu));

        let v1 = format!("{:.1}%", r1.avg_cpu);
        let v2 = format!("{:.1}%", r2.avg_cpu);
        print_row("avg cpu", &v1, &v2, &delta_pp(r1.avg_cpu, r2.avg_cpu));
    }

    if show_mem {
        let v1 = fmt_bytes(r1.peak_memory_bytes);
        let v2 = fmt_bytes(r2.peak_memory_bytes);
        print_row(
            "peak memory",
            &v1,
            &v2,
            &delta_bytes(r1.peak_memory_bytes, r2.peak_memory_bytes),
        );

        let v1 = fmt_bytes(r1.avg_memory_bytes);
        let v2 = fmt_bytes(r2.avg_memory_bytes);
        print_row(
            "avg memory",
            &v1,
            &v2,
            &delta_bytes(r1.avg_memory_bytes, r2.avg_memory_bytes),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_detail_converts_to_compare_record() {
        let detail = store::RunDetail {
            id: "run-1".to_string(),
            timestamp: 1,
            source: "run".to_string(),
            command: Some("cargo test".to_string()),
            pid: None,
            process_name: None,
            exit_code: Some(0),
            duration_ms: 1234,
            peak_cpu: 42.0,
            avg_cpu: 21.0,
            peak_memory_bytes: 4096,
            avg_memory_bytes: 2048,
            samples: vec![],
        };

        let record = RunRecord::from(detail);

        assert_eq!(record.command.as_deref(), Some("cargo test"));
        assert_eq!(record.pid, None);
        assert_eq!(record.process_name, None);
        assert_eq!(record.duration_ms, 1234);
        assert_eq!(record.peak_cpu, 42.0);
        assert_eq!(record.avg_cpu, 21.0);
        assert_eq!(record.peak_memory_bytes, 4096);
        assert_eq!(record.avg_memory_bytes, 2048);
    }
}
