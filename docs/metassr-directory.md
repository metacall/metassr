# The `.metassr` directory

`~/.metassr` holds a declared state for the MetaSSR CLI installed by npm. The npm
packages are read-only, so anything that has to be written down between runs
lives here instead.

There are two things in it:

## ~/.metassr/runtime/configurations

MetaCall reads its loader configuration from the path in the
`CONFIGURATION_PATH` environment variable. The entries are absolute paths to
files inside the installed payload, so they cannot be baked into the binary.
The [JavaScript shim](../npm/cli/bin/) writes them on every run, which only
takes a moment because they are a few kilobytes.

There are two files:

- `global.json` — points to which loader configurations to use.
- `node_loader.json` — points to the shared lib `libnode.so` and at the payload's
  `node_modules`.

Nothing in the Rust code writes these files. Before the npm package existed,
the MetaCall installer supplied equivalent files under
`/gnu/share/metacall/configurations/`.

So it's more of a declarative way instead of an imperative (cli-drived) one.

## ~/.metassr/vendor/bundler

`metassr build` bundles the application with esbuild, and esbuild has to live
somewhere the user can write to, so it is copied into the home directory.

Two separate places manage this directory:

1. The JavaScript shim (during installing MetaSSR via npm) copies `vendor/bundler` out of the payload when the
   `package.json` there differs. This is the normal path and it works without
   network access.
2. `crates/metassr-bundler/src/vendor.rs` checks the same directory before
   building. If `node_modules/esbuild` is missing or the version does not
   match, it runs `npm install`.

The Rust check is a fallback for when the payload copy is missing. Normally the
shim has already populated the directory, so the Rust code finds everything it
needs and does nothing.

## Removing it

Uninstalling the npm packages does not remove `~/.metassr`. Deleting it by hand
is safe; the shim recreates what it needs on the next run.
