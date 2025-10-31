#!/bin/bash

# MetaSSR Benchmark Suite
# Comprehensive performance testing for MetaSSR framework

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
DEFAULT_BENCH_DIR="$PROJECT_ROOT/.bench"
BENCH_DIR="${BENCH_DIR:-$DEFAULT_BENCH_DIR}"
BENCHMARK_SESSION_DIR="$BENCH_DIR/$TIMESTAMP"
RESULTS_DIR="$BENCHMARK_SESSION_DIR/results"
TEMP_DIR="$BENCHMARK_SESSION_DIR/temp"
RESULT_FILE="$RESULTS_DIR/benchmark.json"
LOG_FILE="$TEMP_DIR/benchmark.log"
SUMMARY_FILE="$RESULTS_DIR/benchmark_summary.md"

# Create results directory
mkdir -p "$RESULTS_DIR"
mkdir -p "$TEMP_DIR"

echo -e "${BLUE}=== MetaSSR Benchmark Suite ===${NC}"
echo "Starting comprehensive performance testing..."
echo "Session directory: $BENCHMARK_SESSION_DIR"
echo "Results will be saved to: $RESULT_FILE"
echo "Logs will be saved to: $LOG_FILE"

# Initialize results JSON
cat > "$RESULT_FILE" << EOF
{
  "metadata": {
    "timestamp": "$(date -Iseconds)",
    "hostname": "$(hostname)",
    "os": "$(uname -s)",
    "arch": "$(uname -m)",
    "cpu_cores": $(nproc),
    "memory_gb": $(free -g | awk '/^Mem:/{print $2}')
  },
  "tests": []
}
EOF

# Function to run benchmark and parse results
run_benchmark() {
    local test_name="$1"
    local threads="$2"
    local connections="$3"
    local duration="$4"
    local url="$5"
    
    echo -e "${YELLOW}Running $test_name...${NC}"
    echo "Threads: $threads, Connections: $connections, Duration: ${duration}s"
    
    # Run wrk and capture output
    local wrk_output=$(wrk -t"$threads" -c"$connections" -d"${duration}s" --latency "$url" 2>&1)
    
    # Parse wrk output
    local requests_per_sec=$(echo "$wrk_output" | grep "Requests/sec:" | awk '{print $2}')
    local transfer_per_sec=$(echo "$wrk_output" | grep "Transfer/sec:" | awk '{print $2}')
    local avg_latency=$(echo "$wrk_output" | grep "Latency" | head -1 | awk '{print $2}')
    local total_requests=$(echo "$wrk_output" | grep "requests in" | awk '{print $1}')
    local total_errors=$(echo "$wrk_output" | grep "Socket errors:" | awk '{print $3}' || echo "0")
    
    # Extract percentile latencies
    local latency_50=$(echo "$wrk_output" | grep "50%" | awk '{print $2}')
    local latency_75=$(echo "$wrk_output" | grep "75%" | awk '{print $2}')
    local latency_90=$(echo "$wrk_output" | grep "90%" | awk '{print $2}')
    local latency_99=$(echo "$wrk_output" | grep "99%" | awk '{print $2}')
    
    # Add to results JSON
    local test_json=$(cat << EOF
{
  "name": "$test_name",
  "config": {
    "threads": $threads,
    "connections": $connections,
    "duration": $duration,
    "url": "$url"
  },
  "results": {
    "requests_per_sec": "$requests_per_sec",
    "transfer_per_sec": "$transfer_per_sec",
    "avg_latency": "$avg_latency",
    "total_requests": "$total_requests",
    "total_errors": "$total_errors",
    "latency_percentiles": {
      "p50": "$latency_50",
      "p75": "$latency_75",
      "p90": "$latency_90",
      "p99": "$latency_99"
    }
  },
  "raw_output": $(echo "$wrk_output" | jq -Rs .)
}
EOF
)
    
    # Update results file
    jq ".tests += [$test_json]" "$RESULT_FILE" > "${RESULT_FILE}.tmp" && mv "${RESULT_FILE}.tmp" "$RESULT_FILE"
    
    # Log output
    echo "=== $test_name ===" >> "$LOG_FILE"
    echo "$wrk_output" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
    
    echo -e "${GREEN}✓ $test_name completed${NC}"
    echo "  Requests/sec: $requests_per_sec"
    echo "  Avg Latency: $avg_latency"
    echo ""
}

# Function to check server health
check_server() {
    local url="$1"
    local max_attempts=30
    local attempt=1
    
    echo -e "${YELLOW}Checking server health at $url...${NC}"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s --max-time 5 "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Server is responding${NC}"
            return 0
        fi
        echo -n "."
        sleep 1
        attempt=$((attempt + 1))
    done
    
    echo -e "\n${RED}✗ Server is not responding after $max_attempts attempts${NC}"
    return 1
}

