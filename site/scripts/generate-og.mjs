#!/usr/bin/env node
//
// Render the Open Graph / social preview image for the MetaSSR landing page.
//
// It builds a small standalone HTML page that mirrors the hero (logo, title,
// lead paragraph) using the same design tokens as src/styles/global.css, then
// screenshots it with headless Chromium at the standard 1200x630 OG size.
//
// The generated image is committed at static/assets/og-image.png, so this is a
// developer convenience: CI does not need a browser to deploy the site.
//
// Environment:
//   CHROME_PATH  browser binary to use (default: chromium/google-chrome on PATH)
//   OG_OUT       output path (default: static/assets/og-image.png)
//   OG_SCALE     device scale factor (default: 2 for a crisp 2400x1260 image)

import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const WIDTH = 1200;
const HEIGHT = 630;
const SCALE = process.env.OG_SCALE || "2";
const OUT = resolve(ROOT, process.env.OG_OUT || "static/assets/og-image.png");

const BROWSERS = [
	process.env.CHROME_PATH,
	"chromium",
	"chromium-browser",
	"google-chrome",
	"google-chrome-stable",
].filter(Boolean);

function findBrowser() {
	for (const candidate of BROWSERS) {
		try {
			const resolved = execFileSync("which", [candidate], { encoding: "utf8" }).trim();
			if (resolved) {
				return resolved;
			}
		} catch {
			// try the next candidate
		}
	}
	throw new Error("No Chromium/Chrome binary found. Set CHROME_PATH to one.");
}

function dataUri(file, mime) {
	return `data:${mime};base64,${readFileSync(join(ROOT, file)).toString("base64")}`;
}

const logo = dataUri("static/assets/metassr-logo.svg", "image/svg+xml");
const metacall = dataUri("static/assets/metacall-logo.png", "image/png");

const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<style>
    :root {
        --font-sans: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        --color-canvas: #f4f1ea;
        --color-ink: #1f2933;
        --color-accent: #006d77;
    }

    *,
    *::before,
    *::after {
        box-sizing: border-box;
    }

    html,
    body {
        margin: 0;
    }

    body {
        display: flex;
        align-items: center;
        justify-content: center;
        width: ${WIDTH}px;
        height: ${HEIGHT}px;
        padding: 72px 96px;
        background: var(--color-canvas);
        color: var(--color-ink);
        font-family: var(--font-sans);
        line-height: 1.6;
        -webkit-font-smoothing: antialiased;
    }

    .og {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 26px;
        max-width: 940px;
        text-align: center;
    }

    .og-logo {
        width: 210px;
        height: auto;
    }

    .og-title {
        margin: 0;
        font-size: 42px;
        font-weight: 600;
        line-height: 1.25;
    }

    .og-brand {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        color: var(--color-accent);
        font-weight: 700;
    }

    .og-mark {
        width: 42px;
        height: 42px;
        object-fit: contain;
    }

    .og-lead {
        margin: 0;
        font-size: 21px;
        line-height: 1.6;
    }
</style>
</head>
<body>
    <div class="og">
        <img class="og-logo" src="${logo}" alt="">
        <h1 class="og-title">
            Polyglot Programming on the Web, powered by
            <span class="og-brand"><img class="og-mark" src="${metacall}" alt="">MetaCall</span>
        </h1>
        <p class="og-lead">
            MetaSSR is a powerful experimental Server-Side Rendering (SSR) framework
            crafted for high-performance, dynamic web applications. MetaSSR uses the
            MetaCall Runtime, exploring web-based use cases for polyglot programming.
        </p>
    </div>
</body>
</html>
`;

const browser = findBrowser();
const dir = mkdtempSync(join(tmpdir(), "metassr-og-"));
const page = join(dir, "og.html");
writeFileSync(page, html);

mkdirSync(dirname(OUT), { recursive: true });

execFileSync(
	browser,
	[
		"--headless",
		"--no-sandbox",
		"--disable-gpu",
		"--hide-scrollbars",
		"--force-device-scale-factor=" + SCALE,
		`--window-size=${WIDTH},${HEIGHT}`,
		`--screenshot=${OUT}`,
		`file://${page}`,
	],
	{ stdio: "inherit" },
);

if (!existsSync(OUT)) {
	throw new Error(`Screenshot was not written to ${OUT}`);
}

console.log(`Wrote ${OUT} (${WIDTH}x${HEIGHT} @${SCALE}x) using ${browser}.`);
