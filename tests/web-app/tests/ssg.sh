#!/bin/bash
set -e

BASE_URL="http://localhost:8080"
FAILED=0

assert_status() {
    local url="$1"
    local expected="$2"
    local label="$3"
    local actual
    actual=$(curl --write-out "%{http_code}" --silent --output /dev/null "$url")
    if [ "$actual" -eq "$expected" ]; then
        echo "[PASS] $label -> $actual"
    else
        echo "[FAIL] $label -> expected $expected, got $actual"
        FAILED=$((FAILED + 1))
    fi
}

assert_contains() {
    local url="$1"
    local needle="$2"
    local label="$3"
    local body
    body=$(curl --silent "$url")
    if echo "$body" | grep -q "$needle"; then
        echo "[PASS] $label contains \"$needle\""
    else
        echo "[FAIL] $label missing \"$needle\""
        echo "       Body (first 300 chars): $(echo "$body" | head -c 300)"
        FAILED=$((FAILED + 1))
    fi
}

echo "=== SSG Page Routes ==="
assert_status "$BASE_URL/"     200 "GET / (root)"
assert_status "$BASE_URL/home" 200 "GET /home"
assert_status "$BASE_URL/blog" 200 "GET /blog"

echo ""
echo "=== SSG HTML Content ==="
assert_contains "$BASE_URL/"     "Hello from index page"   "Root page content"
assert_contains "$BASE_URL/home" "counter"                 "Home page content"
assert_contains "$BASE_URL/blog" "cool blog"               "Blog page content"

echo ""
echo "=== SSG Head Rendering ==="
assert_contains "$BASE_URL/" "<title>My website</title>" "Head title"

echo ""
echo "=== SSG Client Bundles ==="
assert_contains "$BASE_URL/" "<script"  "Script tag injected"

echo ""
echo "=== SSG 404 Handling ==="
assert_status "$BASE_URL/nonexistent-route" 404 "GET /nonexistent-route returns 404"

echo ""
echo "=== SSG Static Files ==="
assert_status "$BASE_URL/static/assets/metacall-logo.png" 200 "GET static asset"

echo ""
echo "=== SSG Pre-rendered Files ==="
DIST_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)/dist"
for page in home blog _notfound; do
    if [ -f "$DIST_DIR/pages/$page/index.html" ]; then
        echo "[PASS] Pre-rendered HTML exists: pages/$page/index.html"
    else
        echo "[FAIL] Missing pre-rendered HTML: pages/$page/index.html"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
if [ "$FAILED" -eq 0 ]; then
    echo "All SSG tests passed!"
    exit 0
else
    echo "Failed SSG tests: $FAILED"
    exit 1
fi