# Function to warm up server
warmup_server() {
    local url="$1"
    echo -e "${YELLOW}Warming up server...${NC}"
    
    for i in {1..10}; do
        curl -s "$url" > /dev/null 2>&1 || true
        sleep 0.5
    done
    
    echo -e "${GREEN}✓ Server warmed up${NC}"
}

# Function to monitor system resources
monitor_resources() {
    local pid="$1"
    local duration="$2"
    local output_file="$3"
    
    echo "timestamp,cpu_percent,memory_mb,memory_percent" > "$output_file"
    
    for i in $(seq 1 $duration); do
        local cpu=$(ps -p "$pid" -o %cpu --no-headers 2>/dev/null || echo "0")
        local memory_kb=$(ps -p "$pid" -o rss --no-headers 2>/dev/null || echo "0")
        local memory_mb=$((memory_kb / 1024))
        local memory_percent=$(ps -p "$pid" -o %mem --no-headers 2>/dev/null || echo "0")
        
        echo "$i,$cpu,$memory_mb,$memory_percent" >> "$output_file"
        sleep 1
    done
}

# Main execution
main() {
    local server_url="http://localhost:8080"
    local server_pid=""
    
    echo -e "${BLUE}Starting MetaSSR server...${NC}"
    
    # Check if server is already running
    if ! check_server "$server_url" 2>/dev/null; then
        echo "Server not running, please start MetaSSR server first"
        exit 1
    fi
    
    # Get server PID for monitoring
    server_pid=$(lsof -ti:8080 || echo "")
    
    # Warm up server
    warmup_server "$server_url"
    
    # Start resource monitoring in background
    if [ -n "$server_pid" ]; then
        monitor_resources "$server_pid" 300 "$TEMP_DIR/resources.csv" &
        monitor_pid=$!
    fi
    
    # Run benchmark suite
    echo -e "${BLUE}Starting benchmark tests...${NC}"
    
    # Light load test
    run_benchmark "Light Load" 1 10 30 "$server_url"
    
    # Medium load test
    run_benchmark "Medium Load" 4 50 30 "$server_url"
    
    # Standard load test
    run_benchmark "Standard Load" 8 100 30 "$server_url"
    
    # Heavy load test
    run_benchmark "Heavy Load" 12 500 30 "$server_url"
    
    # Extreme load test
    run_benchmark "Extreme Load" 16 1000 30 "$server_url"
    
    # Sustained load test
    run_benchmark "Sustained Load" 8 200 120 "$server_url"
    
    # Endurance test
    run_benchmark "Endurance Test" 4 100 300 "$server_url"
    
    # Stop resource monitoring
    if [ -n "$monitor_pid" ]; then
        kill "$monitor_pid" 2>/dev/null || true
    fi
    
    echo -e "${GREEN}=== Benchmark Suite Completed ===${NC}"
    echo "Results saved to: $RESULT_FILE"
    echo "Logs saved to: $LOG_FILE"
    
    # Generate summary
    generate_summary
}

