# MetaSSR Mutex vs Lockfree Benchmark

- Date: 2026-05-17 22:14:05
- Mutex commit: `fbc73bc411e55ac94743f5302c99c1da95febeab^` (dd1611d)
- Lockfree commit: `HEAD` (6e66af6)

| Scenario | Mutex RPS | Lockfree RPS | RPS Delta | Mutex P99 (ms) | Lockfree P99 (ms) | P99 Delta | Errors (M/L) |
|---|---:|---:|---:|---:|---:|---:|---:|
| Light | 174239 | 161734 | -7.18% | 0.14 | 0.13 | -7.25% | 0/0 |
| Medium | 500508 | 540001 | +7.89% | 0.61 | 0.50 | -18.57% | 0/0 |
| Heavy | 735696 | 755691 | +2.72% | 1.06 | 1.08 | +1.89% | 0/0 |
| Stress | 811540 | 868067 | +6.97% | 2.99 | 2.90 | -3.01% | 0/0 |
