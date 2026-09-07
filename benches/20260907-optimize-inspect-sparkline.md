# Optimize Inspect Sparkline

Date: 2026-09-07

Environment:
- OS: Linux 6.6.87.2-microsoft-standard-WSL2 x86_64
- rustc: 1.95.0
- cargo: 1.95.0
- profile: `cargo build --release`

Baseline:
- before: `3e4089e`
- after: `c1d94c3`

Problem:
- `inspect` built full `Vec<f64>` copies for CPU and memory before rendering sparklines.
- It then downsampled those full arrays into another intermediate Vec.
- The final sparkline is fixed-width, so allocating full per-metric arrays was unnecessary for large run detail files.

Change:
- Sparkline rendering now reads values directly from the sample slice through an accessor.
- Downsampling is calculated per output column without materializing full CPU or memory arrays.
- Visible `inspect` output remains the same.

Method:
- Seed one run detail file with synthetic samples
- Run `cavira inspect bench-run` 10 times
- Compare average command time

| samples | before avg | after avg | change |
| ---: | ---: | ---: | ---: |
| 10,000 | 4.295 ms | 4.169 ms | 2.9% faster |
| 100,000 | 25.068 ms | 26.202 ms | 4.5% slower |
| 1,000,000 | 233.733 ms | 225.020 ms | 3.7% faster |

Result:
- Small and medium files are effectively noise-level changes.
- At 1,000,000 samples, inspect improved from 233.733 ms to 225.020 ms, a 3.7% reduction.
