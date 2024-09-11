const { rspack } = require('@rspack/core');
const path = require('path');

function safelyParseJSON(json) {
    try {
        return JSON.parse(json)
    } catch (_) {
        return undefined
    }
}

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

    }
}

async function web_bundling(entry, dist) {

    const compiler = rspack(
        {
            ...config,
            entry: safelyParseJSON(entry) ?? entry,
            output: dist ? {
                ...config.output,
                path: path.join(process.cwd(), dist)
            } : config.output,

            name: 'Client',
            mode: 'development',
            devtool: 'source-map',
            stats: { preset: 'errors-warnings', timings: true, colors: true },
            target: 'web',
        }

    );

    return new Promise((resolve, reject) => {
        return compiler.run((error, stats) => {
            if (error) {
                reject(1);
            }

            if (stats?.hasErrors()) {
                reject(1);
            }

            resolve(0);
        });
    });
}

module.exports = {
    web_bundling
};
