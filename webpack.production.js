const common = require("./webpack.base");
const TerserPlugin = require("terser-webpack-plugin");
const { merge } = require("webpack-merge");

module.exports = merge(common, {
    mode: "production",
    optimization: {
        minimize: false,
    }
});