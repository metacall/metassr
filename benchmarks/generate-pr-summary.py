#!/usr/bin/env python3
"""Generate PR benchmark summary from benchmark results"""

import json
import os
import sys
from pathlib import Path
from datetime import datetime

def find_latest_results():
    """Find the latest benchmark results file"""
    # Look for results in .bench structure
    bench_dirs = [
        ".bench",
        "benchmark-results", 
        "tests/web-app/benchmark-results"
    ]
    
    for bench_dir in bench_dirs:
        bench_path = Path(bench_dir)
        if bench_path.exists():
            if bench_dir == ".bench":
                # Find latest session directory
                session_dirs = [d for d in bench_path.iterdir() 
                               if d.is_dir() and d.name.isdigit() or '_' in d.name]
                if session_dirs:
                    latest_session = max(session_dirs, key=os.path.getmtime)
                    results_dir = latest_session / "results"
                    if results_dir.exists():
                        json_files = list(results_dir.glob("benchmark.json"))
                        if json_files:
                            return json_files[0]
            else:
                # Legacy structure
                json_files = list(bench_path.glob("benchmark_*.json"))
                if json_files:
                    return max(json_files, key=os.path.getmtime)
    
    return None

def find_latest_summary():
    """Find the latest benchmark summary markdown file"""
    bench_path = Path(".bench")
    if bench_path.exists():
        session_dirs = [d for d in bench_path.iterdir() 
                       if d.is_dir() and (d.name.isdigit() or '_' in d.name)]
        if session_dirs:
            latest_session = max(session_dirs, key=os.path.getmtime)
            summary_file = latest_session / "results" / "benchmark_summary.md"
            if summary_file.exists():
                return summary_file
    return None

def format_rps(rps_str):
    """Format RPS for display"""
    if not rps_str:
        return "N/A"
    try:
        rps = float(str(rps_str).replace(',', ''))
        return f"{rps:,.0f}" if rps >= 1000 else f"{rps:.1f}"
    except:
        return str(rps_str)

def generate_mermaid_chart(tests):
    """Generate Mermaid bar chart for RPS results"""
    if not tests:
        return ""
    
    chart_lines = [
        "```mermaid",
        "%%{init: {'theme':'base', 'themeVariables': {'primaryColor': '#f9f9f9'}}}%%",
        "xychart-beta",
        '    title "Requests per Second by Test Scenario"',
        '    x-axis [' + ', '.join([f'"{test["name"]}"' for test in tests]) + ']',
        '    y-axis "RPS" 0 --> ' + str(max([
            int(float(str(test['results'].get('requests_per_sec', '0')).replace(',', '')) or 0) 
            for test in tests
        ], default=1000)),
        '    bar [' + ', '.join([
            str(int(float(str(test['results'].get('requests_per_sec', '0')).replace(',', '')) or 0))
            for test in tests
        ]) + ']',
        "```"
    ]
    
    return "\n".join(chart_lines)

def generate_performance_pie_chart(tests):
    """Generate Mermaid pie chart for performance distribution"""
    if not tests:
        return ""
    
    # Count performance levels
    performance_counts = {}
    for test in tests:
        indicator = get_performance_indicator(test['results'].get('requests_per_sec', '0'))
        performance_counts[indicator] = performance_counts.get(indicator, 0) + 1
    
    if not performance_counts:
        return ""
    
    chart_lines = [
        "```mermaid",
        "%%{init: {'theme':'base', 'themeVariables': {'primaryColor': '#f9f9f9'}}}%%",
        "pie title Performance Distribution",
    ]
    
    for level, count in performance_counts.items():
        chart_lines.append(f'    "{level}" : {count}')
    
    chart_lines.append("```")
    
    return "\n".join(chart_lines)

def get_performance_indicator(rps):
    """Get performance indicator based on RPS"""
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

def generate_pr_summary(results_file):
    """Generate PR summary"""
    with open(results_file, 'r') as f:
        data = json.load(f)
    
    commit_sha = os.environ.get('GITHUB_SHA', '')
    runner_os = os.environ.get('RUNNER_OS', 'ubuntu')
    
    summary = [
        "# MetaSSR Benchmark Results",
        "",
        f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')}",
        f"**Runner:** {runner_os}",
        f"**Commit:** `{commit_sha[:8]}`" if commit_sha else "",
        ""
    ]
    
    # System information
    metadata = data.get('metadata', {})
    if metadata:
        summary.extend([
            "## System Information",
            "",
            f"- **OS:** {metadata.get('os', 'Unknown')} {metadata.get('arch', '')}",
            f"- **CPU Cores:** {metadata.get('cpu_cores', 'Unknown')}",
            f"- **Memory:** {metadata.get('memory_gb', 'Unknown')} GB",
            ""
        ])
    
    tests = data.get('tests', [])
    if not tests:
        summary.append("No benchmark tests found.")
        return "\n".join(summary)
    
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
    
    summary.extend([
        "## Performance Summary",
        "",
        f"**Best Performance:** {best_test} - {format_rps(best_rps)} RPS",
        f"**Total Requests:** {total_requests:,}",
        "",
        "## Performance Charts",
        "",
        generate_mermaid_chart(tests),
        "",
        generate_performance_pie_chart(tests),
        "",
        "## Test Results",
        "",
        "| Test | Performance | RPS | Avg Latency | P99 Latency | Errors |",
        "|------|-------------|-----|-------------|-------------|--------|"
    ])
    
    for test in tests:
        name = test['name']
        results = test['results']
        rps = format_rps(results.get('requests_per_sec'))
        indicator = get_performance_indicator(results.get('requests_per_sec', '0'))
        avg_lat = results.get('avg_latency', 'N/A')
        p99_lat = results.get('latency_percentiles', {}).get('p99', 'N/A')
        errors = results.get('total_errors', '0')
        
        summary.append(f"| {name} | {indicator} | **{rps}** | {avg_lat} | {p99_lat} | {errors} |")
    
    # Check for errors
    error_tests = [test['name'] for test in tests 
                   if int(str(test['results'].get('total_errors', '0'))) > 0]
    
    if error_tests:
        summary.extend(["", "**Tests with errors:**"])
        summary.extend([f"- {test}" for test in error_tests])
    else:
        summary.extend(["", "**All tests completed without errors**"])
    
    summary.extend([
        "",
        "---",
        "*Full benchmark data available in workflow artifacts.*"
    ])
    
    return "\n".join(summary)

def main():
    # First try to find the latest summary markdown file
    summary_file = find_latest_summary()
    
    if summary_file:
        # Use the pre-generated summary directly
        with open(summary_file, 'r') as f:
            summary = f.read()
    else:
        # Fallback to generating from JSON results
        results_file = find_latest_results()
        
        if not results_file:
            commit_sha = os.environ.get('GITHUB_SHA', '')
            runner_os = os.environ.get('RUNNER_OS', 'ubuntu')
            
            summary = f"""# MetaSSR Benchmark Results

**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')}
**Runner:** {runner_os}
**Commit:** `{commit_sha[:8] if commit_sha else 'unknown'}`

**No benchmark results found**

Check the workflow logs for details.
"""
        else:
            summary = generate_pr_summary(results_file)
    
    with open('pr_benchmark_summary.md', 'w') as f:
        f.write(summary)
    
    print("Generated PR benchmark summary")
    return 0

if __name__ == '__main__':
    sys.exit(main())