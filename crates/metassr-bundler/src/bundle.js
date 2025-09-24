const { rspack } = require('@rspack/core');
const { join } = require('path');

function safelyParseJSON(json) {
    try {
        return JSON.parse(json);
    } catch {
        return undefined;
    }
}

const defaultConfig = {
    output: {
        filename: '[name].js',
        library: {
            type: 'commonjs2',
        },
        publicPath: '',
    },
    resolve: {
        extensions: ['.js', '.jsx', '.tsx', '.ts'],
    },
    optimization: {
        minimize: true,
    },
    module: {
        rules: [
        {
                test: /\.(jsx|js)$/,
                exclude: /node_modules/,
                use: {
                    loader: 'builtin:swc-loader',
                    options: {
                        jsc: {
                            parser: {
                                syntax: 'ecmascript',
                                jsx: true,
                                dynamicImport: true
                            },
                            transform: {
                                react: {
                                    runtime: 'automatic',
                                    throwIfNamespace: true
                                }
                            }
                        }
                    }
                },
                type: 'javascript/auto'
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
                                decorators: true
                            },
                            transform: {
                                react: {
                                    runtime: 'automatic',
                                    throwIfNamespace: true
                                }
                            }
                        }
                    }
                },
                type: 'javascript/auto'
            },
            {
                test: /\.(png|svg|jpg|jpeg|gif|woff|woff2|eot|ttf|otf)$/,
                type: 'asset/inline', // Inline assets as Base64 strings
            }
        ]
    }
};

function createBundlerConfig(entry, dist) {
    return {
        ...defaultConfig,
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
        // plugins: [],
        stats: {
            preset: 'errors-warnings',
            timings: true,
            colors: true,
            modules: true
        },
        target: 'web',
        module: defaultConfig.module,
        performance: {
            hints: 'warning',
            maxAssetSize: 250000,
            maxEntrypointSize: 400000
        }
    };
}

async function web_bundling(entry, dist) {
    const compiler = rspack(createBundlerConfig(entry, dist));

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
    web_bundling
};
