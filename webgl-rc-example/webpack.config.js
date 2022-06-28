const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const TsconfigPathsPlugin = require("tsconfig-paths-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
    mode: "production",
    entry: {
        index: "./ui/app.tsx",
    },
    output: {
        path: dist,
        filename: "[name].js",
        wasmLoading: "fetch",
    },
    resolve: {
        extensions: [".ts", ".tsx", ".js"],
        plugins: [new TsconfigPathsPlugin()],
    },
    module: {
        rules: [{ test: /\.tsx?$/, loader: "ts-loader" }],
    },
    devServer: {
        static: dist,
        compress: true,
        port: 1234,
    },
    plugins: [
        new CopyPlugin({
            patterns: [{ from: path.resolve(__dirname, "static"), to: dist }],
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "./"),
        }),
    ],
    experiments: {
        syncWebAssembly: true,
    },
    performance: {
        maxEntrypointSize: 512000,
        maxAssetSize: 512000,
    },
};
