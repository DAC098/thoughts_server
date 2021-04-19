const path = require("path");

module.exports = {
	mode: "development",
	devtool: "inline-source-map",
	entry: {
		main: "./view/entry.tsx"
	},
	output: {
		path: path.resolve(__dirname, "static"),
		filename: "[name].b.js"
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
		runtimeChunk: "single"
	}
};