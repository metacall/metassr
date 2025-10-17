# MetaSSR Bundler Tests

This directory contains tests for the MetaSSR bundler.

## Web App Test

The `web-app` example tests the bundler with a React application including:
- TypeScript/TSX files
- CSS imports (should be embedded in the bundle)
- Image assets (should be inlined as base64)
- Multiple pages and components

### Expected Output

After running `npm run build` in the `web-app` directory, the `dist` folder should contain:

```
dist/
├── cache/
│   ├── head.js                              # Head bundle
│   ├── head.js.map                          # Head source map
│   └── pages/
│       ├── blog/
│       │   ├── $article/
│       │   │   ├── index.js                 # Client bundle
│       │   │   ├── index.server.js          # Server bundle
│       │   │   ├── index.server.js.map      # Server source map
│       │   │   ├── index.server.css         # Server CSS (separate)
│       │   │   └── index.server.css.map     # CSS source map
│       │   ├── index.js
│       │   ├── index.server.js
│       │   ├── index.server.js.map
│       │   ├── index.server.css
│       │   └── index.server.css.map
│       ├── home/                            # Similar structure for home page
│       ├── index.js                         # Root page client bundle
│       ├── index.server.js                  # Root page server bundle
│       ├── index.server.js.map
│       ├── index.server.css
│       ├── index.server.css.map
│       └── _notfound/                       # Similar structure for 404 page
├── pages/
│   ├── blog/
│   │   ├── $article/
│   │   │   ├── index.js.js                  # Client bundle
│   │   │   ├── index.js.js.map              # Client source map
│   │   │   ├── index.js.css                 # Client CSS (separate)
│   │   │   └── index.js.css.map             # CSS source map
│   │   └── (similar structure for blog index)
│   ├── home/                                # Similar structure for home page
│   ├── _notfound/                           # Similar structure for 404 page
│   └── (root page files)
└── manifest.json                            # Build manifest
```

**Note:** The build creates both client and server bundles. CSS files are currently generated separately for both client and server builds. This may be expected behavior for SSR (Server-Side Rendering).

### Running Tests Locally

1. Build the project:
```bash
cd tests/web-app
npm install
npm run build
```

2. Run the test script from anywhere in the project:
```bash
# Make the script executable
chmod +x tests/test-bundle.sh

# Run from project root
./tests/test-bundle.sh

# Or run from anywhere
/path/to/metassr/tests/test-bundle.sh
```

### Running Tests in Docker

```bash
# Build the Docker image
docker build -f Dockerfile.dev -t metassr-test .

# Run tests inside Docker
docker run --rm metassr-test bash -c "cd /root/tests/web-app && npm run build && ls -la dist/"
```

### CI/CD

The GitHub Actions workflow (`.github/workflows/test.yml`) automatically runs these tests on every push and pull request.

## Test Configuration

The test script is designed to be easily configurable. Key configuration sections:

### Expected Directories
```bash
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
```

### Expected Files
```bash
EXPECTED_FILES=(
    "manifest.json"
    "cache/head.js"
    "cache/head.js.map"
)
```

### Expected Patterns
```bash
EXPECTED_PATTERNS=(
    "cache/pages/*/index.js"
    "cache/pages/*/index.server.js"
    "cache/pages/*/index.server.js.map"
    "pages/*/index.js.js"
    "pages/*/index.js.js.map"
)
```

### CSS Embedding Check
```bash
CSS_SEPARATE_FILES_PATTERNS=(
    "pages/*/index.js.css"
    "cache/pages/*/index.server.css"
)
```

To modify the test expectations, simply update these arrays in the test script.
