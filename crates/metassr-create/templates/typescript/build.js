const { rspack } = require('@rspack/core');
const path = require('path');

const config = {
    entry: {
        index: './src/pages/index.tsx',
        // client: './client/index.js'
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: '[name].js'
    },
    target: 'web',
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

    },
    devServer: {
        port: 1111,
        onListening: function (devServer) {
            if (!devServer) {
                throw new Error('@rspack/dev-server is not defined');
            }

            const port = devServer.server.address().port;
            console.log('Listening on port:', port);
        },
        compress: true,
        port: 9000,
        hot: true, // Enable hot module replacement
        open: true // Open the browser after the server starts
    },
    watch: true
};

import { rspack } from '@rspack/core';
import { RspackDevServer } from '@rspack/dev-server';
import path from 'path';
import fs from 'fs';
import config from '../rspack.config';
import { normalizeAssets, requireFromString, hash } from './util';

async function main() {
    const compiler = rspack([
        {
            ...config,
            name: 'Client',
            entry: {
                client: './src/entry-client.jsx',
            },
            mode: 'development',
            devtool: 'cheap-module-source-map',
            builtins: { noEmitAssets: false },
            stats: { preset: 'errors-warnings', timings: true, colors: true },
            target: 'web',
        },
        {
            ...config,
            name: 'Server',
            entry: {
                server: './src/entry-server.jsx',
            },
            mode: 'development',
            devtool: 'cheap-module-source-map',
            builtins: { noEmitAssets: false },
            stats: { preset: 'errors-warnings', timings: true, colors: true },
            target: 'node',
            output: {
                library: {
                    type: 'commonjs-module',
                },
            },
        },
    ]);
    const devServer = new RspackDevServer(
        {
            ...(config.devServer ?? {}),
            devMiddleware: { serverSideRender: true },
            hot: true,
            client: { overlay: { errors: true, warnings: false } },
            // Allow CodeSandbox to access dev server
            allowedHosts: 'all',
            setupMiddlewares(middlewares, devServer) {
                if (!devServer) {
                    throw new Error('webpack-dev-server is not defined');
                }

                middlewares.push(async (req, res) => {
                    const { devMiddleware } = res.locals.webpack;
                    const outputFileSystem = devMiddleware.outputFileSystem;
                    const jsonWebpackStats = devMiddleware.stats.toJson();
                    const jsonWebpackStatsClient = jsonWebpackStats.children[0];
                    const jsonWebpackStatsServer = jsonWebpackStats.children[1];
                    const { assetsByChunkName, outputPath } = jsonWebpackStatsClient;

                    if (req.originalUrl === '/') {
                        let render = () => '';

                        const serverChunkPath = path.join(
                            jsonWebpackStatsServer.outputPath,
                            jsonWebpackStatsServer.assetsByChunkName.server[
                            jsonWebpackStatsServer.assetsByChunkName.server.length - 1
                            ],
                        );
                        const serverChunkString = outputFileSystem
                            .readFileSync(serverChunkPath)
                            .toString();
                        if (!serverChunkString) {
                            throw new Error('Server entry compilation result is null!');
                        }
                        try {
                            // TODO: get chunk hash from rspack directly
                            render =
                                requireFromString(
                                    serverChunkString,
                                    `${serverChunkPath}?hash=${hash(serverChunkString)}`,
                                ).render || render;
                        } catch (e) {
                            throw new Error(
                                'Load server entry compilation result failed',
                                // @ts-ignore
                                e?.message,
                            );
                        }

                        // Then use `assetsByChunkName` for server-side rendering
                        // For example, if you have only one main chunk:
                        res.send(
                            `
<!DOCTYPE html>
<html>
  <head>
    <title>My App</title>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    ${normalizeAssets(jsonWebpackStatsServer.assetsByChunkName.server)
                                    .filter((path) => path.endsWith('.css'))
                                    .map((path) => `<link rel="stylesheet" href="${path}">`)
                                    .join('\n')}
    ${normalizeAssets(assetsByChunkName.client)
                                    .filter((path) => path.endsWith('.css'))
                                    .map((path) => `<link rel="stylesheet" href="${path}">`)
                                    .join('\n')}
    ${normalizeAssets(assetsByChunkName.client)
                                    .filter((path) => path.endsWith('.js'))
                                    .map((path) => `<script src="${path}" defer></script>`)
                                    .join('\n')}
  </head>
  <body>
    <div id="root">${await render()}</div>
  </body>
</html>
      `.trim(),
                        );
                    }
                });

                return middlewares;
            },
            // Make sure server entry modification will trigger re-render normally
            watchFiles: ['./src'],
        },
        compiler,
    );

    await devServer.start();
}

