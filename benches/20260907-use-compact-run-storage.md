# Use Compact Run Storage

Date: 2026-09-07

Environment:
- OS: Linux 6.6.87.2-microsoft-standard-WSL2 x86_64
- rustc: 1.95.0
- cargo: 1.95.0
- profile: `cargo build --release`

Baseline:
- before: `3868c90`, pretty JSON string plus full-file write for run detail storage
- after: current change, compact JSON written with `serde_json::to_writer`

Problem:
- `store::save_run()` used `serde_json::to_string_pretty()` for internal run detail files.
- Pretty JSON is useful for people reading command output, but internal persisted run detail files are usually read back by Cavira.
- For sample-heavy runs, pretty formatting increases file size and requires building a full intermediate String before writing.

Change:
- `store::save_run()` now creates the run detail file and writes compact JSON directly with `serde_json::to_writer`.
- User-facing `--json` output remains pretty JSON.
- History storage was already compact JSONL from the history append optimization.

Method:
- Build before and after with `cargo build --release`.
- Run `cavira run --silent -i 0 -t 1000ms sleep 2` 10 times per variant.
- Use a temporary `XDG_DATA_HOME` for each run.
- Compare external wall time, stored run detail size, and sample count.

| variant | avg wall | median wall | avg samples | median samples | avg run file size | median run file size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| before | 1012.346 ms | 1009.478 ms | 348.4 | 348.5 | 33,650.4 B | 33,658.0 B |
| after | 1011.279 ms | 1011.035 ms | 349.9 | 350.5 | 21,853.5 B | 21,891.5 B |

Immediate numbers:
- Average run detail size dropped from 33,650.4 B to 21,853.5 B, 35.1% smaller.
- Median run detail size dropped from 33,658.0 B to 21,891.5 B, 35.0% smaller.
- Average wall time changed from 1012.346 ms to 1011.279 ms, 0.1% faster.

Result:
- File size improvement is clear for persisted run detail storage.
- Wall time is effectively unchanged in this CLI scenario because the 1000 ms profiling timeout dominates the measurement.
