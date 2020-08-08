const path = require("path");
const MonacoWebpackPlugin = require("monaco-editor-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");

module.exports = {
    entry: "./index.js",
    output: {
        path: path.resolve(__dirname, "..", "public"),
        filename: "bundle.js",
    },
    optimization: {
        moduleIds: "hashed",
        runtimeChunk: {
            name: "mc-runtime",
        },
        splitChunks: {
            cacheGroups: {
                monaco: {
                    test: /[\\/]node_modules[\\/]monaco-editor/,
                    name: "mc-monaco",
                    chunks: "all",
                    priority: 1,
                },
                vendor: {
                    test: /[\\/]node_modules[\\/]/,
                    name: "mc-vendor",
                    chunks: "all",
                },
            },
        },
    },
    mode: process.env.NODE_ENV === "production" ? "production" : "development",
    plugins: [
        new HtmlWebpackPlugin({ template: "./index.html" }),
        new MonacoWebpackPlugin({
            filename: "[name].[contenthash].bundle.js",
            // available options are documented at https://github.com/Microsoft/monaco-editor-webpack-plugin#options
            languages: ["javascript", "typescript"],
        }),
    ],
    module: {
        rules: [
            {
                test: /\.css$/i,
                use: ["style-loader", "css-loader"],
            },
            {
                test: /\.(woff|woff2|eot|ttf|otf)$/,
                use: ["file-loader"],
            },
        ],
    },
    devtool: "source-map",
};
