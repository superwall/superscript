// webpack.browser.js used to build the WASM module for the browser environment.
const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "./target/browser/");

module.exports = {
    name: "superscript-browser",
    mode: "production",
    entry: {
        index: "./src/index.ts"
    },
    output: {
        path: path.resolve(__dirname, './target/browser'),
        filename: "superscript.js",
        library: {
            type: 'module'
        }
    },
    resolve: {
        extensions: ['.ts', '.js', '.wasm']
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/
            }
        ]
    },
    devServer: {
        contentBase: dist,
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "."),
            outDir: "./target/browser",
            target: "web"
        }),
    ],
    experiments: {
        asyncWebAssembly: true,
        topLevelAwait: true,
        outputModule: true
    },
};