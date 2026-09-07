use crate::cli::pid::PidArgs;
use crate::commands::sample_stats::SampleStats;
use serde::Serialize;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Serialize)]
struct Sample {
    timestamp_ms: u64,
    cpu_percent: f32,
    memory_bytes: u64,
}

#[derive(Serialize)]
struct PidResult {
    pid: u32,
    process_name: String,
    duration_ms: u64,
    peak_cpu: f32,
    avg_cpu: f32,
    peak_memory_bytes: u64,
    avg_memory_bytes: u64,
    samples: Vec<Sample>,
}

pub fn execute(args: PidArgs) {
    let id = crate::store::now_ms().to_string();
    let pid = Pid::from_u32(args.pid);
    let duration_limit = args.duration.as_deref().map(parse_duration);
    let interval = Duration::from_millis(args.interval);

    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);

    let process_name = match sys.process(pid) {
        Some(p) => p.name().to_string_lossy().into_owned(),
        None => {
            eprintln!("error: no process found with pid {}", args.pid);
            std::process::exit(1);
        }
    };

    let mut samples: Vec<Sample> = Vec::new();
    let mut stats = SampleStats::default();
    let start = Instant::now();

    loop {
        thread::sleep(interval);

        let elapsed = start.elapsed();

        if let Some(limit) = duration_limit
            && elapsed >= limit
        {
            break;
        }

        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

        match sys.process(pid) {
            Some(proc) => {
                let sample = Sample {
                    timestamp_ms: elapsed.as_millis() as u64,
                    cpu_percent: proc.cpu_usage(),
                    memory_bytes: proc.memory(),
                };
                stats.record(sample.cpu_percent, sample.memory_bytes);
                samples.push(sample);
            }
            None => break,
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    let result = PidResult {
        pid: args.pid,
        process_name,
        duration_ms,
        peak_cpu: stats.peak_cpu(),
        avg_cpu: stats.avg_cpu(),
        peak_memory_bytes: stats.peak_memory_bytes(),
        avg_memory_bytes: stats.avg_memory_bytes(),
        samples,
    };

    print_summary(&result);

    if let Some(path) = &args.json {
        let json = serde_json::to_string_pretty(&result).unwrap();
        std::fs::write(path, json).unwrap_or_else(|e| {
            eprintln!("error: failed to write to '{path}': {e}");
        });
        println!("results saved to {path}");
    }

    let timestamp = crate::store::now_secs();
    crate::store::save_run(&crate::store::RunDetail {
        id: id.clone(),
        timestamp,
        source: "pid".to_string(),
        command: None,
        pid: Some(result.pid),
        process_name: Some(result.process_name.clone()),
        exit_code: None,
        duration_ms: result.duration_ms,
        peak_cpu: result.peak_cpu,
        avg_cpu: result.avg_cpu,
        peak_memory_bytes: result.peak_memory_bytes,
        avg_memory_bytes: result.avg_memory_bytes,
        samples: result
            .samples
            .iter()
            .map(|s| crate::store::DetailSample {
                timestamp_ms: s.timestamp_ms,
                cpu_percent: s.cpu_percent,
                memory_bytes: s.memory_bytes,
            })
            .collect(),
    });
    crate::store::append(crate::store::HistoryEntry {
        id,
        timestamp,
        source: "pid".to_string(),
        command: None,
        pid: Some(result.pid),
        process_name: Some(result.process_name.clone()),
        duration_ms: result.duration_ms,
        peak_cpu: result.peak_cpu,
        avg_cpu: result.avg_cpu,
        peak_memory_bytes: result.peak_memory_bytes,
        avg_memory_bytes: result.avg_memory_bytes,
    });
}

fn print_summary(r: &PidResult) {
    println!("pid:         {}", r.pid);
    println!("process:     {}", r.process_name);
    println!("duration:    {:.3}s", r.duration_ms as f64 / 1000.0);
    println!("peak cpu:    {:.1}%", r.peak_cpu);
    println!("avg cpu:     {:.1}%", r.avg_cpu);
    println!("peak memory: {}", fmt_bytes(r.peak_memory_bytes));
    println!("avg memory:  {}", fmt_bytes(r.avg_memory_bytes));
}

fn parse_duration(s: &str) -> Duration {
    if let Some(n) = s.strip_suffix("ms") {
        Duration::from_millis(n.trim().parse().unwrap_or(0))
    } else if let Some(n) = s.strip_suffix('s') {
        Duration::from_secs(n.trim().parse().unwrap_or(0))
    } else if let Some(n) = s.strip_suffix('m') {
        Duration::from_secs(n.trim().parse::<u64>().unwrap_or(0) * 60)
    } else {
        Duration::from_secs(s.trim().parse().unwrap_or(0))
    }
}

fn fmt_bytes(b: u64) -> String {
    match b {
        b if b >= 1 << 30 => format!("{:.1} GB", b as f64 / (1 << 30) as f64),
        b if b >= 1 << 20 => format!("{:.1} MB", b as f64 / (1 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.1} KB", b as f64 / (1 << 10) as f64),
        b => format!("{b} B"),
    }
}
