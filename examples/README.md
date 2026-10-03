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

## devtools

A polyglot toolbox — small developer utilities, each backed by an API route
running on a different MetaSSR loader (Python, Node.js), with a TypeScript
React frontend.

### Start the example

```sh
cd examples/devtools
npm install
npm run dev
```

The dev server starts at `http://localhost:3000` and includes live reload.

### Run with Docker

Build the app image (it installs the `metassr` CLI from npm):

```sh
docker build -f examples/devtools/Dockerfile -t devtools examples/devtools
docker run --rm -p 8080:8080 devtools
```

Open `http://localhost:8080`.

### What it demonstrates

| Route | Language | Description |
|---|---|---|
| `GET/POST /api/csvconvert` | **Python** | CSV ⇄ JSON conversion (standard library) |
| `GET/POST /api/jsontools` | **Python** | JSON format / minify / validate |
| `GET/POST /api/base64` | **JavaScript** | Base64, hex, and URL encode / decode |
| `GET/POST/DELETE /api/snippets` | **JavaScript** | In-memory snippet store (Node.js loader) |
| `/`, `/csv`, `/json`, `/base64`, `/snippets` | React + TS | Tool pages that fetch from the routes above |
