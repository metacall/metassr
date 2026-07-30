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

### What it demonstrates

| Route | Language | Description |
|---|---|---|
| `GET/POST /api/sales` | **Python** | Hello-world API handler loaded via MetaCall's Python runtime |
| `GET/POST /api/stats` | **JavaScript** | Hello-world API handler loaded via MetaCall's Node.js runtime |
| `/` | React + TS | Frontend that fetches from both API routes and displays responses |
