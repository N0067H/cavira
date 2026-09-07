# Optimize History Append

Date: 2026-09-07

Environment:
- OS: Linux 6.6.87.2-microsoft-standard-WSL2 x86_64
- rustc: 1.95.0
- cargo: 1.95.0
- profile: `cargo build --release`

Baseline:
- before: `ccfe498`
- after: `04375d5`

Problem:
- Every recorded run called `store::append()`.
- The old append path loaded the full `history.json`, pushed one new entry into the Vec, serialized the whole Vec again, and rewrote the full file.
- That made normal profiling commands slower as local history grew, even when the new run itself was small.

Change:
- New history entries are appended to `history.jsonl` as one compact JSON object per line.
- `cavira history` reads both the old `history.json` array and the new `history.jsonl` records.
- Existing `history.json` files are left in place so old history remains visible.

Method:
- Seed `XDG_DATA_HOME/cavira/history.json` with existing entries
- Run `cavira run --silent true` 20 times
- Compare average command time and resulting history storage size

| existing entries | before avg | after avg | time change | before size | after size | size change |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 124.413 ms | 122.787 ms | 1.3% faster | 235,452 B | 192,250 B | 18.3% smaller |
| 10,000 | 142.536 ms | 123.180 ms | 13.6% faster | 2,323,452 B | 1,902,250 B | 18.1% smaller |
| 100,000 | 321.233 ms | 120.508 ms | 62.5% faster | 23,293,452 B | 19,092,250 B | 18.0% smaller |

Result:
- Large histories show the expected improvement.
- With 100,000 existing entries, average command time dropped from 321.233 ms to 120.508 ms, a 62.5% reduction.
