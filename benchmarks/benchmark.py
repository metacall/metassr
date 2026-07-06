#!/usr/bin/env python3
"""MetaSSR Benchmark Suite - Simple Performance Testing"""

import argparse
import json
import os
import platform
import re
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

# Colors for terminal output
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'

def log(msg): print(f"{Colors.BLUE}[INFO]{Colors.NC} {msg}")
def error(msg): print(f"{Colors.RED}[ERROR]{Colors.NC} {msg}", file=sys.stderr)
def success(msg): print(f"{Colors.GREEN}[OK]{Colors.NC} {msg}")

# Test scenarios: (name, threads, connections, duration)
SCENARIOS = [
    ("Light",  1,  10,  20),
    ("Medium", 4,  50,  40),
    ("Heavy",  8,  200, 80),
    ("Stress", 12, 500, 120),
]

def check_deps():
    """Check required dependencies"""
    missing = []
    for dep in ["wrk", "curl"]:
        if subprocess.run(["which", dep], capture_output=True).returncode != 0:
            missing.append(dep)
    if missing:
        error(f"Missing: {', '.join(missing)}")
        print("Install with: sudo apt-get install wrk curl")
        sys.exit(1)

def wait_for_server(url, timeout=30):
    """Wait for server to be ready"""
    log(f"Waiting for server at {url}...")
    for _ in range(timeout):
        try:
            result = subprocess.run(
                ["curl", "-s", "--max-time", "2", url],
                capture_output=True
            )
            if result.returncode == 0:
                return True
        except:
            pass
        time.sleep(1)
    error("Server not responding")
    return False

def get_server_pid(port):
    """Get PID of server running on given port"""
    try:
        result = subprocess.run(
            ["lsof", "-ti", f":{port}"],
            capture_output=True, text=True
        )
        if result.returncode == 0 and result.stdout.strip():
            return int(result.stdout.strip().split()[0])
    except:
        pass
    return None

def get_memory_usage(pid):
    """Get memory usage in MB for a process"""
    if not pid:
        return 0
    try:
        if platform.system() == "Linux":
            with open(f"/proc/{pid}/status") as f:
                for line in f:
                    if line.startswith("VmRSS:"):
                        return int(line.split()[1]) / 1024  # KB to MB
        elif platform.system() == "Darwin":
            result = subprocess.run(
                ["ps", "-o", "rss=", "-p", str(pid)],
                capture_output=True, text=True
            )
            if result.returncode == 0:
                return int(result.stdout.strip()) / 1024  # KB to MB
    except:
        pass
    return 0

def parse_latency_ms(latency_str):
    """Convert latency string to milliseconds"""
    if not latency_str:
        return 0
    match = re.match(r'([\d.]+)(\w+)', latency_str)
    if not match:
        return 0
    value, unit = float(match.group(1)), match.group(2).lower()
    if unit == 'us':
        return value / 1000
    elif unit == 'ms':
        return value
    elif unit == 's':
        return value * 1000
    return value

def parse_wrk_output(output):
    """Parse wrk output and extract metrics"""
    result = {
        "rps": 0,
        "latency": "0ms",
        "latency_ms": 0,
        "p99": "0ms",
        "p99_ms": 0,
        "requests": 0,
        "errors": 0
    }
    
    # Requests/sec
    match = re.search(r'Requests/sec:\s+([\d.]+)', output)
    if match:
        result["rps"] = float(match.group(1))
    
    # Average latency
    match = re.search(r'Latency\s+([\d.]+\w+)', output)
    if match:
        result["latency"] = match.group(1)
        result["latency_ms"] = parse_latency_ms(match.group(1))
    
    # P99 latency
    match = re.search(r'99%\s+([\d.]+\w+)', output)
    if match:
        result["p99"] = match.group(1)
        result["p99_ms"] = parse_latency_ms(match.group(1))
    
    # Total requests
    match = re.search(r'(\d+)\s+requests in', output)
    if match:
        result["requests"] = int(match.group(1))
    
    # Socket errors
    match = re.search(r'Socket errors:.*?(\d+)', output)
    if match:
        result["errors"] = int(match.group(1))
    
    return result

