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
│   ├── components/         # header, footer, github, link, clock
│   ├── layout/             # PageLayout
│   ├── pages/index.tsx     # landing page
│   ├── pages/_notfound.tsx # 404 page
│   └── styles/global.css   # same stylesheet as the create templates
├── static/assets/          # served at /static/assets (e.g. metacall-logo.png)
├── scripts/export-pages.mjs
└── metassr.toml            # build.type = "ssg"
```

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
