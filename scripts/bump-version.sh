#!/bin/bash

if [ $# -ne 1 ]; then
    echo "Usage: $0 <new-version>"
    exit 1
fi

NEW_VERSION=$1

update_version() {
    local file=$1
    echo "Updating version in $file to $NEW_VERSION"
    # Use `sed` to find and replace the version in the Cargo.toml file
    sed -i -E "s/^version = \"[^\"]+\"/version = \"$NEW_VERSION\"/" "$file"
}

# Find all Cargo.toml files and update their versions
find . -name 'Cargo.toml' -print0 | while IFS= read -r -d '' file; do
    update_version "$file"
done

echo "Version update completed."
