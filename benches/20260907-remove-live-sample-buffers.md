# Remove Live Sample Buffers

Date: 2026-09-07

Environment:
- OS: Linux 6.6.87.2-microsoft-standard-WSL2 x86_64
- rustc: 1.95.0
- cargo: 1.95.0

Baseline:
- before: `652c5d2`, `live` keeps `cpu_samples`, `mem_samples`, and detail samples
- after: `3e4089e`, `live` keeps detail samples and uses incremental stats

Problem:
- `live` stored CPU samples in one Vec, memory samples in another Vec, and detailed samples in a third Vec.
- The detailed samples already contained both CPU and memory values.
- Long live sessions therefore kept duplicate per-sample data in memory.

Change:
- Removed the CPU-only and memory-only buffers from `live`.
- Kept detailed samples for persisted run detail output.
- Reused incremental summary stats for final peak and average values.

Method:
- Synthetic size calculation for removed duplicate buffers:
  - old `cpu_samples: Vec<f32>`
  - old `mem_samples: Vec<u64>`
- This excludes Vec capacity overhead and allocator metadata.

| samples | old duplicate payload | after duplicate payload | removed |
| ---: | ---: | ---: | ---: |
| 10,000 | 120,000 B | 0 B | 120,000 B |
| 100,000 | 1,200,000 B | 0 B | 1,200,000 B |
| 1,000,000 | 12,000,000 B | 0 B | 12,000,000 B |
| 5,000,000 | 60,000,000 B | 0 B | 60,000,000 B |

Result:
- Memory reduction scales linearly with live sample count.
- At 1,000,000 samples, the removed duplicate buffer payload is 12,000,000 B.