# Function to generate benchmark summary
generate_summary() {
    local summary_file="$RESULTS_DIR/benchmark_summary.md"
    
    echo "Generating comprehensive summary..."
    
    cat > "$summary_file" << 'EOF'
# MetaSSR Benchmark Report

**Generated:** $(date)
**Session:** $TIMESTAMP

## System Information

- **Hostname:** $(hostname)
- **OS:** $(uname -s) $(uname -r)
- **Architecture:** $(uname -m)
- **CPU Cores:** $(nproc)
- **Memory:** $(free -h | awk '/^Mem:/{print $2}')
- **Load Average:** $(uptime | awk -F'load average:' '{print $2}')

## Performance Overview

EOF

    # Extract performance data for charts
    local test_names=()
    local rps_values=()
    local latency_values=()
    
    # Parse JSON results to get data for charts
    while IFS= read -r line; do
        test_name=$(echo "$line" | jq -r '.name')
        rps=$(echo "$line" | jq -r '.results.requests_per_sec // "0"' | sed 's/,//g')
        latency=$(echo "$line" | jq -r '.results.avg_latency // "0"')
        
        test_names+=("$test_name")
        rps_values+=("$rps")
        latency_values+=("$latency")
    done < <(jq -c '.tests[]' "$RESULT_FILE")
    
    # Generate Mermaid charts
    cat >> "$summary_file" << 'EOF'

### Performance Charts

```mermaid
%%{init: {'theme':'base', 'themeVariables': {'primaryColor': '#f9f9f9'}}}%%
xychart-beta
    title "Requests per Second by Test Scenario"
    x-axis [EOF
    
    # Add test names to chart
    for i in "${!test_names[@]}"; do
        if [ $i -eq 0 ]; then
            echo -n "\"${test_names[$i]}\"" >> "$summary_file"
        else
            echo -n ", \"${test_names[$i]}\"" >> "$summary_file"
        fi
    done
    
    echo "]" >> "$summary_file"
    
    # Find max RPS for y-axis scaling
    local max_rps=0
    for rps in "${rps_values[@]}"; do
        if (( $(echo "$rps > $max_rps" | bc -l) )); then
            max_rps=$rps
        fi
    done
    
    echo "    y-axis \"RPS\" 0 --> ${max_rps%.*}" >> "$summary_file"
    echo -n "    bar [" >> "$summary_file"
    
    # Add RPS values
    for i in "${!rps_values[@]}"; do
        if [ $i -eq 0 ]; then
            echo -n "${rps_values[$i]%.*}" >> "$summary_file"
        else
            echo -n ", ${rps_values[$i]%.*}" >> "$summary_file"
        fi
    done
    
    cat >> "$summary_file" << 'EOF'
]
```

```mermaid
%%{init: {'theme':'base', 'themeVariables': {'primaryColor': '#f9f9f9'}}}%%
pie title Performance Distribution
EOF

    # Calculate performance distribution
    local excellent=0 high=0 good=0 fair=0 low=0
    
    for rps in "${rps_values[@]}"; do
        rps_num=${rps%.*}
        if [ "$rps_num" -ge 50000 ]; then
            ((excellent++))
        elif [ "$rps_num" -ge 20000 ]; then
            ((high++))
        elif [ "$rps_num" -ge 10000 ]; then
            ((good++))
        elif [ "$rps_num" -ge 5000 ]; then
            ((fair++))
        else
            ((low++))
        fi
    done
    
    # Add performance distribution to pie chart
    [ $excellent -gt 0 ] && echo "    \"EXCELLENT\" : $excellent" >> "$summary_file"
    [ $high -gt 0 ] && echo "    \"HIGH\" : $high" >> "$summary_file"
    [ $good -gt 0 ] && echo "    \"GOOD\" : $good" >> "$summary_file"
    [ $fair -gt 0 ] && echo "    \"FAIR\" : $fair" >> "$summary_file"
    [ $low -gt 0 ] && echo "    \"LOW\" : $low" >> "$summary_file"
    
    echo '```' >> "$summary_file"
    
    # Add detailed results table
    cat >> "$summary_file" << 'EOF'

## Detailed Results

| Test Scenario | RPS | Avg Latency | P99 Latency | Errors | Total Requests |
|---------------|-----|-------------|-------------|--------|----------------|
EOF
    
    # Parse and format results
    jq -r '.tests[] | "| \(.name) | \(.results.requests_per_sec // "N/A") | \(.results.avg_latency // "N/A") | \(.results.latency_percentiles.p99 // "N/A") | \(.results.total_errors // "0") | \(.results.total_requests // "N/A") |"' "$RESULT_FILE" >> "$summary_file"
    
    # Add performance insights
    cat >> "$summary_file" << 'EOF'

## Performance Insights

EOF

    # Check for errors and high performers
    local error_tests=$(jq -r '.tests[] | select((.results.total_errors // "0") != "0") | .name' "$RESULT_FILE")
    local best_test=$(jq -r '.tests | max_by(.results.requests_per_sec | tonumber) | .name + " - " + .results.requests_per_sec + " RPS"' "$RESULT_FILE")
    
    if [ -n "$error_tests" ]; then
        echo "**Tests with errors:**" >> "$summary_file"
        echo "$error_tests" | while read -r test; do
            echo "- $test" >> "$summary_file"
        done
        echo "" >> "$summary_file"
    else
        echo "**All tests completed without errors**" >> "$summary_file"
        echo "" >> "$summary_file"
    fi
    
    echo "**Best Performance:** $best_test" >> "$summary_file"
    echo "" >> "$summary_file"
    
    # Add file locations
    cat >> "$summary_file" << EOF

## Files Generated

- **Raw Results:** \`$RESULT_FILE\`
- **Detailed Logs:** \`$LOG_FILE\`
- **Resource Monitoring:** \`$TEMP_DIR/resources.csv\`
- **Session Directory:** \`$BENCHMARK_SESSION_DIR\`

---
*Generated by MetaSSR Benchmark Suite*
EOF

    echo "Summary generated: $summary_file"
}

# Check dependencies
check_dependencies() {
    local deps=("wrk" "jq" "curl" "lsof")
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            echo -e "${RED}Error: $dep is not installed${NC}"
            echo "Please install required dependencies:"
            echo "  Ubuntu/Debian: sudo apt-get install wrk jq curl lsof"
            echo "  macOS: brew install wrk jq curl lsof"
            exit 1
        fi
    done
}

# Help function
show_help() {
    cat << EOF
MetaSSR Benchmark Suite

Usage: $0 [options]

Options:
  -h, --help     Show this help message
  -u, --url      Server URL (default: http://localhost:8080)
  -o, --output   Output directory (default: benchmark-results)

Examples:
  $0                           # Run full benchmark suite
  $0 -u http://localhost:3000  # Test different server
  $0 -o custom-results         # Custom output directory

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -u|--url)
            SERVER_URL="$2"
            shift 2
            ;;
        -o|--output)
            RESULTS_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Check dependencies and run
check_dependencies
main "$@"