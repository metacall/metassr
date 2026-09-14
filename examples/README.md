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

Build the base image once, then the app image:

```sh
docker build -f docker/base.Dockerfile -t metacall/metassr:1.0.0-alpha .
docker build -f docker/app.Dockerfile -t sales-dashboard examples/sales-dashboard
docker run --rm -p 8080:8080 sales-dashboard
```

Open `http://localhost:8080`. The Python route's `numpy` and `pandas` dependencies are listed in `requirements.txt` and installed into the app image.

### What it demonstrates

| Route | Language | Description |
|---|---|---|
| `GET/POST /api/sales` | **Python** | Sales analytics (numpy + pandas) loaded via MetaCall's Python runtime |
| `GET/POST /api/stats` | **JavaScript** | Stats endpoint loaded via MetaCall's Node.js runtime |
| `/` | React + TS | Frontend that fetches from both API routes and renders the data |
