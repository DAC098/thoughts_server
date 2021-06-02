const path = require("path");
const { WebpackManifestPlugin } = require("webpack-manifest-plugin");

module.exports = {
	entry: {
		main: "./view/entry.tsx"
	},
	output: {
		path: path.resolve(__dirname, "static"),
		filename: "[name].b.js",
		clean: true
	},
	resolve: {
		extensions: [".ts",".tsx",".js",".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.tsx?$/,
				exclude: /node_modules/,
				use: [
					{
						loader: "ts-loader"
					}
				]
			}
		]
	},
	optimization: {
		runtimeChunk: "single",
		splitChunks: {
			cacheGroups: {
				fabric_ui: {
					test: /[\\/]node_modules[\\/]@fluentui/,
					name: "fabric_ui",
					chunks: "all"
				},
				vendor: {
					test: /[\\/]node_modules[\\/](?!@fluentui)/,
					name: "vendor",
					chunks: "all"
				}
			}
		}
	},
	plugins: [
		new WebpackManifestPlugin({})
	]
};