def run_test(name, threads, connections, duration, url, server_pid):
    """Run a single benchmark test"""
    print(f"{Colors.YELLOW}[{name}]{Colors.NC} threads={threads} connections={connections} duration={duration}s")
    
    mem_before = get_memory_usage(server_pid)
    
    cmd = ["wrk", f"-t{threads}", f"-c{connections}", f"-d{duration}s", "--latency", url]
    result = subprocess.run(cmd, capture_output=True, text=True)
    output = result.stdout + result.stderr
    
    mem_after = get_memory_usage(server_pid)
    mem_peak = max(mem_before, mem_after)
    
    metrics = parse_wrk_output(output)
    metrics["memory_mb"] = round(mem_peak, 1)
    
    print(f"  RPS: {metrics['rps']:.0f} | Latency: {metrics['latency']} | P99: {metrics['p99']} | Memory: {metrics['memory_mb']:.1f}MB")
    
    return {
        "name": name,
        "threads": threads,
        "connections": connections,
        "duration": duration,
        **metrics
    }

def get_system_info():
    """Collect system information"""
    info = {
        "os": platform.system(),
        "os_version": platform.release(),
        "arch": platform.machine(),
        "python": platform.python_version(),
        "cpu": "Unknown",
        "cpu_cores": os.cpu_count() or 0,
        "memory_gb": 0
    }
    
    # Get CPU info
    try:
        if platform.system() == "Linux":
            with open("/proc/cpuinfo") as f:
                for line in f:
                    if "model name" in line:
                        info["cpu"] = line.split(":")[1].strip()
                        break
        elif platform.system() == "Darwin":
            result = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"], 
                                   capture_output=True, text=True)
            if result.returncode == 0:
                info["cpu"] = result.stdout.strip()
    except:
        pass
    
    # Get memory info
    try:
        if platform.system() == "Linux":
            with open("/proc/meminfo") as f:
                for line in f:
                    if "MemTotal" in line:
                        kb = int(line.split()[1])
                        info["memory_gb"] = round(kb / 1024 / 1024, 1)
                        break
        elif platform.system() == "Darwin":
            result = subprocess.run(["sysctl", "-n", "hw.memsize"], 
                                   capture_output=True, text=True)
            if result.returncode == 0:
                info["memory_gb"] = round(int(result.stdout.strip()) / 1024 / 1024 / 1024, 1)
    except:
        pass
    
    return info

