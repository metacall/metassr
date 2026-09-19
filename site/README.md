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
├── static/assets/          # served at /static/assets (metacall + metassr logos)
├── scripts/export-pages.mjs
└── metassr.toml            # build.type = "ssg"
```

## Design system

The landing page and the `metassr create` templates share one visual identity,
driven by CSS custom properties in `src/styles/global.css`:

- **Surfaces**: cream `--color-surface` cards with a strong
  `--color-border-strong` outline and `--radius-lg` corners on a white canvas.
- **Ink & accent**: `--color-ink` / `--color-ink-muted` text with a teal
  `--color-accent` for links, buttons and the benchmark table.
- **Type**: Inter with a system-sans fallback; `--font-mono` for code.
- **Spacing & radius**: the `--space-*` and `--radius-*` scales keep the
  layout and components consistent.

To rebrand, change the tokens in `:root`; components only reference variables.

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
