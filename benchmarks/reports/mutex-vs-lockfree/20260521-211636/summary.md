# MetaSSR Mutex vs Lockfree Benchmark

- Date: 2026-05-21 21:16:36
- URL: `http://127.0.0.1:8080/api/bench`
- Mutex commit: `fbc73bc411e55ac94743f5302c99c1da95febeab^` (dd1611d)
- Lockfree commit: `8c36012b3fea78304e193898d4568608ecd0affa` (8c36012)

| Scenario | Mutex RPS | Lockfree RPS | RPS Delta | Mutex P99 (ms) | Lockfree P99 (ms) | P99 Delta | Errors (M/L) |
|---|---:|---:|---:|---:|---:|---:|---:|
| Light | 49088 | 132562 | +170.05% | 0.65 | 0.14 | -78.70% | 0/0 |
| Medium | 45961 | 248847 | +441.43% | 2.30 | 0.64 | -72.35% | 0/0 |
| Heavy | 43403 | 266665 | +514.39% | 9.93 | 2.49 | -74.92% | 0/0 |
| Stress | 43719 | 231476 | +429.47% | 28.54 | 8.45 | -70.39% | 0/0 |
