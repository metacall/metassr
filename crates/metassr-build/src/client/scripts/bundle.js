const { rspack } = require('@rspack/core')

function safelyParseJSON(json) {
    let parsed

    try {
        parsed = JSON.parse(json)
    } catch (_) {
        parsed = undefined
    }

    return parsed // Could be undefined!
}

const config = {

    output: {
        filename: "[name].bundle.js"
    },
    resolve: {
        extensions: ['.js', '.jsx', '.tsx', '.ts']
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
                // TODO: asset/resource will render different URL in server and client (with host), need fix
                // type: 'asset/resource',
                type: 'asset/inline',
            },
        ],

    }
}



function bundling_client(entry, dist) {
    const compiler = rspack(
        {
            ...config,
            entry: safelyParseJSON(entry) ?? entry,
            output: dist ? {
                ...config.output,
                path: process.cwd() + "/" + dist
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
    bundling_client
};
