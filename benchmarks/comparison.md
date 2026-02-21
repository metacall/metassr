# MetaSSR vs Next.js Performance Benchmark

## 1. Objective

This benchmark compares MetaSSR (Rust-based SSR framework) and Next.js (Node.js-based SSR framework) under identical load conditions.

Metrics evaluated:

- Requests per second (RPS)
- Average latency
- P99 latency
- Memory usage


## 2. Test Environment

- OS: Linux 6.11.0-17-generic
- CPU: 12th Gen Intel(R) Core(TM) i5-1235U (12 cores)
- RAM: 7.4 GB
- Rust: stable
- Node.js: v20.x
- Benchmark tool: `wrk`
- All tests executed on the same machine


## 3. Methodology

- Both frameworks served a minimal static SSR page.
- Production builds were used:
  - MetaSSR: `cargo build --release`
  - Next.js: `npm run build` + `npm start`
- No development servers were used.
- Identical load configurations:

| Test | Threads | Connections | Duration |
|------|---------|------------|----------|
| Light | 1 | 10 | 20s |
| Medium | 4 | 50 | 40s |
| Heavy | 8 | 200 | 80s |
| Stress | 12 | 500 | 120s |


## 4. Results

### Maximum Throughput

| Framework | Max RPS |
|------------|----------|
| MetaSSR | 364,662 |
| Next.js | 3,715 |

MetaSSR achieved approximately **98× higher throughput** under peak load.

---

### Latency (Stress Test)

| Framework | Avg Latency | P99 Latency |
|------------|-------------|--------------|
| MetaSSR | 1.33 ms | 4.24 ms |
| Next.js | 131.51 ms | 220.41 ms |

Additionally, a P99 spike of **1.64 seconds** was observed in Next.js under light load conditions.


### Raw Results

Full benchmark results are available in:

- `benchmarks/results/nextjs.json` (Next.js)
- `benchmarks/results/metassr.json` (MetaSSR)

These JSON files contain complete test metadata, system configuration, and per-test metrics.
------------------------
## 4. Results

### Maximum Throughput

| Framework | Max RPS |
|------------|----------|
| MetaSSR | 364,662 |
| Next.js | 3,715 |

MetaSSR achieved approximately **98× higher throughput** under peak load.


### Latency (Stress Test)

| Framework | Avg Latency | P99 Latency |
|------------|-------------|--------------|
| MetaSSR | 1.33 ms | 4.24 ms |
| Next.js | 131.51 ms | 220.41 ms |

Under stress conditions, MetaSSR maintains low and stable latency, while Next.js exhibits significantly higher response times.

Additionally, a P99 spike of **1.64 seconds** was observed in Next.js under light load conditions.


### Memory Usage

- MetaSSR peak memory usage: 23.3 MB
- Next.js memory not captured in this run (external process)

MetaSSR demonstrates a low and predictable memory footprint.


## 5. Analysis

The performance difference can be attributed to:

### 1. Runtime Architecture
- MetaSSR: Compiled Rust binary.
- Next.js: Node.js runtime (V8 engine).

### 2. Concurrency Model
- MetaSSR leverages multi-threading.
- Node.js uses an event-loop model.

### 3. Memory Model
- Rust uses deterministic memory management.
- Node.js includes garbage collection overhead.

Under identical conditions, MetaSSR demonstrates substantially higher throughput and significantly lower latency.


## 6. Limitations

- Benchmarks executed on a single machine.
- Results may vary across hardware and environments.
- Real-world applications with complex business logic may behave differently.
- CI environments may not reflect absolute performance values.


## 7. Conclusion

Under controlled test conditions, MetaSSR significantly outperforms Next.js in terms of throughput and latency.

These results highlight the performance advantages of a Rust-based SSR framework for high-throughput server workloads.

Note: CI benchmarks are intended for relative comparison and regression detection. Results may vary depending on runner hardware.

These results highlight the performance advantages of a Rust-based SSR framework for high-throughput server workloads.