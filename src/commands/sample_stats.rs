#[derive(Default)]
pub(crate) struct SampleStats {
    count: usize,
    peak_cpu: f32,
    sum_cpu: f32,
    peak_memory_bytes: u64,
    sum_memory_bytes: u64,
}

impl SampleStats {
    pub(crate) fn record(&mut self, cpu_percent: f32, memory_bytes: u64) {
        self.count += 1;
        self.peak_cpu = self.peak_cpu.max(cpu_percent);
        self.sum_cpu += cpu_percent;
        self.peak_memory_bytes = self.peak_memory_bytes.max(memory_bytes);
        self.sum_memory_bytes += memory_bytes;
    }

    pub(crate) fn peak_cpu(&self) -> f32 {
        self.peak_cpu
    }

    pub(crate) fn avg_cpu(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_cpu / self.count as f32
        }
    }

    pub(crate) fn peak_memory_bytes(&self) -> u64 {
        self.peak_memory_bytes
    }

    pub(crate) fn avg_memory_bytes(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.sum_memory_bytes / self.count as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SampleStats;

    #[test]
    fn empty_stats_match_existing_defaults() {
        let stats = SampleStats::default();

        assert_eq!(stats.peak_cpu(), 0.0);
        assert_eq!(stats.avg_cpu(), 0.0);
        assert_eq!(stats.peak_memory_bytes(), 0);
        assert_eq!(stats.avg_memory_bytes(), 0);
    }

    #[test]
    fn single_sample_stats_match_sample_values() {
        let mut stats = SampleStats::default();

        stats.record(12.5, 2048);

        assert_eq!(stats.peak_cpu(), 12.5);
        assert_eq!(stats.avg_cpu(), 12.5);
        assert_eq!(stats.peak_memory_bytes(), 2048);
        assert_eq!(stats.avg_memory_bytes(), 2048);
    }

    #[test]
    fn multiple_sample_stats_match_rescan_calculation() {
        let mut stats = SampleStats::default();

        stats.record(10.0, 1024);
        stats.record(30.0, 4096);
        stats.record(20.0, 2048);

        assert_eq!(stats.peak_cpu(), 30.0);
        assert_eq!(stats.avg_cpu(), 20.0);
        assert_eq!(stats.peak_memory_bytes(), 4096);
        assert_eq!(stats.avg_memory_bytes(), 2389);
    }
}
