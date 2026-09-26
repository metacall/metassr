# AGENTS.md

Guide for AI agents (opencode, Claude Code, Codex) working in this repository.

## Repository orientation

| Path | What it is |
| --- | --- |
| `crates/` | Rust workspace: `metassr-cli`, `metassr-build`, `metassr-bundler`, `metassr-create`, ... |
| `metassr-cli/` | CLI entry point |
| `npm/` | Published npm packages: `metassr` CLI + `@metassr/linux-x64-gnu` runtime payload |
| `docker/` | `app.Dockerfile` — the generic app image |
| `examples/` | Community example apps (the gallery) |
| `tests/` | Test apps + smoke scripts (`tests/web-app/tests/`) |
| `docs/` | User documentation |
| `.opencode/skills/build-metassr-app/` | The "build a MetaSSR app" skill (see SKILLS.md) |

The skill directory is **symlinked** into `.claude/skills/build-metassr-app`
and `.codex/skills/build-metassr-app`. The symlinks are committed and must
stay symlinks — never replace them with copies.

## Building an app

When the user asks to build an app ("build me a dashboard with cards"),
follow the `build-metassr-app` skill for the app itself (intake questions,
scaffold, build, run, iterate). Then, if the user wants it shared with the
community, bring it into the example gallery with the flow below.

## The example-gallery loop (generate → verify → PR)

1. **Intake** — project name + idea. If the user does not name a backend
   language, show a checkbox of the supported loaders (`npm view metassr
   loaders`) and ask. If they do name one, wire its dependencies per the
   table below.
2. **Scaffold** — create the app inside `examples/<name>/` following the
   skill (standard layout: `src/pages`, `src/api`, `src/_app.tsx`,
   `src/_head.tsx`, `static/`, `metassr.toml`, `package.json`).
3. **Own app image** — write `examples/<name>/Dockerfile`: the example's
   own image, based on `docker/app.Dockerfile`, plus the dependencies its
   loaders need (see table). **Ask the user before building images or
   running containers.**
4. **Run & test in the container** — build the image, run it, and smoke-test
   inside it: `curl` the page routes, API routes, static assets and the 404
   path (mirror `tests/web-app/tests/ssr.sh` / `ssg.sh`).
5. **Iterate** — apply the user's edits, rebuild, re-test in the container.
6. **Share** — fully automated fork + PR (below), always with the user's
   confirmation at each step.

## Dependencies per app type

Each loader brings its own dependency file and runtime for the app image:

| Loader | Route extension | Dependencies file | App image adds | Status |
| --- | --- | --- | --- | --- |
| `node` | `.js` | `package.json` | nothing extra | shipped |
| `typescript` | `.ts` | `package.json` | nothing extra | shipped |
| `python` | `.py` | `requirements.txt` | `python3` + `python3-pip`; `pip install -r requirements.txt` | reserved |
| `ruby` | `.rb` | `Gemfile` | ruby; `bundle install` | reserved |
| `rust` | `.rs` | `Cargo.toml` | rust toolchain; `cargo build` | reserved |
| `go` | `.go` | `go.mod` | golang; `go build` | reserved |

- **Shipped** loaders (currently `node`, `typescript`) run in the npm payload
  and in the generic app image as-is.
- **Reserved** loaders are not in the npm payload yet: their routes will not
  run in containers until the loader ships. Say so, and keep the dependency
  wiring in place so the example works the moment it does.

The runtime payload currently ships only the Node and TypeScript loaders —
confirm with `npm view metassr loaders`.

## Example app image

`examples/<name>/Dockerfile` is a standalone image for that example: start
from the `docker/app.Dockerfile` pattern (`ARG METASSR_VERSION` +
`BUILD_TYPE`; build stage: `npm install` + `metassr build`; runtime stage:
serve), then add the loader dependencies from the table above. Build and test
with the example directory as context:

```sh
docker build -f examples/<name>/Dockerfile -t metassr-example-<name> examples/<name>
docker run -d --name example-<name> -p 8080:8080 metassr-example-<name>
# smoke-test: page routes, API routes, static assets, 404 handling
```

## Share flow (opt-in — always ask first)

Ask the user before **each** of these steps:

1. Ensure `gh` is installed and authenticated (`gh auth status`).
2. Fork: `gh repo fork metacall/metassr --remote` (reuse an existing fork if
   present).
3. Create a branch `examples/<name>`.
4. Copy the example in; never commit `node_modules/`, `dist/`, or `target/`.
5. Re-verify locally (build + container smoke test).
6. Open the PR: `gh pr create --base master` with a title like
   `example: <name>` and a short description (match the style of existing
   PRs in this repo).

## Constraints

- Container images are linux/amd64; the npm runtime payload requires glibc
  2.38+ (Debian 13 trixie, Ubuntu 24.04+).
- The CLI requires Node >= 20.
- `npm view metassr loaders` is the source of truth for what runs out of the
  box.