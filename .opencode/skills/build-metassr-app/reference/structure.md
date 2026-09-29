# MetaSSR project structure

Reference layout of a MetaSSR app. `metassr create` scaffolds most of this;
add the rest as the app grows.

```
my-app/
├── metassr.toml          # project config
├── package.json          # app deps + scripts (react, react-dom)
├── tsconfig.json         # when using the typescript template
├── static/               # static assets, served at /static/...
└── src/
    ├── _app.tsx          # app shell: wraps every page
    ├── _head.tsx         # document <head> content (title, meta)
    ├── pages/            # routes, one file per route
    │   ├── index.tsx     #   -> /
    │   ├── home.tsx      #   -> /home
    │   ├── blog/
    │   │   ├── index.tsx #   -> /blog
    │   │   └── $article.tsx  # -> /blog/:article (dynamic route)
    │   └── _notfound.tsx #   -> 404 page
    ├── api/              # API routes, served under /api/...
    │   └── hello.js      #   -> /api/hello (GET, POST, ...)
    ├── layout/           # shared layouts
    ├── components/       # reusable components
    └── styles/           # css
```

## Routing

- `src/pages/index.tsx` renders `/`, `src/pages/home.tsx` renders `/home`.
- `$` prefixes a dynamic segment: `blog/$article.tsx` renders `/blog/:article`.
- `_notfound.tsx` renders the 404 page.
- Pages are React components (default export). SSR renders them per request; SSG pre-renders them at build time into `dist/pages/<route>/index.html`.

## API routes

- Any file in `src/api/` becomes a route under `/api/`. The file extension picks the loader (`hello.js` → Node, `hello.ts` → TypeScript, `hello.py` → Python, ...).
- Export one function per HTTP method:

```js
function GET(_req) {
    return JSON.stringify({ status: 200, body: { message: "Hello!" } });
}

function POST(req) {
    const data = typeof req === 'string' ? JSON.parse(req) : req;
    return JSON.stringify({ status: 201, body: { received: data } });
}

module.exports = { GET, POST };
```

- Handlers return a JSON string with `status` and `body`.

## metassr.toml

```toml
[build]
type = "ssr"        # ssr | ssg
out_dir = "dist"

[server]
port = 8080

[debug]
mode = "off"        # off | metacall | http | all

[dev]
hmr = true
server_port = 3000
ws_port = 3001
```

## Static files

Files under `static/` are served at `/static/...` (e.g. `static/assets/logo.png` → `/static/assets/logo.png`).