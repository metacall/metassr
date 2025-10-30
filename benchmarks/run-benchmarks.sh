#!/bin/bash

# MetaSSR Automated Benchmark Runner
# Orchestrates the complete benchmarking process

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCHMARK_SCRIPT="$SCRIPT_DIR/benchmark.sh"
ANALYZER_SCRIPT="$SCRIPT_DIR/analyze-benchmarks.py"
CONFIG_FILE="$SCRIPT_DIR/benchmark-config.json"

# Default values
SERVER_PORT=8080
BUILD_TYPE="release"
ANALYZE_RESULTS=true
GENERATE_PLOTS=false
OUTPUT_DIR="benchmark-results"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

usage() {
    cat << EOF
MetaSSR Automated Benchmark Runner

Usage: $0 [options]

Options:
    -p, --port PORT        Server port (default: 8080)
    -b, --build TYPE       Build type: debug|release (default: release)
    -o, --output DIR       Output directory (default: benchmark-results)
    -a, --analyze          Analyze results after benchmarking (default: true)
    -g, --graphs           Generate performance graphs
    -s, --skip-build       Skip building the project
    -h, --help             Show this help

Examples:
    $0                     # Full benchmark with default settings
    $0 -p 3000 -b debug    # Debug build on port 3000
    $0 -g                  # Include performance graphs
    $0 -s                  # Skip build step

EOF
}

log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

check_dependencies() {
    local deps=("cargo" "npm" "wrk" "jq" "curl" "lsof")
    local missing=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing+=("$dep")
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        error "Missing dependencies: ${missing[*]}"
        echo "Please install the missing dependencies and try again."
        exit 1
    fi
}

build_project() {
    log "Building MetaSSR project..."
    cd "$PROJECT_ROOT"
    
    if [ "$BUILD_TYPE" = "release" ]; then
        cargo build --release
    else
        cargo build
    fi
    
    success "Project built successfully"
}

setup_test_app() {
    log "Setting up test application..."
    cd "$PROJECT_ROOT/tests/web-app"
    
    if [ ! -f "package.json" ]; then
        error "Test app not found at tests/web-app"
        exit 1
    fi
    
    npm install
    
    if [ "$BUILD_TYPE" = "release" ]; then
        npm run build
    else
        npm run build:debug
    fi
    
    success "Test application ready"
}

start_server() {
    log "Starting MetaSSR server on port $SERVER_PORT..."
    cd "$PROJECT_ROOT/tests/web-app"
    
    # Check if server is already running
    if curl -s "http://localhost:$SERVER_PORT" > /dev/null 2>&1; then
        warn "Server already running on port $SERVER_PORT, using existing instance"
        return 0
    fi
    
    if [ "$BUILD_TYPE" = "release" ]; then
        npm start &
    else
        npm run start:debug &
    fi
    
    SERVER_PID=$!
    echo $SERVER_PID > metassr_benchmark.pid
    
    # Wait for server to start
    local max_wait=30
    local count=0
    
    while ! curl -s "http://localhost:$SERVER_PORT" > /dev/null 2>&1; do
        if [ $count -ge $max_wait ]; then
            error "Server failed to start within $max_wait seconds"
            cleanup
            exit 1
        fi
        sleep 1
        count=$((count + 1))
        echo -n "."
    done
    
    echo ""
    success "Server started successfully (PID: $SERVER_PID)"
}

stop_server() {
    log "Stopping MetaSSR server..."
    cd "$PROJECT_ROOT/tests/web-app"
    
    # Only stop server if we started it
    if [ -f "metassr_benchmark.pid" ]; then
        local pid=$(cat metassr_benchmark.pid)
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid"
            # Wait for graceful shutdown
            local count=0
            while kill -0 "$pid" 2>/dev/null && [ $count -lt 10 ]; do
                sleep 1
                count=$((count + 1))
            done
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid"
            fi
        fi
        rm metassr_benchmark.pid
        success "Server stopped"
    else
        log "No server PID file found, server was not started by this script"
    fi
}

run_benchmarks() {
    log "Running benchmark suite..."
    
    # Set absolute path for output directory
    local abs_output_dir="$PROJECT_ROOT/$OUTPUT_DIR"
    
    # Ensure output directory exists
    mkdir -p "$abs_output_dir"
    
    # Run the benchmark script with absolute path
    RESULTS_DIR="$abs_output_dir" "$BENCHMARK_SCRIPT" -u "http://localhost:$SERVER_PORT" -o "$abs_output_dir"
    
    success "Benchmarks completed"
}

analyze_results() {
    if [ "$ANALYZE_RESULTS" = false ]; then
        return
    fi
    
    log "Analyzing benchmark results..."
    
    # Use absolute path for results
    local abs_output_dir="$PROJECT_ROOT/$OUTPUT_DIR"
    
    # Find the latest results file
    local latest_result=$(ls -t "$abs_output_dir"/benchmark_*.json 2>/dev/null | head -1)
    
    if [ -z "$latest_result" ]; then
        warn "No benchmark results found to analyze"
        return
    fi
    
    # Create analysis directory
    local analysis_dir="$abs_output_dir/analysis_$(date +%Y%m%d_%H%M%S)"
    
    # Run analyzer
    local analyzer_args=("$latest_result" "-o" "$analysis_dir")
    
    if [ "$GENERATE_PLOTS" = true ]; then
        analyzer_args+=("--plots")
    fi
    
    if command -v python3 &> /dev/null; then
        python3 "$ANALYZER_SCRIPT" "${analyzer_args[@]}"
        success "Analysis completed: $analysis_dir"
    else
        warn "Python3 not available, skipping analysis"
    fi
}

cleanup() {
    log "Cleaning up..."
    stop_server
    
    # Kill any remaining processes
    pkill -f "npm.*start" 2>/dev/null || true
    
    success "Cleanup completed"
}

# Signal handlers
trap cleanup EXIT
trap 'error "Interrupted"; exit 130' INT TERM

main() {
    log "Starting MetaSSR automated benchmark..."
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -p|--port)
                SERVER_PORT="$2"
                shift 2
                ;;
            -b|--build)
                BUILD_TYPE="$2"
                shift 2
                ;;
            -o|--output)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            -a|--analyze)
                ANALYZE_RESULTS=true
                shift
                ;;
            -g|--graphs)
                GENERATE_PLOTS=true
                shift
                ;;
            -s|--skip-build)
                SKIP_BUILD=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    # Validate build type
    if [[ "$BUILD_TYPE" != "debug" && "$BUILD_TYPE" != "release" ]]; then
        error "Invalid build type: $BUILD_TYPE. Must be 'debug' or 'release'"
        exit 1
    fi
    
    # Check dependencies
    check_dependencies
    
    # Build project if not skipped
    if [ "$SKIP_BUILD" != true ]; then
        build_project
        setup_test_app
    fi
    
    # Start server
    start_server
    
    # Run benchmarks
    run_benchmarks
    
    # Stop server
    stop_server
    
    # Analyze results
    analyze_results
    
    success "Automated benchmark completed successfully!"
    echo ""
    echo "Results location: $PROJECT_ROOT/$OUTPUT_DIR"
    
    if [ "$ANALYZE_RESULTS" = true ]; then
        echo "Analysis reports generated in: $PROJECT_ROOT/$OUTPUT_DIR/analysis_*"
    fi
}

# Check if script is being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi