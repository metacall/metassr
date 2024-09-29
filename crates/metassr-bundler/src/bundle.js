const { rspack } = require('@rspack/core');
const path = require('path');

/**
 * Safely parses a JSON string, returning undefined if parsing fails.
 * @param {string} json - The JSON string to parse.
 * @returns {Object|undefined} - Parsed object or undefined if parsing fails.
 */
function safelyParseJSON(json) {
    try {
        return JSON.parse(json);
    } catch (_) {
        return undefined;
    }
}

// Default configuration object for rspack bundling process
let config = {
    output: {
        filename: '[name].js', // Output filename with the entry name
        library: {
            type: 'commonjs2', // Set library type to CommonJS2 (Node.js modules)
        },
        publicPath: '' // Specify the base path for all assets within the application
    },
    resolve: {
        extensions: ['.js', '.jsx', '.tsx', '.ts'] // Extensions that will be resolved
    },
    optimization: {
        minimize: false, // Disable minimization for easier debugging
    },
    module: {
        rules: [
            {
                test: /\.(jsx|js)$/, // Rule for JavaScript and JSX files
                exclude: /node_modules/, // Exclude node_modules directory
                use: {
                    loader: 'builtin:swc-loader', // Use the SWC loader to transpile ES6+ and JSX
                    options: {
                        sourceMap: true, // Enable source maps for easier debugging
                        jsc: {
                            parser: {
                                syntax: 'ecmascript', // Set parser syntax to ECMAScript
                                jsx: true, // Enable parsing JSX syntax
                            },
                            externalHelpers: false, // Disable external helpers (use inline helpers)
                            preserveAllComments: false, // Remove comments from output
                            transform: {
                                react: {
                                    runtime: 'automatic', // Use React's automatic JSX runtime
                                    throwIfNamespace: true, // Throw error if namespace is used
                                    useBuiltins: false, // Don't include built-in polyfills
                                },
                            },
                        },
                    },
                },
                type: 'javascript/auto', // Specify the type as auto (for backward compatibility)
            },
            {
                test: /\.(tsx|ts)$/, // Rule for TypeScript and TSX files
                exclude: /node_modules/, // Exclude node_modules directory
                use: {
                    loader: 'builtin:swc-loader', // Use the SWC loader to transpile TS and TSX
                    options: {
                        jsc: {
                            parser: {
                                syntax: 'typescript', // Set parser syntax to TypeScript
                                tsx: true, // Enable parsing TSX syntax
                            },
                            transform: {
                                react: {
                                    runtime: 'automatic', // Use React's automatic JSX runtime
                                    throwIfNamespace: true, // Throw error if namespace is used
                                    useBuiltins: false, // Don't include built-in polyfills
                                },
                            },
                        },
                    },
                },
                type: 'javascript/auto', // Specify the type as auto
            },
            {
                test: /\.(png|svg|jpg)$/, // Rule for image files (PNG, SVG, JPG)
                type: 'asset/inline', // Inline assets as Base64 strings
            },
        ],
    },
};

/**
 * Bundles web resources using rspack.
 * @param {Object|string} entry - The entry point(s) for the bundling process (can be a string or JSON object).
 * @param {string} dist - The distribution path where bundled files will be output.
 * @returns {Promise} - Resolves when bundling is successful, rejects if there is an error.
 */
async function web_bundling(entry, dist) {
    // Create a bundler instance using the config and parameters
    const compiler = rspack(
        {
            ...config, // Merge with the default config
            entry: safelyParseJSON(entry) ?? entry, // Parse entry if it's JSON, otherwise use it as is
            output: dist ? {
                ...config.output,
                path: path.join(process.cwd(), dist), // Use current working directory and output path
            } : config.output,
            // minimize: true,
            name: 'Client', // Name of the bundle (Client)
            mode: 'production', // Set mode to development (for non-minimized builds)
            devtool: 'source-map', // Enable source maps for better debugging
            stats: { preset: 'errors-warnings', timings: true, colors: true }, // Customize bundling stats output
            target: 'web', // Set the target environment to web (for browser usage)
        }
    );

    // Return a promise that runs the bundling process and resolves or rejects based on the result
    return new Promise((resolve, reject) => {
        return compiler.run((error, stats) => {
            // Handle errors during the bundling process
            if (error) {
                reject(error.message); // Reject with the error message if bundling fails
            }

            // Check if there are any errors in the bundling stats
            if (error || stats?.hasErrors()) {
                reject(stats.toString("errors-only")); // Reject with errors-only details from stats
            }
            resolve(0); // Resolve successfully when bundling is complete
        });
    });
}

module.exports = {
    web_bundling // Export the web_bundling function to call it via metacall
};
