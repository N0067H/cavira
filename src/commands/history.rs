use crate::cli::history::HistoryArgs;
use crate::store::{self, HistoryEntry};

pub fn execute(args: HistoryArgs) {
    let mut entries = store::load();

    if let Some(kw) = &args.filter {
        let kw = kw.to_lowercase();
        entries.retain(|e| {
            e.command
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(&kw)
                || e.process_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&kw)
        });
    }

    if let Some(n) = args.limit {
        let skip = entries.len().saturating_sub(n);
        entries = entries.into_iter().skip(skip).collect();
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&entries).unwrap());
        return;
    }

    if entries.is_empty() {
        println!("no history found");
        return;
    }

    print_table(&entries);
}

fn print_table(entries: &[HistoryEntry]) {
    const W_ID: usize = 13;
    const W_TS: usize = 19;
    const W_SRC: usize = 3;
    const W_CMD: usize = 28;
    const W_DUR: usize = 9;
    const W_CPU: usize = 8;
    const W_MEM: usize = 9;

    println!(
        "  {:<W_ID$}  {:<W_TS$}  {:<W_SRC$}  {:<W_CMD$}  {:<W_DUR$}  {:<W_CPU$}  peak mem",
        "id", "timestamp", "src", "command / process", "duration", "peak cpu"
    );
    println!(
        "  {:─<W_ID$}  {:─<W_TS$}  {:─<W_SRC$}  {:─<W_CMD$}  {:─<W_DUR$}  {:─<W_CPU$}  {:─<W_MEM$}",
        "", "", "", "", "", "", ""
    );

    for entry in entries {
        let ts = fmt_timestamp(entry.timestamp);
        let label = entry_label(entry);
        let cmd = truncate(&label, W_CMD);
        let dur = format!("{:.3}s", entry.duration_ms as f64 / 1000.0);
        let cpu = format!("{:.1}%", entry.peak_cpu);
        let mem = fmt_bytes(entry.peak_memory_bytes);

        println!(
            "  {:<W_ID$}  {:<W_TS$}  {:<W_SRC$}  {:<W_CMD$}  {:<W_DUR$}  {:<W_CPU$}  {}",
            entry.id, ts, entry.source, cmd, dur, cpu, mem
        );
    }
}

fn entry_label(e: &HistoryEntry) -> String {
    if let Some(cmd) = &e.command {
        return cmd.clone();
    }
    if let (Some(name), Some(pid)) = (&e.process_name, e.pid) {
        return format!("{name} (pid: {pid})");
    }
    "?".to_string()
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        chars[..max].iter().collect()
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

fn fmt_timestamp(secs: u64) -> String {
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = (secs / 86400) as i64;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02}")
}

// https://howardhinnant.github.io/date_algorithms.html
fn days_to_ymd(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 {
        z / 146097
    } else {
        (z - 146096) / 146097
    };
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}