def generate_summary(results, output_dir, server_url, system_info):
    """Generate markdown summary with Mermaid charts"""
    # Build chart data
    labels = ", ".join(f'"{r["name"]}"' for r in results)
    rps_values = ", ".join(str(int(r["rps"])) for r in results)
    latency_values = ", ".join(f'{r["latency_ms"]:.2f}' for r in results)
    p99_values = ", ".join(f'{r["p99_ms"]:.2f}' for r in results)
    memory_values = ", ".join(str(r["memory_mb"]) for r in results)
    requests_values = ", ".join(str(r["requests"]) for r in results)
    
    # Calculate max values for chart scaling (with 20% padding)
    max_rps = int(max((r["rps"] for r in results), default=100) * 1.2)
    max_latency = max((r["latency_ms"] for r in results), default=1) * 1.2
    max_p99 = max((r["p99_ms"] for r in results), default=1) * 1.2
    max_memory = max((r["memory_mb"] for r in results), default=100) * 1.2
    max_requests = int(max((r["requests"] for r in results), default=1000) * 1.2)
    
    # Find best result
    best = max(results, key=lambda r: r["rps"])
    
    # Check for errors
    total_errors = sum(r["errors"] for r in results)
    error_status = "PASSED - All tests completed successfully" if total_errors == 0 else f"WARNING - {total_errors} errors detected"
    
    summary = f"""# MetaSSR Benchmark Results

**Date:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  
**Server:** {server_url}  
**Status:** {error_status}

## System Information

| Property | Value |
|----------|-------|
| OS | {system_info['os']} {system_info['os_version']} |
| Architecture | {system_info['arch']} |
| CPU | {system_info['cpu']} |
| CPU Cores | {system_info['cpu_cores']} |
| Memory | {system_info['memory_gb']} GB |

## Performance Charts

### Requests per Second
```mermaid
xychart-beta
    title "Requests per Second"
    x-axis [{labels}]
    y-axis "RPS" 0 --> {max_rps}
    bar [{rps_values}]
```

### Average Latency (ms)
```mermaid
xychart-beta
    title "Average Latency"
    x-axis [{labels}]
    y-axis "Latency (ms)" 0 --> {max_latency:.1f}
    bar [{latency_values}]
```

### P99 Latency (ms)
```mermaid
xychart-beta
    title "P99 Latency"
    x-axis [{labels}]
    y-axis "P99 (ms)" 0 --> {max_p99:.1f}
    bar [{p99_values}]
```

### Memory Usage (MB)
```mermaid
xychart-beta
    title "Memory Usage"
    x-axis [{labels}]
    y-axis "Memory (MB)" 0 --> {max_memory:.0f}
    bar [{memory_values}]
```

### Total Requests
```mermaid
xychart-beta
    title "Total Requests Handled"
    x-axis [{labels}]
    y-axis "Requests" 0 --> {max_requests}
    bar [{requests_values}]
```

## Detailed Results

| Test | RPS | Avg Latency | P99 Latency | Memory | Requests | Errors |
|------|-----|-------------|-------------|--------|----------|--------|
"""
    
    for r in results:
        error_cell = f"FAIL ({r['errors']})" if r['errors'] > 0 else "OK"
        summary += f"| {r['name']} | {int(r['rps']):,} | {r['latency']} | {r['p99']} | {r['memory_mb']:.1f} MB | {r['requests']:,} | {error_cell} |\n"
    
    summary += f"""
## Summary

| Metric | Best | Average | Worst |
|--------|------|---------|-------|
| RPS | {max(r['rps'] for r in results):,.0f} | {sum(r['rps'] for r in results)/len(results):,.0f} | {min(r['rps'] for r in results):,.0f} |
| Latency | {min(r['latency_ms'] for r in results):.2f}ms | {sum(r['latency_ms'] for r in results)/len(results):.2f}ms | {max(r['latency_ms'] for r in results):.2f}ms |
| P99 | {min(r['p99_ms'] for r in results):.2f}ms | {sum(r['p99_ms'] for r in results)/len(results):.2f}ms | {max(r['p99_ms'] for r in results):.2f}ms |
| Memory | {min(r['memory_mb'] for r in results):.1f}MB | {sum(r['memory_mb'] for r in results)/len(results):.1f}MB | {max(r['memory_mb'] for r in results):.1f}MB |

**Best Performance:** {best['name']} with {int(best['rps']):,} RPS
"""
    
    # Write summary
    summary_file = output_dir / "summary.md"
    summary_file.write_text(summary)
    
    return summary

def analyze_results(results):
    """Print analysis of benchmark results"""
    print(f"\n{'='*60}")
    print("BENCHMARK ANALYSIS")
    print(f"{'='*60}\n")
    
    print(f"{'Test':<12} | {'RPS':>12} | {'Latency':>10} | {'P99':>10} | {'Memory':>10} | Status")
    print("-" * 75)
    
    for r in results:
        status = "OK" if r["errors"] == 0 else f"FAIL ({r['errors']})"
        print(f"{r['name']:<12} | {r['rps']:>12,.0f} | {r['latency']:>10} | {r['p99']:>10} | {r['memory_mb']:>8.1f}MB | {status}")
    
    print(f"\n{'-'*60}")
    print(f"Max RPS: {max(r['rps'] for r in results):,.0f}")
    print(f"Avg RPS: {sum(r['rps'] for r in results)/len(results):,.0f}")
    print(f"Max Memory: {max(r['memory_mb'] for r in results):.1f}MB")
    
    error_tests = [r["name"] for r in results if r["errors"] > 0]
    if error_tests:
        print(f"\n[WARNING] Tests with errors: {', '.join(error_tests)}")
    else:
        print("\n[OK] All tests passed without errors")

