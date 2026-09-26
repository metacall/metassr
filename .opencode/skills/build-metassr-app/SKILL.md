---
name: build-metassr-app
description: Build a MetaSSR app (React + Node/TypeScript, server-side rendering) from a user's idea — ask a few intake questions, scaffold, run, and iterate. Use when the user asks to "build an app", "build me a dashboard", "create a website", or start a new MetaSSR project from scratch.
---

# Build a MetaSSR App

Turn a user's idea into a running MetaSSR app and iterate on it with them. This skill is for building apps anywhere; the metacall/metassr repo's example-gallery and container flow live in that repo's AGENTS.md, not here.

## When to use

- The user asks to build/create/make an app, site, dashboard, landing page, blog, or similar with MetaSSR.

## Intake (keep it minimal)

Ask, in order, stopping as soon as the picture is clear:

1. **Project name** — used for the project directory and `package.json`.
2. **The idea** — what the app is: main pages and features (e.g. "a dashboard with cards, a chart page, and a settings page").
3. Only if still ambiguous: **SSR vs SSG** — SSR (default) for dynamic/API-driven apps, SSG for mostly static content.

Do not front-load questions about styling, dependencies, or architecture. React is built in; add CSS, components, or small libraries only when the idea requires them.

## Loaders (languages for API routes)

- If the user explicitly names a backend language (e.g. "a Python API"), wire that language's dependencies — see `reference/loaders.md`.
- If they don't know yet: present a checkbox of the supported loaders and ask which to use.
- Get the list at runtime: `npm view metassr loaders` (fallback: `npm view @metassr/linux-x64-gnu`). If the registry lookup fails, assume `node` and `typescript`.
- The file extension of a route in `src/api/` picks the loader (`hello.js`, `hello.ts`, ...). Handlers are named `GET`/`POST`/etc. functions; see `reference/structure.md`.

## Steps

1. **Install the CLI if needed** — `npm install -g metassr` (requires Node >= 20, Linux x64 glibc). Check with `metassr --version`.
2. **Scaffold** — `metassr create <name> --template <javascript|typescript> --install`. Pick the template from the idea: TypeScript when typing helps or the user prefers it, otherwise JavaScript. If the directory exists, build the layout manually per `reference/structure.md`.
3. **Compose the app** — create the pages the idea calls for:
   - `src/pages/` — one file per route (`index.tsx` → `/`, `home.tsx` → `/home`, `blog/$article.tsx` → `/blog/:article`, `_notfound.tsx` → 404).
   - `src/api/` — API routes, one handler per HTTP method.
   - `src/_app.tsx` (app shell), `src/_head.tsx` (document `<head>`), `src/layout/` (shared layouts), `src/components/`, `src/styles/`.
   - `static/` — assets served at `/static/...`.
   - `metassr.toml` — project config (see `reference/structure.md`).
4. **Build** — `metassr build -t ssr` (or `-t ssg`). Fix errors until it builds.
5. **Run & verify** — `metassr start` (SSR, port 8080) or `metassr start --serve` (SSG), then run the smoke checks in `reference/verify.md`.
6. **Iterate** — start `metassr dev` (dev server on :3000 with live reload), apply the user's edit requests, and re-verify.

## Rules

- Commit one logical step at a time when working in a git repo (scaffold first, then app code).
- Never commit `node_modules/`, `dist/`, or `target/`.
- If the user picks a loader marked `reserved` in `npm view metassr loaders`, tell them it isn't shipped yet; offer to structure the route so the dependency can be added when it ships.
- Keep the app's dependency footprint minimal.