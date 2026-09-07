# Stream Sample Summary Stats

Date: 2026-09-07

Environment:
- OS: Linux 6.6.87.2-microsoft-standard-WSL2 x86_64
- rustc: 1.95.0
- cargo: 1.95.0
- profile: `rustc -O` synthetic microbenchmark

Baseline:
- before: `04375d5`, post-collection Vec rescans
- after: `652c5d2`, incremental stats finalization

Problem:
- `run`, `pid`, and `live` collected samples first and then scanned sample buffers again to compute peak CPU, average CPU, peak memory, and average memory.
- The extra scans were avoidable because each sample already passes through the collection loop once.
- The cost becomes visible with high sample counts or long-running profiling sessions.

Change:
- Added a shared `SampleStats` accumulator.
- Each command records CPU and memory into the accumulator while collecting samples.
- Final summary values are read from the accumulator instead of rescanning the sample Vec.

Method:
- Synthetic microbenchmark comparing old post-collection Vec rescans with incremental stats finalization.
- Sample generation cost is excluded from the measured old/new summary times.

| samples | old rescan | new finalize | change | speedup |
| ---: | ---: | ---: | ---: | ---: |
| 10,000 | 0.018087 ms | 0.000076 ms | 99.6% faster | 238.0x |
| 100,000 | 0.180636 ms | 0.000077 ms | >99.9% faster | 2,345.9x |
| 1,000,000 | 6.558958 ms | 0.000052 ms | >99.9% faster | 126,133.8x |
| 5,000,000 | 33.107192 ms | 0.000053 ms | >99.9% faster | 624,664.0x |

Result:
- Old summary work scales with sample count.
- New finalization is effectively constant time.
