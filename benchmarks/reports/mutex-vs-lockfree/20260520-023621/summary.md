# MetaSSR Mutex vs Lockfree Benchmark

- Date: 2026-05-20 02:36:21
- URL: `http://127.0.0.1:8080/api/bench`
- Mutex commit: `fbc73bc411e55ac94743f5302c99c1da95febeab^` (dd1611d)
- Lockfree commit: `8c36012b3fea78304e193898d4568608ecd0affa` (8c36012)

| Scenario | Mutex RPS | Lockfree RPS | RPS Delta | Mutex P99 (ms) | Lockfree P99 (ms) | P99 Delta | Errors (M/L) |
|---|---:|---:|---:|---:|---:|---:|---:|
| Light | 43025 | 132711 | +208.45% | 0.92 | 0.17 | -81.41% | 0/0 |
| Medium | 43002 | 290822 | +576.30% | 3.27 | 0.51 | -84.34% | 0/0 |
| Heavy | 41518 | 238100 | +473.49% | 13.41 | 2.88 | -78.52% | 0/0 |
| Stress | 36604 | 248414 | +578.66% | 38.67 | 6.29 | -83.73% | 0/0 |
