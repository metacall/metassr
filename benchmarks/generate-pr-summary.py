#!/usr/bin/env python3
"""
Generate PR benchmark summary from benchmark results
"""

import json
import os
import sys
from pathlib import Path
from datetime import datetime

def find_latest_results(results_dir="benchmark-results"):
    """Find the latest benchmark results file"""
    # Try multiple possible locations
    possible_dirs = [
        results_dir,
        "benchmark-results", 
        "tests/web-app/benchmark-results",
        "../benchmark-results"
    ]
    
    for dir_path in possible_dirs:
        results_path = Path(dir_path)
        if results_path.exists():
            json_files = list(results_path.glob("benchmark_*.json"))
            if json_files:
                # Return the most recent file
                return max(json_files, key=os.path.getmtime)
    
    return None

def parse_latency(latency_str):
    """Parse latency string and convert to milliseconds"""
    if not latency_str or latency_str == "N/A":
        return 0
        
    latency_str = str(latency_str).lower().strip()
    
    try:
        # Check for most specific patterns first
        if 'ms' in latency_str:
            return float(latency_str.replace('ms', ''))
        elif 'us' in latency_str:
            return float(latency_str.replace('us', '')) / 1000
        elif latency_str.endswith('s'):
            # Only for pure seconds (not microseconds or milliseconds)
            return float(latency_str.replace('s', '')) * 1000
        else:
            # Try to parse as plain number (assume milliseconds)
            return float(latency_str)
    except (ValueError, TypeError):
        return 0

def format_rps(rps_str):
    """Format RPS for display"""
    if not rps_str or rps_str == "N/A":
        return "N/A"
    try:
        rps = float(str(rps_str).replace(',', ''))
        if rps >= 1000:
            return f"{rps:,.0f}"
        else:
            return f"{rps:.1f}"
    except:
        return str(rps_str)

def generate_performance_indicator(rps):
    """Generate performance indicator based on performance"""
    try:
        rps_num = float(str(rps).replace(',', ''))
        if rps_num >= 50000:
            return "EXCELLENT"
        elif rps_num >= 20000:
            return "HIGH"
        elif rps_num >= 10000:
            return "GOOD"
        elif rps_num >= 5000:
            return "FAIR"
        else:
            return "LOW"
    except:
        return "UNKNOWN"

