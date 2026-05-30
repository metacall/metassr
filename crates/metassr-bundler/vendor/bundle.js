const esbuild = require('esbuild');
const { join } = require('path');

/**
 * Safely parses a JSON string, returning undefined if parsing fails.
 * @param {string} json - The JSON string to parse.
 * @returns {Object|undefined} - Parsed object or undefined if parsing fails.
 */
function safelyParseJSON(json) {
    try {
        return JSON.parse(json);
    } catch {
        return undefined;
    }
}

/**
 * Bundles web resources using esbuild (synchronous).
 * @param {Object|string} entry - The entry point(s) as JSON: {"name": "path", ...}
 * @param {string} dist - The distribution path where bundled files will be output.
 * @param {string} devMode - "true" for development mode (no minification, no sourcemaps).
 * @returns {number} 0 on success.
 */
function webBundling(entry, dist, devMode) {
    const isDev = devMode === 'true';
    const entries = safelyParseJSON(entry) || entry;

    // esbuild expects entryPoints as { outName: inputPath }
    // Our entries are already in that format: {"pages/home": "/abs/path/to/file.js"}
    const entryPoints = typeof entries === 'object' ? entries : { main: entries };

    const result = esbuild.buildSync({
        entryPoints,
        outdir: join(process.cwd(), dist),
        bundle: true,
        allowOverwrite: true,
        format: 'cjs',
        platform: 'browser',
        target: 'es2020',
        minify: !isDev,
        sourcemap: isDev ? false : true,
        jsx: 'automatic',
        resolveExtensions: ['.js', '.jsx', '.tsx', '.ts'],
        loader: {
            '.js': 'jsx',
            '.png': 'dataurl',
            '.svg': 'dataurl',
            '.jpg': 'dataurl',
            '.jpeg': 'dataurl',
            '.gif': 'dataurl',
            '.woff': 'dataurl',
            '.woff2': 'dataurl',
            '.eot': 'dataurl',
            '.ttf': 'dataurl',
            '.otf': 'dataurl',
            '.webp': 'dataurl',
        },
        logLevel: 'error',
    });

    if (result.errors && result.errors.length > 0) {
        throw new Error(`Compilation errors:\n${result.errors.map(e => e.text).join('\n')}`);
    }

    return 0;
}

module.exports = {
    web_bundling: webBundling
};
