const { rspack } = require('@rspack/core');
const path = require("path");

function safelyParseJSON(json) {
    let parsed

    try {
        parsed = JSON.parse(json)
    } catch (_) {
        parsed = undefined
    }

    return parsed // Could be undefined!
}

let config = {

    output: {
        filename: "[name].js",
        library: {
            type: "commonjs2",
        },
        publicPath: ""
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
                                    runtime: "automatic",
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
                                    runtime: "automatic",
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



function web_bundling(entry, dist) {

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

    compiler.run((error, stats) => {
        if (error) {
            console.error(error);
            process.exit(2);
        }
        if (stats && stats.hasErrors()) {
            process.exitCode = 1;
        }
        if (!compiler || !stats) {
            return;
        }
    });

}

module.exports = {
    web_bundling
};
