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
| `python` | Python | reserved — not in the payload yet |
| `ruby` | Ruby | reserved — not in the payload yet |
| `rust` | Rust | reserved — not in the payload yet |
| `go` | Go | reserved — not in the payload yet |

Check the list programmatically:

```sh
npm view metassr loaders
```

`reserved` loaders are planned but not bundled; their API routes do not run
with this package yet.