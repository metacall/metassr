#!/bin/bash

# Test script for MetaSSR bundler
# This script verifies the bundler output

set -e

# Get the script directory and project root
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
WEB_APP_DIR=$(dirname "$SCRIPT_DIR")
DIST_DIR="$WEB_APP_DIR/dist"

# Configuration: Expected files and directories
EXPECTED_DIRECTORIES=(
    "cache"
    "cache/pages"
    "cache/pages/blog"
    "cache/pages/home"
    "cache/pages/_notfound"
    "pages"
    "pages/blog"
    "pages/home"
    "pages/_notfound"
)

EXPECTED_FILES=(
    "manifest.json"
    "cache/head.js"
    "cache/head.js.map"
)

# Expected patterns for generated files
EXPECTED_PATTERNS=(
    "cache/pages/*/index.js"
    "cache/pages/*/index.server.js"
    "cache/pages/*/index.server.js.map"
    "pages/*/index.js.js"
    "pages/*/index.js.js.map"
)

# CSS embedding test configuration
CSS_SEPARATE_FILES_PATTERNS=(
    "pages/*/index.js.css"
    "cache/pages/*/index.server.css"
)

# Minimum file sizes (in bytes)
MIN_BUNDLE_SIZE=500
MIN_CSS_SIZE=100

echo "Running MetaSSR bundler tests..."
echo "[INFO] Web app directory: $WEB_APP_DIR"
echo "[INFO] Dist directory: $DIST_DIR"

# Function to check if a file/directory exists
check_exists() {
    local path="$1"
    local type="$2"  # "file" or "directory"
    
    if [ "$type" = "directory" ] && [ -d "$path" ]; then
        echo "[PASS] Found directory: $(basename "$path")"
        return 0
    elif [ "$type" = "file" ] && [ -f "$path" ]; then
        echo "[PASS] Found file: $(basename "$path")"
        return 0
    else
        echo "[ERROR] Missing $type: $path"
        return 1
    fi
}

# Function to check file patterns using find
check_pattern() {
    local pattern="$1"
    local description="$2"
    
    # Convert glob pattern to find pattern
    local find_pattern="${pattern//\*/*}"
    local found_files
    found_files=$(find "$DIST_DIR" -path "$DIST_DIR/$find_pattern" 2>/dev/null | wc -l)
    
    if [ "$found_files" -gt 0 ]; then
        echo "[PASS] Found $found_files files matching pattern: $description"
        return 0
    else
        echo "[ERROR] No files found matching pattern: $pattern"
        return 1
    fi
}

# Function to check file size
check_file_size() {
    local file="$1"
    local min_size="$2"
    local description="$3"
    
    if [ ! -f "$file" ]; then
        echo "[ERROR] File not found for size check: $file"
        return 1
    fi
    
    local size
    size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null)
    
    if [ "$size" -ge "$min_size" ]; then
        echo "[PASS] $description size: $size bytes (>= $min_size)"
        return 0
    else
        echo "[ERROR] $description size too small: $size bytes (< $min_size)"
        return 1
    fi
}

# Function to validate JavaScript syntax
validate_js() {
    local file="$1"
    
    if [ ! -f "$file" ]; then
        echo "[SKIP] JavaScript validation (file not found): $file"
        return 0
    fi
    
    if node -c "$file" 2>/dev/null; then
        echo "[PASS] Valid JavaScript: $(basename "$file")"
        return 0
    else
        echo "[WARN] Invalid JavaScript syntax: $file"
        return 1
    fi
}

# Main test execution
FAILED_TESTS=0

# Check if web-app directory exists
if ! check_exists "$WEB_APP_DIR" "directory"; then
    exit 1
fi

# Check if dist directory exists
if ! check_exists "$DIST_DIR" "directory"; then
    echo "[INFO] Available directories in web-app:"
    ls -la "$WEB_APP_DIR"
    exit 1
fi

echo ""
echo "=== Checking expected directories ==="
for dir in "${EXPECTED_DIRECTORIES[@]}"; do
    if ! check_exists "$DIST_DIR/$dir" "directory"; then
        ((FAILED_TESTS++))
    fi
done

echo ""
echo "=== Checking expected files ==="
for file in "${EXPECTED_FILES[@]}"; do
    if ! check_exists "$DIST_DIR/$file" "file"; then
        ((FAILED_TESTS++))
    fi
done

echo ""
echo "=== Checking file patterns ==="
for pattern in "${EXPECTED_PATTERNS[@]}"; do
    if ! check_pattern "$pattern" "$pattern"; then
        ((FAILED_TESTS++))
    fi
done

echo ""
echo "=== Checking CSS embedding ==="
CSS_SEPARATE_COUNT=0
for pattern in "${CSS_SEPARATE_FILES_PATTERNS[@]}"; do
    find_pattern="${pattern//\*/*}"
    found_files=$(find "$DIST_DIR" -path "$DIST_DIR/$find_pattern" 2>/dev/null | wc -l)
    CSS_SEPARATE_COUNT=$((CSS_SEPARATE_COUNT + found_files))
done

if [ "$CSS_SEPARATE_COUNT" -gt 0 ]; then
    echo "[WARN] Found $CSS_SEPARATE_COUNT separate CSS files. CSS should be embedded in JS bundles."
    echo "[INFO] This might be expected behavior for server-side rendering."
else
    echo "[PASS] No separate CSS files found - CSS is embedded"
fi

echo ""
echo "=== Checking file sizes ==="
# Check sizes of key files
if [ -f "$DIST_DIR/cache/head.js" ]; then
    check_file_size "$DIST_DIR/cache/head.js" "$MIN_BUNDLE_SIZE" "Head bundle"
fi

# Check first page bundle
FIRST_PAGE_JS=$(find "$DIST_DIR/pages" -name "index.js.js" | head -1)
if [ -n "$FIRST_PAGE_JS" ]; then
    check_file_size "$FIRST_PAGE_JS" "$MIN_BUNDLE_SIZE" "Page bundle"
fi

echo ""
echo "=== Validating JavaScript syntax ==="
# Validate key JavaScript files
find "$DIST_DIR" -name "*.js" -not -name "*.js.map" | head -5 | while read -r js_file; do
    validate_js "$js_file"
done

echo ""
echo "=== Dist directory structure ==="
echo "Contents of dist directory:"
find "$DIST_DIR" -type f | sort | while read -r file; do
    rel_path="${file#$DIST_DIR/}"
    size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null)
    printf "  %-50s %8s bytes\n" "$rel_path" "$size"
done

echo ""
if [ "$FAILED_TESTS" -eq 0 ]; then
    echo "All tests passed!"
    exit 0
else
    echo "Failed tests: $FAILED_TESTS"
    exit 1
fi
