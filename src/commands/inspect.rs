use crate::cli::inspect::InspectArgs;
use crate::store::{self, RunDetail};

pub fn execute(args: InspectArgs) {
    let detail = store::load_run(&args.run_id).unwrap_or_else(|| {
        eprintln!("error: no run found with id '{}'", args.run_id);
        std::process::exit(1);
    });

    if args.json {
        println!("{}", serde_json::to_string_pretty(&detail).unwrap());
        return;
    }

    print_detail(&detail);
}

fn print_detail(d: &RunDetail) {
    println!("id:           {}", d.id);
    println!("timestamp:    {}", fmt_timestamp(d.timestamp));
    println!("source:       {}", d.source);
    if let Some(cmd) = &d.command {
        println!("command:      {}", cmd);
    }
    if let Some(pid) = d.pid {
        println!("pid:          {}", pid);
    }
    if let Some(name) = &d.process_name {
        println!("process:      {}", name);
    }
    if let Some(code) = d.exit_code {
        println!("exit code:    {}", code);
    }
    println!("duration:     {:.3}s", d.duration_ms as f64 / 1000.0);
    println!("samples:      {}", d.samples.len());
    println!();
    println!("peak cpu:     {:.1}%", d.peak_cpu);
    println!("avg cpu:      {:.1}%", d.avg_cpu);
    println!("peak memory:  {}", fmt_bytes(d.peak_memory_bytes));
    println!("avg memory:   {}", fmt_bytes(d.avg_memory_bytes));

    if d.samples.is_empty() {
        return;
    }

    println!();
    println!(
        "cpu (%)    {}",
        sparkline(&d.samples, 64, |s| s.cpu_percent as f64)
    );
    println!(
        "memory     {}",
        sparkline(&d.samples, 64, |s| s.memory_bytes as f64)
    );
}

fn sparkline<T>(values: &[T], max_width: usize, value: impl Fn(&T) -> f64 + Copy) -> String {
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let width = values.len().min(max_width);
    if width == 0 {
        return String::new();
    }

    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for i in 0..width {
        let v = sparkline_value(values, width, i, value);
        min = min.min(v);
        max = max.max(v);
    }

    let range = max - min;
    (0..width)
        .map(|i| {
            let v = sparkline_value(values, width, i, value);
            let norm = if range == 0.0 { 0.0 } else { (v - min) / range };
            let idx = (norm * (BLOCKS.len() - 1) as f64).round() as usize;
            BLOCKS[idx.min(BLOCKS.len() - 1)]
        })
        .collect()
}

fn sparkline_value<T>(
    values: &[T],
    width: usize,
    index: usize,
    value: impl Fn(&T) -> f64 + Copy,
) -> f64 {
    if values.len() <= width {
        return value(&values[index]);
    }

    let start = index * values.len() / width;
    let end = (index + 1) * values.len() / width;
    values[start..end].iter().map(value).sum::<f64>() / (end - start) as f64
}

#[cfg(test)]
mod tests {
    use super::sparkline;

    #[test]
    fn sparkline_handles_empty_values() {
        let values: [f64; 0] = [];
        assert_eq!(sparkline(&values, 64, |v| *v), "");
    }

    #[test]
    fn sparkline_handles_constant_values() {
        let values = [5.0, 5.0, 5.0];
        assert_eq!(sparkline(&values, 64, |v| *v), "▁▁▁");
    }

    #[test]
    fn sparkline_downsamples_without_changing_shape() {
        let values = [0.0, 2.0, 4.0, 6.0];
        assert_eq!(sparkline(&values, 2, |v| *v), "▁█");
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
