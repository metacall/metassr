# MetaSSR website

The MetaSSR landing page. It is itself a MetaSSR app, built with static-site
generation (`metassr build -t ssg`) and deployed to GitHub Pages by the
[`Deploy Site`](../../.github/workflows/pages.yml) workflow.

It deliberately reuses the same components, layout and `global.css` as the
`metassr create` templates so the site matches every generated example.

## Layout

```
site/
├── src/
│   ├── _app.tsx            # wraps every page in PageLayout
│   ├── _head.tsx           # global <head> content
│   ├── components/         # footer (github CTA), github, link
│   ├── data/benchmarks.ts  # benchmark figures shown on the landing page
│   ├── layout/             # PageLayout
│   ├── pages/index.tsx     # landing page
│   ├── pages/_notfound.tsx # 404 page
│   └── styles/global.css   # same stylesheet as the create templates
├── static/assets/          # served at /static/assets (logos + og-image.png)
├── scripts/export-pages.mjs
├── scripts/generate-og.mjs # regenerates the social preview image
└── metassr.toml            # build.type = "ssg"
```

## Design system

The landing page and the `metassr create` templates share one visual identity,
driven by CSS custom properties in `src/styles/global.css`:

- **Background**: a flat, full-bleed `--color-canvas` (`#f4f1ea`, the same warm
  tone as the sales-dashboard example) with no cards, borders or chrome.
- **Ink & accent**: `--color-ink` / `--color-ink-muted` text with a teal
  `--color-accent` for links, buttons and the benchmark table.
- **Type**: Inter with a system-sans fallback; the hero heading is 26px and
  inline logos match it. `--font-mono` is kept for code.
- **Layout**: content is centred and constrained to `--content` (820px); the
  `--space-*` scale keeps spacing consistent.

To rebrand, change the tokens in `:root`; components only reference variables.

The social preview image (`static/assets/og-image.png`, 2400x1260) is rendered
from the hero with `npm run og:image`; it needs a local Chromium/Chrome. The
result is committed, so CI does not regenerate it.

## Local development

Build MetaSSR and install MetaCall first (see the
[installation guide](../../docs/getting-started/installation.md)), then:

```sh
cd site
npm install
npm run build:ssg      # writes site/dist
npm run serve          # serves the pre-rendered site on :8080
```

## Deployment

CI builds the SSG output inside the MetaSSR Docker base image, then runs
`scripts/export-pages.mjs` to turn `dist/` into a GitHub Pages-ready `_site/`:

- flattens `dist/pages/<route>/index.html` to `_site/<route>/index.html`;
- copies the client bundles so `/dist/...` and `/static/...` URLs resolve;
- injects `<base href="...">` and prefixes asset URLs with the project base
  path (`BASE_PATH`, e.g. `/metassr`);
- emits `404.html` and `.nojekyll`.

To enable deployments, set **Settings → Pages → Source** to **GitHub Actions**
on the repository, then push to `master` or run the workflow manually.
