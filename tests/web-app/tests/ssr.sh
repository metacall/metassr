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

assert_not_contains() {
    local url="$1"
    local needle="$2"
    local label="$3"
    local body
    body=$(curl --silent "$url")
    if echo "$body" | grep -q "$needle"; then
        echo "[FAIL] $label should not contain \"$needle\""
        FAILED=$((FAILED + 1))
    else
        echo "[PASS] $label does not contain \"$needle\""
    fi
}

assert_json_field() {
    local url="$1"
    local method="$2"
    local field="$3"
    local expected="$4"
    local label="$5"
    local body
    if [ "$method" = "POST" ]; then
        body=$(curl --silent -X POST -H "Content-Type: application/json" -d '{"name":"world"}' "$url")
    else
        body=$(curl --silent "$url")
    fi
    if echo "$body" | grep -q "$expected"; then
        echo "[PASS] $label"
    else
        echo "[FAIL] $label -> expected \"$expected\" in response"
        echo "       Body: $body"
        FAILED=$((FAILED + 1))
    fi
}

echo "=== SSR Page Routes ==="
assert_status "$BASE_URL/"     200 "GET / (root)"
assert_status "$BASE_URL/home" 200 "GET /home"
assert_status "$BASE_URL/blog" 200 "GET /blog"

echo ""
echo "=== SSR HTML Content ==="
assert_contains "$BASE_URL/"     "Hello from index page"   "Root page content"
assert_contains "$BASE_URL/home" "counter"                 "Home page content"
assert_contains "$BASE_URL/blog" "cool blog"               "Blog page content"

echo ""
echo "=== Head Rendering ==="
assert_contains "$BASE_URL/" "<title>My website</title>" "Head title"
assert_contains "$BASE_URL/" "charSet"                     "Head meta charset"

echo ""
echo "=== Client Bundles ==="
assert_contains "$BASE_URL/" "<script"  "Script tag injected"

echo ""
echo "=== 404 Handling ==="
assert_status "$BASE_URL/_notfound" 200 "GET /_notfound page exists"
assert_status "$BASE_URL/nonexistent-route" 303 "GET /nonexistent-route redirects"

echo ""
echo "=== Static Files ==="
assert_status "$BASE_URL/static/assets/metacall-logo.png" 200 "GET static asset"

echo ""
echo "=== API Endpoints ==="
assert_status "$BASE_URL/api/hello" 200 "GET /api/hello"
assert_json_field "$BASE_URL/api/hello" "GET"  "message" "Hello from MetaSSR API" "API GET response"
assert_json_field "$BASE_URL/api/hello" "POST" "message" "Hello, world!"           "API POST response"

echo ""
if [ "$FAILED" -eq 0 ]; then
    echo "All SSR tests passed!"
    exit 0
else
    echo "Failed SSR tests: $FAILED"
    exit 1
fi
