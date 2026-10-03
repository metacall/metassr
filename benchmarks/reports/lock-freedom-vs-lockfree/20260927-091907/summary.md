# MetaSSR lock_freedom vs lockfree Benchmark

- Date: 20260927-091907
- `lockfree` 0.5.1 (abandoned) vs `lock_freedom` 0.1.1 (maintained fork)
- Ops per thread: 300000, pre-populated keys: 30000

| Workload | Threads | lockfree ops/s | lock_freedom ops/s | Delta |
|---|---:|---:|---:|---:|
| Insert | 1 | 2278471 | 3634502 | +59.51% |
| Get | 1 | 16854471 | 13315504 | -21.00% |
| Mixed | 1 | 14864959 | 15696061 | +5.59% |
| Insert | 2 | 4896573 | 5061922 | +3.38% |
| Get | 2 | 12816391 | 13325491 | +3.97% |
| Mixed | 2 | 13399287 | 12598897 | -5.97% |
| Insert | 4 | 10279473 | 8993386 | -12.51% |
| Get | 4 | 17173340 | 16774977 | -2.32% |
| Mixed | 4 | 15077535 | 14256008 | -5.45% |
| Insert | 8 | 9775863 | 9633378 | -1.46% |
| Get | 8 | 14197094 | 16377514 | +15.36% |
| Mixed | 8 | 15044481 | 13122034 | -12.78% |

Workloads (u64 keys):
- `Insert`: each thread inserts its own keys (no collisions).
- `Get`: all threads read a shared key space (contended reads, the API hot path).
- `Mixed`: reads with 1% remove+reinsert churn (hot-reload scenario).

Positive delta means `lock_freedom` is faster.
