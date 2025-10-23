const { rspack } = require('@rspack/core');
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

// Default configuration object for rspack bundling process
const defaultConfig = {
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
        minimize: true, // You can disable minimization for easier debugging
    },
    module: {
        rules: [
            {
                test: /\.(jsx|js)$/, // Rule for JavaScript and JSX files
                exclude: /node_modules/, // Exclude node_modules directory
                use: {
                    loader: 'builtin:swc-loader', // Use the SWC loader to transpile ES6+ and JSX
                    options: {
                        jsc: {
                            parser: {
                                syntax: 'ecmascript', // Set parser syntax to ECMAScript
                                jsx: true, // Enable parsing JSX syntax
                                dynamicImport: true, // Enable parsing dynamic imports
                            },
                            transform: {
                                react: {
                                    runtime: 'automatic', // Use React's automatic JSX runtime
                                    throwIfNamespace: true, // Throw error if namespace is used
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
                                decorators: true
                            },
                            transform: {
                                react: {
                                    runtime: 'automatic', // Use React's automatic JSX runtime
                                    throwIfNamespace: true, // Throw error if namespace is used
                                },
                            },
                        },
                    },
                },
                type: 'javascript/auto', // Specify the type as auto
            },
            {
                test: /\.(png|svg|jpg|jpeg|gif|woff|woff2|eot|ttf|otf|webp)$/,
                type: 'asset/inline', // Inline assets as Base64 strings
            },
        ],
    }
};

function createBundlerConfig(entry, dist) {
    return {
        ...defaultConfig, // Merge with the default config
        entry: safelyParseJSON(entry) ?? entry,
        output: dist ? {
            ...defaultConfig.output,
            path: join(process.cwd(), dist)
        } : defaultConfig.output,
        name: 'Client',
        mode: 'production',
        devtool: 'source-map',
        experiments: {
            css: true
        },
        stats: {
            preset: 'errors-warnings',
            timings: true,
            colors: true,
            modules: true
        },
        target: 'web',
        performance: {
            hints: 'warning',
            maxAssetSize: 250000,
            maxEntrypointSize: 400000
        }
    };
}


/**
 * Bundles web resources using rspack.
 * @param {Object|string} entry - The entry point(s) for the bundling process (can be a string or JSON object).
 * @param {string} dist - The distribution path where bundled files will be output.
 * @returns {Promise} - Resolves when bundling is successful, rejects if there is an error.
 */
async function web_bundling(entry, dist) {
    // Create a bundler instance using the config and parameters
    const compiler = rspack(createBundlerConfig(entry, dist));

    // Return a promise that runs the bundling process and resolves or rejects based on the result
    return new Promise((resolve, reject) => {
        compiler.run((error, stats) => {
            if (error) {
                return reject(new Error(`Bundling failed: ${error.message}`));
            }

            if (stats?.hasErrors()) {
                const info = stats.toJson();
                const errors = info.errors?.map(e => e.message).join('\n') || 'Unknown compilation errors';
                return reject(new Error(`Compilation errors:\n${errors}`));
            }

            resolve(0);
        });
    });
}

module.exports = {
    web_bundling // Export the web_bundling function to call it via metacall
};