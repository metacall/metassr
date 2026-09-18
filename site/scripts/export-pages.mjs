#!/usr/bin/env node
//
// Export a MetaSSR static-site build (metassr build -t ssg) into a
// directory that plain static hosts such as GitHub Pages can serve.
//
// MetaSSR emits HTML at dist/pages/<route>/index.html and references assets by
// site-root absolute URLs (/dist/..., /static/...). Its own server does the
// route-to-file mapping at runtime. A static host does not run that code, so
// this script:
//
//   1. flattens dist/pages/<route>/index.html to <out>/<route>/index.html
//   2. copies the client bundles to <out>/dist/... so /dist/... URLs resolve
//   3. copies static/ to <out>/static/
//   4. injects <base href="..."> and prefixes root-absolute asset URLs with the
//      deploy base path (e.g. /metassr for a GitHub project site)
//   5. emits 404.html from the _notfound page and a .nojekyll marker
//
// Environment:
//   DIST       build output directory        (default: dist)
//   STATIC     static assets directory       (default: static)
//   OUT        export directory              (default: _site)
//   BASE_PATH  deploy base path, e.g. /repo  (default: /)

import { readdir, readFile, writeFile, mkdir, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join, relative, dirname, extname, sep } from "node:path";

const ROOT = process.cwd();
const DIST = process.env.DIST || "dist";
const STATIC = process.env.STATIC || "static";
const OUT = process.env.OUT || "_site";

function normalizeBase(raw) {
  const value = (raw || "/").trim();
  if (value === "" || value === "/") {
    return "";
  }
  const withLeadingSlash = value.startsWith("/") ? value : `/${value}`;
  return withLeadingSlash.replace(/\/+$/, "");
}

const BASE = normalizeBase(process.env.BASE_PATH);
const BASE_HREF = `${BASE}/`;

async function walk(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await walk(full)));
    } else if (entry.isFile()) {
      files.push(full);
    }
  }
  return files;
}

async function copyFile(src, dest) {
  const destDir = dirname(dest);
  await mkdir(destDir, { recursive: true });
  const data = await readFile(src);
  await writeFile(dest, data);
}

function rewriteHtml(html) {
  if (!BASE) {
    return html;
  }
  return html
    .replaceAll('="/dist/', `="${BASE}/dist/`)
    .replaceAll('="/static/', `="${BASE}/static/`)
    .replaceAll("='/dist/", `='${BASE}/dist/`)
    .replaceAll("='/static/", `='${BASE}/static/`);
}

function rewriteCss(css) {
  if (!BASE) {
    return css;
  }
  return css
    .replaceAll("url(/dist/", `url(${BASE}/dist/`)
    .replaceAll("url(/static/", `url(${BASE}/static/`)
    .replaceAll('url("/dist/', `url("${BASE}/dist/`)
    .replaceAll('url("/static/', `url("${BASE}/static/`)
    .replaceAll("url('/dist/", `url('${BASE}/dist/`)
    .replaceAll("url('/static/", `url('${BASE}/static/`);
}

function injectBaseTag(html) {
  if (/<base\s/i.test(html)) {
    return html;
  }
  const tag = `<base href="${BASE_HREF}">`;
  return html.replace(/<head([^>]*)>/i, (match) => `${match}\n    ${tag}`);
}

async function main() {
  const distPages = join(ROOT, DIST, "pages");
  if (!existsSync(distPages)) {
    throw new Error(
      `No static-site output found at ${distPages}. Run "metassr build -t ssg" first.`,
    );
  }

  const outDir = join(ROOT, OUT);
  await rm(outDir, { recursive: true, force: true });
  await mkdir(outDir, { recursive: true });

  const distFiles = await walk(distPages);
  let routeCount = 0;
  let assetCount = 0;

  for (const file of distFiles) {
    const rel = relative(distPages, file);
    const ext = extname(file).toLowerCase();

    if (file.endsWith(`${sep}index.html`) || rel === "index.html") {
      // Flatten dist/pages/<route>/index.html -> <out>/<route>/index.html
      const html = injectBaseTag(rewriteHtml(await readFile(file, "utf8")));
      const dest = join(outDir, rel);
      await mkdir(dirname(dest), { recursive: true });
      await writeFile(dest, html);
      routeCount += 1;
      continue;
    }

    if (ext === ".js" || ext === ".css") {
      // Keep the /dist/... layout so absolute asset URLs resolve.
      const dest = join(outDir, DIST, "pages", rel);
      await mkdir(dirname(dest), { recursive: true });
      const content = await readFile(file);
      await writeFile(dest, ext === ".css" ? rewriteCss(content.toString("utf8")) : content);
      assetCount += 1;
    }
  }

  // Route pages reference /dist/pages/... but the 404 handler needs the same
  // assets available when served from arbitrary paths.
  if (existsSync(join(ROOT, STATIC))) {
    for (const file of await walk(join(ROOT, STATIC))) {
      const rel = relative(join(ROOT, STATIC), file);
      const dest = join(outDir, STATIC, rel);
      await mkdir(dirname(dest), { recursive: true });
      const content = await readFile(file);
      await writeFile(
        dest,
        extname(file).toLowerCase() === ".css" ? rewriteCss(content.toString("utf8")) : content,
      );
    }
  }

  // GitHub Pages serves 404.html for unknown paths.
  const notFound = join(outDir, "_notfound", "index.html");
  if (existsSync(notFound)) {
    await copyFile(notFound, join(outDir, "404.html"));
  }

  // Prevent Jekyll from stripping underscore-prefixed paths.
  await writeFile(join(outDir, ".nojekyll"), "");

  console.log(
    `Exported ${routeCount} page(s) and ${assetCount} asset(s) to ${OUT}/ ` +
      `(base path: ${BASE || "/"}).`,
  );
}

main().catch((error) => {
  console.error(`[export-pages] ${error.message}`);
  process.exit(1);
});