def main():
    parser = argparse.ArgumentParser(description="MetaSSR Benchmark Suite")
    parser.add_argument("-u", "--url", default="http://localhost:8080", help="Server URL")
    parser.add_argument("-p", "--port", type=int, default=8080, help="Server port")
    parser.add_argument("-o", "--output", default=".bench", help="Output directory")
    parser.add_argument("-s", "--skip-build", action="store_true", help="Skip building")
    parser.add_argument("--analyze-only", metavar="FILE", help="Only analyze existing results.json")
    args = parser.parse_args()
    
    # Handle analyze-only mode
    if args.analyze_only:
        results_file = Path(args.analyze_only)
        if not results_file.exists():
            error(f"File not found: {results_file}")
            sys.exit(1)
        data = json.loads(results_file.read_text())
        analyze_results(data["tests"])
        return
    
    # Setup
    server_url = f"http://localhost:{args.port}" if args.port else args.url
    script_dir = Path(__file__).parent.resolve()
    project_root = script_dir.parent
    output_dir = project_root / args.output
    output_dir.mkdir(exist_ok=True)
    
    print(f"{Colors.BLUE}=== MetaSSR Benchmark ==={Colors.NC}")
    
    check_deps()
    
    # Collect system info
    system_info = get_system_info()
    log(f"System: {system_info['os']} {system_info['arch']}, {system_info['cpu_cores']} cores, {system_info['memory_gb']}GB RAM")
    
    # Build if needed
    if not args.skip_build and (project_root / "Cargo.toml").exists():
        log("Building MetaSSR...")
        subprocess.run(["cargo", "build", "--release"], cwd=project_root, check=True)
        
        # Use standalone benchmark app
        bench_app = script_dir / "apps" / "metassr-app"
        if bench_app.exists():
            log("Setting up benchmark app...")
            subprocess.run(["npm", "install", "--silent"], cwd=bench_app, check=True)
            subprocess.run(["npm", "run", "build"], cwd=bench_app, capture_output=True)
            subprocess.Popen(["npm", "start"], cwd=bench_app,
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        else:
            print("Apps directory is not exist")
    
    # Wait for server
    if not wait_for_server(server_url):
        sys.exit(1)
    
    # Get server PID for memory monitoring
    server_pid = get_server_pid(args.port)
    if server_pid:
        log(f"Monitoring server process (PID: {server_pid})")
    else:
        log("Could not find server PID, memory monitoring disabled")
    
    # Warmup
    log("Warming up server...")
    for _ in range(5):
        subprocess.run(["curl", "-s", server_url], capture_output=True)
    time.sleep(2)
    
    # Run benchmarks
    results = []
    for name, threads, connections, duration in SCENARIOS:
        result = run_test(name, threads, connections, duration, server_url, server_pid)
        results.append(result)
    
    # Save JSON results
    results_data = {
        "timestamp": datetime.now().isoformat(),
        "server": server_url,
        "system": system_info,
        "tests": results
    }
    results_file = output_dir / "results.json"
    results_file.write_text(json.dumps(results_data, indent=2))
    
    # Generate summary
    summary = generate_summary(results, output_dir, server_url, system_info)
    
    success(f"Results saved to {output_dir}/")
    
    # Print summary and analysis
    print()
    print(summary)
    analyze_results(results)

if __name__ == "__main__":
    main()
