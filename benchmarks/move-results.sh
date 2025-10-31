#!/bin/bash

# Move benchmark results from tests/web-app/benchmark-results to project root
# This fixes results that were created in the wrong location

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

SOURCE_DIR="$PROJECT_ROOT/tests/web-app/benchmark-results"
TARGET_DIR="$PROJECT_ROOT/benchmark-results"

if [ -d "$SOURCE_DIR" ]; then
    echo "Found benchmark results in $SOURCE_DIR"
    echo "Moving to $TARGET_DIR..."
    
    # Create target directory if it doesn't exist
    mkdir -p "$TARGET_DIR"
    
    # Move all files
    if [ "$(ls -A "$SOURCE_DIR" 2>/dev/null)" ]; then
        mv "$SOURCE_DIR"/* "$TARGET_DIR/" 2>/dev/null || true
        echo "Files moved successfully"
        
        # Remove empty source directory
        rmdir "$SOURCE_DIR" 2>/dev/null || true
        echo "Cleanup completed"
    else
        echo "Source directory is empty"
    fi
else
    echo "No benchmark results found in tests/web-app/benchmark-results"
fi

echo "Benchmark results are now centralized in: $TARGET_DIR"