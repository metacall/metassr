# Examples

Example projects built with MetaSSR.

## Contribute an example (agent-assisted)

Tell an agent (opencode, Claude Code, Codex) to "build me an app" and, when
it's running and you like it, ask to share it with the community. The agent
follows `AGENTS.md` (and the `build-metassr-app` skill) to:

- scaffold the app into `examples/<name>/`;
- give it its own `Dockerfile` with the dependencies its loaders need;
- run and smoke-test it in the container;
- fork the repo and open a PR (`example: <name>`), with your confirmation at
  every step.

Each example is a standalone app: standard layout (`src/pages`, `src/api`,
`src/_app.tsx`, `src/_head.tsx`, `static/`, `metassr.toml`, `package.json`),
its own image, and a README of its own when it demonstrates something worth
explaining.

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

Open `http://localhost:8080`. Both routes work in the container: `/api/stats` runs on MetaCall's Node runtime and `/api/sales` on the bundled Python 3.14 (numpy + pandas from `requirements.txt`).

### What it demonstrates

| Route | Language | Description |
|---|---|---|
| `GET/POST /api/sales` | **Python** | Sales analytics (numpy + pandas) loaded via MetaCall's Python runtime |
| `GET/POST /api/stats` | **JavaScript** | Stats endpoint loaded via MetaCall's Node.js runtime |
| `/` | React + TS | Frontend that fetches from both API routes and renders the data |

## data-dashboard

A data-analysis dashboard that puts both backend runtimes to work: NumPy
computes the analytics in Python, and Node renders the Open Graph image
server-side — a 1200×630 PNG drawn pixel by pixel with a pure-JS encoder (no
image library, no external service).

### Start the example

```sh
cd examples/data-dashboard
npm install
npm run dev
```

The dev server starts at `http://localhost:3000` and includes live reload.

### Run with Docker

Build the app image (it installs the `metassr` CLI from npm):

```sh
docker build -f examples/data-dashboard/Dockerfile -t data-dashboard examples/data-dashboard
docker run --rm -p 8080:8080 data-dashboard
```

Open `http://localhost:8080`. Both routes work in the container: `/api/og`
runs on MetaCall's Node runtime and `/api/analysis` on the bundled Python
3.14 (numpy from `requirements.txt`).

### What it demonstrates

| Route | Language | Description |
|---|---|---|
| `GET /api/analysis` | **Python** | Deterministic sample dataset analyzed with NumPy (KPIs, monthly/category/region series, product ranking) |
| `GET /api/og` | **JavaScript** | Renders a 1200×630 Open Graph PNG in pure Node — zlib PNG encoder + 5×7 bitmap font, `?page=` selects the card |
| `/` | React + TS | Dashboard fetching from both backends, with a live OG-image preview card |
