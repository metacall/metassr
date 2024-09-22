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
        filename: '[name].js',
        library: {
            type: 'commonjs2',
        },
        publicPath: ''
    },
    resolve: {
        extensions: ['.js', '.jsx', '.tsx', '.ts']
    },
    optimization: {
        minimize: false,
    },
    module: {
        rules: [
            {
                test: /\.(jsx|js)$/,
                exclude: /node_modules/,
                use: {
                    loader: 'builtin:swc-loader',
                    options: {
                        sourceMap: true,
                        jsc: {
                            parser: {
                                syntax: 'ecmascript',
                                jsx: true,
                            },
                            externalHelpers: false,
                            preserveAllComments: false,
                            transform: {
                                react: {
                                    runtime: 'automatic',
                                    throwIfNamespace: true,
                                    useBuiltins: false,
                                },
                            },
                        },
                    },

                },
                type: 'javascript/auto',
            },
            {
                test: /\.(tsx|ts)$/,
                exclude: /node_modules/,
                use: {
                    loader: 'builtin:swc-loader',
                    options: {
                        jsc: {
                            parser: {
                                syntax: 'typescript',
                                tsx: true,
                            },
                            transform: {
                                react: {
                                    runtime: 'automatic',
                                    throwIfNamespace: true,
                                    useBuiltins: false,
                                },
                            },
                        },
                    },
                },
                type: 'javascript/auto',
            },
            {
                test: /\.(png|svg|jpg)$/,
                type: 'asset/inline',
            },
        ],

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
    web_bundling
};
