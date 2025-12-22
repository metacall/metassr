# MetaSSR Benchmarks

Simple performance benchmarking for MetaSSR.

## Quick Start

```bash
# Start your MetaSSR server first, then:
python3 benchmarks/benchmark.py

# Or with options:
python3 benchmarks/benchmark.py --port 3000 --skip-build

# Analyze existing results:
python3 benchmarks/benchmark.py --analyze-only .bench/results.json
```

## Requirements

- `wrk` - HTTP benchmarking tool
- `curl` - HTTP client
- `lsof` - For memory monitoring (optional)
- Python 3.6+

Install on Ubuntu/Debian:
```bash
sudo apt-get install wrk curl lsof python3
```

## Output

Results are saved to `.bench/`:
- `results.json` - Raw benchmark data with system info
- `summary.md` - Human-readable summary with Mermaid charts

## Metrics Collected

| Metric | Description |
|--------|-------------|
| RPS | Requests per second |
| Latency | Average response latency |
| P99 | 99th percentile latency |
| Memory | Server memory usage (MB) |
| Requests | Total requests handled |
| Errors | Socket/connection errors |

## Test Scenarios

| Test | Threads | Connections | Duration |
|------|---------|-------------|----------|
| Light | 1 | 10 | 30s |
| Medium | 4 | 50 | 30s |
| Heavy | 8 | 200 | 30s |
| Stress | 12 | 500 | 30s |

## Options

```
-u, --url URL       Server URL (default: http://localhost:8080)
-p, --port PORT     Server port (default: 8080)
-o, --output DIR    Output directory (default: .bench)
-s, --skip-build    Skip building the project
--analyze-only FILE Only analyze existing results.json
```

## Charts Generated

The summary includes Mermaid charts for:
- Requests per Second (RPS)
- Average Latency
- P99 Latency  
- Memory Usage
- Total Requests Handled