use crate::cli::live::LiveArgs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessesToUpdate, System};

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

fn terminal_rows() -> Option<u16> {
    Command::new("stty")
        .arg("size")
        .stdin(Stdio::inherit())
        .output()
        .ok()
        .and_then(|out| {
            let s = String::from_utf8_lossy(&out.stdout);
            s.split_whitespace().next()?.parse().ok()
        })
}

fn fmt_bytes(b: u64) -> String {
    match b {
        b if b >= 1 << 30 => format!("{:.1} GB", b as f64 / (1 << 30) as f64),
        b if b >= 1 << 20 => format!("{:.1} MB", b as f64 / (1 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.1} KB", b as f64 / (1 << 10) as f64),
        b => format!("{b} B"),
    }
}

struct TtyMode {
    rows: u16,
}

impl TtyMode {
    fn enter(&self) {
        print!(
            "\x1b[?1049h\x1b[H\x1b[J\x1b[?25l\x1b[2;{}r\x1b[2;1H",
            self.rows
        );
        io::stdout().flush().ok();
        self.update(0.0, 0, 0.0);
    }

    fn update(&self, cpu: f32, mem: u64, elapsed_s: f64) {
        print!(
            "\x1b7\x1b[1;1H\x1b[2K  \x1b[36m[live]\x1b[0m  \
             cpu: \x1b[33m{:.1}%\x1b[0m  \
             mem: \x1b[33m{}\x1b[0m  \
             time: \x1b[33m{:.1}s\x1b[0m\x1b8",
            cpu,
            fmt_bytes(mem),
            elapsed_s
        );
        io::stdout().flush().ok();
    }

    fn leave(&self) {
        print!("\x1b[r\x1b[?25h\x1b[?1049l");
        io::stdout().flush().ok();
    }
}

pub fn execute(args: LiveArgs) {
    let id = crate::store::now_ms().to_string();
    let (program, cmd_args) = spawn_args(&args.command);
    let cmd_str = args.command.join(" ");

    let tty = terminal_rows().map(|rows| TtyMode { rows });

    if let Some(ref t) = tty {
        t.enter();
    }

    let mut child = Command::new(&program)
        .args(&cmd_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| {
            if let Some(ref t) = tty {
                t.leave();
            }
            eprintln!("error: failed to spawn '{program}': {e}");
            std::process::exit(1);
        });

    let pid = Pid::from_u32(child.id());
    let interval = Duration::from_millis(500);

    let mut sys = System::new();
    let start = Instant::now();

    let mut cpu_samples: Vec<f32> = Vec::new();
    let mut mem_samples: Vec<u64> = Vec::new();
    let mut detail_samples: Vec<crate::store::DetailSample> = Vec::new();

    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);

    loop {
        thread::sleep(interval);

        let elapsed = start.elapsed();
        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);

        if let Some(proc) = sys.process(pid) {
            let cpu = proc.cpu_usage();
            let mem = proc.memory();
            cpu_samples.push(cpu);
            mem_samples.push(mem);
            detail_samples.push(crate::store::DetailSample {
                timestamp_ms: elapsed.as_millis() as u64,
                cpu_percent: cpu,
                memory_bytes: mem,
            });
            if let Some(ref t) = tty {
                t.update(cpu, mem, elapsed.as_secs_f64());
            } else {
                println!(
                    "[t={:.1}s]  cpu: {:.1}%  mem: {}",
                    elapsed.as_secs_f64(),
                    cpu,
                    fmt_bytes(mem)
                );
            }
        }

        match child.try_wait() {
            Ok(Some(_)) | Err(_) => break,
            Ok(None) => {}
        }
    }

    let exit_code = child.wait().ok().and_then(|s| s.code());
    let duration_ms = start.elapsed().as_millis() as u64;

    if let Some(ref t) = tty {
        t.leave();
    }

    let peak_cpu = cpu_samples.iter().cloned().fold(0.0f32, f32::max);
    let avg_cpu = if cpu_samples.is_empty() {
        0.0
    } else {
        cpu_samples.iter().sum::<f32>() / cpu_samples.len() as f32
    };
    let peak_mem = mem_samples.iter().cloned().max().unwrap_or(0);
    let avg_mem = if mem_samples.is_empty() {
        0
    } else {
        mem_samples.iter().sum::<u64>() / mem_samples.len() as u64
    };

    println!("command:     {cmd_str}");
    println!("duration:    {:.3}s", duration_ms as f64 / 1000.0);
    println!(
        "exit code:   {}",
        exit_code.map_or("-".to_string(), |c| c.to_string())
    );
    println!("peak cpu:    {:.1}%", peak_cpu);
    println!("avg cpu:     {:.1}%", avg_cpu);
    println!("peak memory: {}", fmt_bytes(peak_mem));
    println!("avg memory:  {}", fmt_bytes(avg_mem));

    let timestamp = crate::store::now_secs();
    crate::store::save_run(&crate::store::RunDetail {
        id: id.clone(),
        timestamp,
        source: "live".to_string(),
        command: Some(cmd_str.clone()),
        pid: None,
        process_name: None,
        exit_code,
        duration_ms,
        peak_cpu,
        avg_cpu,
        peak_memory_bytes: peak_mem,
        avg_memory_bytes: avg_mem,
        samples: detail_samples,
    });
    crate::store::append(crate::store::HistoryEntry {
        id,
        timestamp,
        source: "live".to_string(),
        command: Some(cmd_str),
        pid: None,
        process_name: None,
        duration_ms,
        peak_cpu,
        avg_cpu,
        peak_memory_bytes: peak_mem,
        avg_memory_bytes: avg_mem,
    });
}
