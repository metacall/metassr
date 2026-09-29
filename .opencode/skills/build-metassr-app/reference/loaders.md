# Loaders (API route languages)

MetaSSR API routes are polyglot: the file extension of a route in `src/api/`
selects the MetaCall loader that runs it.

## Checking what is shipped

```sh
npm view metassr loaders
```

If the registry lookup fails, assume `node` and `typescript`.

## Loader table

| Loader | Route extension | Dependencies file | Status |
| --- | --- | --- | --- |
| `node` | `.js` | `package.json` | shipped |
| `typescript` | `.ts` | `package.json` | shipped |
| `python` | `.py` | `requirements.txt` | shipped |
| `ruby` | `.rb` | `Gemfile` | reserved |
| `rust` | `.rs` | `Cargo.toml` | reserved |
| `go` | `.go` | `go.mod` | reserved |

Python routes run on the bundled Python 3.14 runtime; `requirements.txt`
dependencies are installed into it (the container flow does this in the app
image).

## Using a reserved loader

If the user picks a loader marked `reserved`, tell them it is not shipped in
the npm payload yet and the route will not run with `npm install -g metassr`.
Offer to:

1. write the route now so it runs once the loader ships, or
2. keep the app on node/typescript until then.

## Local dev with extra runtimes

For loaders you have installed on the host (e.g. a system Python), the
framework's dev mode may load them directly; the npm payload and the container
images are the constrained environments.