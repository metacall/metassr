# MetaSSR CLI reference

## Install

```sh
npm install -g metassr
```

Requires Node.js >= 20. The runtime payload is published for Linux x64 (glibc
2.38+); macOS/Windows installs fall back to a dev-mode build from source (the
`cargo run --bin metassr` scripts in `package.json`).

## Commands

| Command | Purpose |
| --- | --- |
| `metassr create <name>` | Scaffold a new project. Flags: `-t/--template javascript|typescript`, `-v/--version`, `-d/--description`, `-i/--install` (run `npm install`; non-interactive runs skip unless given), `-y/--yes` (never prompt, accept defaults). Non-interactive (stdin not a TTY): missing options fall back to defaults — template `javascript`, version `1.0.0`, default description, no install without `-i`. The project name is always required. |
| `metassr build` | Build the app. Flags: `-t/--type ssr|ssg` (default ssr), `--out-dir` (default `dist`). |
| `metassr start` | Serve the built app. `--port` (default 8080), `--serve` (serve SSG output statically). |
| `metassr dev` | Dev server with live reload (HMR). Serves on port 3000 (configurable in `metassr.toml` `[dev]`). |

## Global options

- `--root <dir>` — project root (default `.`).
- `--debug-mode <mode>` — `metacall`, `http`, or `all` for extra logs.
- `--log-file <path>` — write logs to a file.

## Examples

```sh
metassr create my-app --template typescript --install
metassr build -t ssr
metassr start --port 8080
metassr dev
metassr build -t ssg && metassr start --serve
```