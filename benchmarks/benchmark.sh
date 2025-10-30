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
DEFAULT_RESULTS_DIR="$PROJECT_ROOT/benchmark-results"
RESULTS_DIR="${RESULTS_DIR:-$DEFAULT_RESULTS_DIR}"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULT_FILE="$RESULTS_DIR/benchmark_$TIMESTAMP.json"
LOG_FILE="$RESULTS_DIR/benchmark_$TIMESTAMP.log"

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}=== MetaSSR Benchmark Suite ===${NC}"
echo "Starting comprehensive performance testing..."
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
        monitor_resources "$server_pid" 300 "$RESULTS_DIR/resources_$TIMESTAMP.csv" &
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
    local summary_file="$RESULTS_DIR/summary_$TIMESTAMP.md"
    
    cat > "$summary_file" << EOF
# MetaSSR Benchmark Summary

**Generated:** $(date)
**Duration:** Multiple test scenarios

## System Information
- **OS:** $(uname -s) $(uname -r)
- **Architecture:** $(uname -m)
- **CPU Cores:** $(nproc)
- **Memory:** $(free -h | awk '/^Mem:/{print $2}')

## Test Results

EOF
    
    # Parse and format results
    jq -r '.tests[] | "### \(.name)\n- **Requests/sec:** \(.results.requests_per_sec)\n- **Avg Latency:** \(.results.avg_latency)\n- **Total Requests:** \(.results.total_requests)\n- **Errors:** \(.results.total_errors)\n- **P99 Latency:** \(.results.latency_percentiles.p99)\n"' "$RESULT_FILE" >> "$summary_file"
    
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