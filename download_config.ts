import { Configuration } from "webpack";

import { rules } from "./webpack.rules"
import { resolve } from "path";

const modulePath = resolve(__dirname, "dist_utility/utility_process");

export const downloadConfig: Configuration = {
  entry: "./src/download.ts", // Change to your own entry point
  target: "node",
  module: {
    rules,
  },
  output: {
    path: modulePath,
    filename: "download.js",
  },
  resolve: {
    extensions: [".js", ".ts", ".jsx", ".tsx", ".css", ".json"],
  },
  // TODO: find a way to infer this based on whether we run electron-forge start
  // or package.
  mode: "development",
  optimization: {
    usedExports: true,
  }
};