# MetaSSR

MetaSSR CLI: polyglot server-side rendering framework powered by MetaCall.

## Install

```sh
npm install -g metassr
```

Requires Node.js >= 20 on Linux x64 (glibc 2.38+). The runtime payload ships in
the `@metassr/linux-x64-gnu` package.

## Supported loaders

The npm package bundles MetaCall loaders for the following languages. API
routes in these languages run out of the box:

| Loader | Languages | Status |
| --- | --- | --- |
| `node` | JavaScript | shipped |
| `typescript` | TypeScript | shipped |
| `python` | Python | shipped |
| `ruby` | Ruby | reserved — not in the payload yet |
| `rust` | Rust | reserved — not in the payload yet |
| `go` | Go | reserved — not in the payload yet |

Python API routes run on the bundled Python 3.14 runtime. Third-party
packages are declared in a `requirements.txt` next to the app and installed
with `pip install --target <python-site-packages> --python-version 3.14`
(see the containers guide for the app image flow).

Check the list programmatically:

```sh
npm view metassr loaders
```

`reserved` loaders are planned but not bundled; their API routes do not run
with this package yet.