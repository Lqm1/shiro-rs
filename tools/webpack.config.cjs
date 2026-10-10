const path = require('node:path');
module.exports = {
  mode: 'production',
  entry: path.resolve(__dirname, '../target/distribution/release/wasm-package/bundler/index.js'),
  output: {
    path: path.resolve(__dirname, '../target/distribution/release/browser-bundler'),
    filename: 'main.js',
    library: { type: 'module' },
  },
  experiments: { asyncWebAssembly: true, outputModule: true },
};
