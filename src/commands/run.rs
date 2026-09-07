use crate::cli::run::RunArgs;
use crate::commands::sample_stats::SampleStats;
use serde::Serialize;
use std::process::{Command, Stdio};
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
struct RunResult {
    command: String,
    exit_code: Option<i32>,
    duration_ms: u64,
    peak_cpu: f32,
    avg_cpu: f32,
    peak_memory_bytes: u64,
    avg_memory_bytes: u64,
    samples: Vec<Sample>,
}

const SHELL_OPS: &[&str] = &["&&", "||", ";", "|", ">", ">>", "<"];

fn spawn_args(tokens: &[String]) -> (String, Vec<String>) {
    if tokens.is_empty() {
        eprintln!("error: no command given");
        std::process::exit(1);
    }
    if tokens.iter().any(|t| SHELL_OPS.contains(&t.as_str())) {
        ("sh".to_string(), vec!["-c".to_string(), tokens.join(" ")])
    } else {
        (tokens[0].clone(), tokens[1..].to_vec())
    }
}

pub fn execute(args: RunArgs) {
    let id = crate::store::now_ms().to_string();
    let (program, cmd_args) = spawn_args(&args.command);

    let mut child = Command::new(&program)
        .args(&cmd_args)
        .stdout(if args.silent {
            Stdio::null()
        } else {
            Stdio::inherit()
        })
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("error: failed to spawn '{program}': {e}");
            std::process::exit(1);
        });

    let pid = Pid::from_u32(child.id());
    let timeout_dur = args.timeout.as_deref().map(parse_duration);
    let interval = Duration::from_millis(args.interval);

    let mut sys = System::new();
    let mut samples: Vec<Sample> = Vec::new();
    let mut stats = SampleStats::default();
    let start = Instant::now();

    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);

    loop {
        thread::sleep(interval);

        let elapsed = start.elapsed();

        if let Some(limit) = timeout_dur
            && elapsed >= limit
        {
            let _ = child.kill();
            break;
        }

        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);

        if let Some(proc) = sys.process(pid) {
            let sample = Sample {
                timestamp_ms: elapsed.as_millis() as u64,
                cpu_percent: proc.cpu_usage(),
                memory_bytes: proc.memory(),
            };
            stats.record(sample.cpu_percent, sample.memory_bytes);
            samples.push(sample);
        }

        match child.try_wait() {
            Ok(Some(_)) | Err(_) => break,
            Ok(None) => {}
        }
    }

    let exit_code = child.wait().ok().and_then(|s| s.code());
    let duration_ms = start.elapsed().as_millis() as u64;

    let result = RunResult {
        command: args.command.join(" "),
        exit_code,
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
        source: "run".to_string(),
        command: Some(result.command.clone()),
        pid: None,
        process_name: None,
        exit_code: result.exit_code,
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
        source: "run".to_string(),
        command: Some(result.command.clone()),
        pid: None,
        process_name: None,
        duration_ms: result.duration_ms,
        peak_cpu: result.peak_cpu,
        avg_cpu: result.avg_cpu,
        peak_memory_bytes: result.peak_memory_bytes,
        avg_memory_bytes: result.avg_memory_bytes,
    });
}

fn print_summary(r: &RunResult) {
    println!("command:     {}", r.command);
    println!("duration:    {:.3}s", r.duration_ms as f64 / 1000.0);
    println!(
        "exit code:   {}",
        r.exit_code.map_or("-".to_string(), |c| c.to_string())
    );
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
