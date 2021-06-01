const common = require("./webpack.base");
const { merge } = require("webpack-merge");

module.exports = merge(common, {
    mode: "production"
});