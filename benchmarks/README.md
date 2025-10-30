# MetaSSR Benchmarks

This directory contains all benchmark-related scripts and configurations for MetaSSR performance testing.

## Scripts Overview

- `run-benchmarks.sh` - Main automated benchmark runner
- `benchmark.sh` - Core benchmark execution script  
- `analyze-benchmarks.py` - Results analysis and reporting
- `generate-pr-summary.py` - Generate PR comment summaries
- `benchmark-config.json` - Test scenarios configuration
- `requirements.txt` - Python dependencies

## Quick Start

```bash
# Run full benchmark suite
./benchmarks/run-benchmarks.sh

# Run with custom options
./benchmarks/run-benchmarks.sh --port 3000 --build debug --graphs

# Analyze existing results
python3 benchmarks/analyze-benchmarks.py benchmark-results/results.json --plots
```

## Dependencies

### System Requirements
- `wrk` - HTTP benchmarking tool
- `jq` - JSON processor
- `curl` - HTTP client
- `lsof` - List open files (for process monitoring)

### Python Requirements
Install with: `pip install -r benchmarks/requirements.txt`
- pandas - Data analysis
- matplotlib - Plotting
- seaborn - Statistical visualization  
- numpy - Numerical computing

## Benchmark Scenarios

Configured in `benchmark-config.json`:

| Scenario | Purpose | Threads | Connections | Duration |
|----------|---------|---------|-------------|----------|
| Light Load | Basic functionality | 1 | 10 | 30s |
| Medium Load | Typical usage | 4 | 50 | 30s |
| Standard Load | Standard testing | 8 | 100 | 30s |
| Heavy Load | Peak performance | 12 | 500 | 30s |
| Extreme Load | Stress testing | 16 | 1000 | 30s |
| Sustained Load | Stability testing | 8 | 200 | 2min |
| Endurance Test | Long-term stability | 4 | 100 | 5min |

## Output Formats

- **JSON** - Structured results for analysis
- **CSV** - Tabular data for spreadsheets
- **Markdown** - Human-readable reports
- **PNG** - Performance charts (with --plots)

## CI/CD Integration

The benchmarks are automatically run via GitHub Actions on:
- Push to master
- Pull requests  
- Weekly schedule
- Manual workflow dispatch

Results are posted as PR comments and stored as workflow artifacts.

## Contributing

When modifying benchmarks:
1. Test locally first
2. Update configuration if adding scenarios
3. Ensure scripts remain executable
4. Update documentation accordingly