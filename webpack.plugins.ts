import type IForkTsCheckerWebpackPlugin from 'fork-ts-checker-webpack-plugin';
import { join } from 'path';
import { DefinePlugin} from 'webpack';

// eslint-disable-next-line @typescript-eslint/no-var-requires
const ForkTsCheckerWebpackPlugin: typeof IForkTsCheckerWebpackPlugin = require('fork-ts-checker-webpack-plugin');

export const plugins = [
  new DefinePlugin({
    UTILITY_PROCESS_PATH: JSON.stringify(join(__dirname, "dist_utility/utility_process/index.js")),
  }),
  new ForkTsCheckerWebpackPlugin({
    logger: 'webpack-infrastructure',
  }),
];
