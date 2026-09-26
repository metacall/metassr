# Examples

Example projects built with MetaSSR.

## sales-dashboard

A polyglot fullstack app demonstrating MetaSSR's API handler with Python and JavaScript backend routes and a React frontend.

### Prerequisites

- Rust toolchain (for building MetaSSR CLI)
- Node.js + npm
- Python 3

### Start the example

```sh
cd examples/sales-dashboard
npm install
npm run dev
```

The dev server starts at `http://localhost:3000` and includes live reload.

### Run with Docker

Build the app image (it installs the `metassr` CLI from npm):

```sh
docker build -f docker/app.Dockerfile -t sales-dashboard examples/sales-dashboard
docker run --rm -p 8080:8080 sales-dashboard
```

Open `http://localhost:8080`. The JavaScript `/api/stats` route works in the container. The Python `/api/sales` route needs MetaCall's Python runtime, which the npm `metassr` payload does not bundle yet, so it only runs with a local install (`npm run dev`).

### What it demonstrates

| Route | Language | Description |
|---|---|---|
| `GET/POST /api/sales` | **Python** | Sales analytics (numpy + pandas) loaded via MetaCall's Python runtime |
| `GET/POST /api/stats` | **JavaScript** | Stats endpoint loaded via MetaCall's Node.js runtime |
| `/` | React + TS | Frontend that fetches from both API routes and renders the data |