def generate_pr_summary(results_file, commit_sha="", runner_os="ubuntu"):
    """Generate a comprehensive PR summary"""
    
    with open(results_file, 'r') as f:
        data = json.load(f)
    
    summary = []
    
    # Header
    summary.append("# MetaSSR Benchmark Results")
    summary.append("")
    summary.append(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')}")
    summary.append(f"**Runner:** {runner_os}")
    if commit_sha:
        summary.append(f"**Commit:** `{commit_sha[:8]}`")
    summary.append("")
    
    # System info
    metadata = data.get('metadata', {})
    if metadata:
        summary.append("## System Information")
        summary.append("")
        summary.append(f"- **OS:** {metadata.get('os', 'Unknown')} {metadata.get('arch', '')}")
        summary.append(f"- **CPU Cores:** {metadata.get('cpu_cores', 'Unknown')}")
        summary.append(f"- **Memory:** {metadata.get('memory_gb', 'Unknown')} GB")
        summary.append("")
    
    # Performance overview
    tests = data.get('tests', [])
    if tests:
        summary.append("## Performance Overview")
        summary.append("")
        
        # Find best performance
        best_rps = 0
        best_test = ""
        total_requests = 0
        
        for test in tests:
            try:
                rps = float(str(test['results'].get('requests_per_sec', '0')).replace(',', ''))
                if rps > best_rps:
                    best_rps = rps
                    best_test = test['name']
                total_requests += int(str(test['results'].get('total_requests', '0')).replace(',', ''))
            except:
                continue
        
        summary.append(f"**Best Performance:** {best_test} - {format_rps(best_rps)} RPS")
        summary.append(f"**Total Requests Processed:** {total_requests:,}")
        summary.append("")
    
    # Detailed results table
    if tests:
        summary.append("## Detailed Results")
        summary.append("")
        summary.append("| Test Scenario | Performance | RPS | Avg Latency | P99 Latency | Errors |")
        summary.append("|---------------|-------------|-----|-------------|-------------|--------|")
        
        for test in tests:
            name = test['name']
            results = test['results']
            
            rps = format_rps(results.get('requests_per_sec', 'N/A'))
            indicator = generate_performance_indicator(results.get('requests_per_sec', '0'))
            
            avg_lat = results.get('avg_latency', 'N/A')
            p99_lat = results.get('latency_percentiles', {}).get('p99', 'N/A')
            errors = results.get('total_errors', '0')
            
            # Format latencies
            if avg_lat != 'N/A':
                avg_lat_ms = parse_latency(avg_lat)
                avg_lat = f"{avg_lat_ms:.2f}ms" if avg_lat_ms > 0 else avg_lat
            
            if p99_lat != 'N/A':
                p99_lat_ms = parse_latency(p99_lat)
                p99_lat = f"{p99_lat_ms:.2f}ms" if p99_lat_ms > 0 else p99_lat
            
            summary.append(f"| {name} | {indicator} | **{rps}** | {avg_lat} | {p99_lat} | {errors} |")
        
        summary.append("")
    
    # Performance insights
    if tests:
        summary.append("## Performance Insights")
        summary.append("")
        
        # Calculate some insights
        latencies = []
        error_tests = []
        high_perf_tests = []
        
        for test in tests:
            results = test['results']
            
            # Check for errors
            errors = int(str(results.get('total_errors', '0')))
            if errors > 0:
                error_tests.append(f"{test['name']} ({errors} errors)")
            
            # Check for high performance
            try:
                rps = float(str(results.get('requests_per_sec', '0')).replace(',', ''))
                if rps >= 20000:
                    high_perf_tests.append(f"{test['name']} ({format_rps(rps)} RPS)")
            except:
                pass
            
            # Collect latencies
            avg_lat = parse_latency(results.get('avg_latency', '0'))
            if avg_lat > 0:
                latencies.append(avg_lat)
        
        # Generate insights
        if high_perf_tests:
            summary.append("**High Performance Tests:**")
            for test in high_perf_tests:
                summary.append(f"  - {test}")
            summary.append("")
        
        if latencies:
            avg_latency = sum(latencies) / len(latencies)
            if avg_latency < 50:
                summary.append("**Excellent latency performance** - Average latency under 50ms")
            elif avg_latency < 100:
                summary.append("**Good latency performance** - Average latency under 100ms")
            else:
                summary.append("**Consider latency optimization** - Average latency above 100ms")
            summary.append("")
        
        if error_tests:
            summary.append("**Tests with errors:**")
            for test in error_tests:
                summary.append(f"  - {test}")
            summary.append("")
        elif tests:
            summary.append("**All tests completed without errors**")
            summary.append("")
    
    # Footer
    summary.append("---")
    summary.append("*Detailed benchmark data and analysis reports are available in the workflow artifacts.*")
    summary.append("")
    summary.append("<details>")
    summary.append("<summary>How to reproduce these benchmarks</summary>")
    summary.append("")
    summary.append("```bash")
    summary.append("# Clone the repository")
    summary.append("git clone https://github.com/metacall/metassr.git")
    summary.append("cd metassr")
    summary.append("")
    summary.append("# Run benchmarks")
    summary.append("./benchmarks/run-benchmarks.sh")
    summary.append("```")
    summary.append("")
    summary.append("</details>")
    
    return "\n".join(summary)

def main():
    # Get environment variables
    commit_sha = os.environ.get('GITHUB_SHA', '')
    runner_os = os.environ.get('RUNNER_OS', 'ubuntu')
    
    # Find latest results
    results_file = find_latest_results()
    
    if not results_file:
        # Generate fallback summary
        summary = f"""# MetaSSR Benchmark Results

**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')}
**Runner:** {runner_os}
**Commit:** `{commit_sha[:8] if commit_sha else 'unknown'}`

**No benchmark results found**

The benchmark process completed but no results file was generated. 
Please check the workflow logs for details.

*Check the workflow artifacts for any available benchmark data.*
"""
    else:
        summary = generate_pr_summary(results_file, commit_sha, runner_os)
    
    # Write to file
    with open('pr_benchmark_summary.md', 'w') as f:
        f.write(summary)
    
    print("Generated PR benchmark summary")
    return 0

if __name__ == '__main__':
    sys.exit(main())