const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const config = require('./release-config.json');
const loader = fs.readFileSync(path.join(__dirname, `../crates/${config.project}-node/index.js`), 'utf8');
const requested = [];
const api = new Proxy({}, { get: (_, key) => key === 'then' ? undefined : () => {} });
vm.runInNewContext(loader, {
  module: { exports: {} }, exports: {}, __dirname: '.',
  process: { platform: 'linux', arch: 'ia32', env: {}, config: { variables: {} } },
  require(name) {
    if (name.startsWith('node:')) return require(name);
    requested.push(name);
    if (name === `@${config.project}/node-linux-ia32-gnu`) return api;
    throw new Error('Simulated missing local binary');
  },
});
assert.ok(requested.includes(`@${config.project}/node-linux-ia32-gnu`));
assert.ok(!requested.some(name => name.includes('linux-x64')));
const missing = [];
assert.throws(() => vm.runInNewContext(loader, {
  module: { exports: {} }, exports: {}, __dirname: '.',
  process: { platform: 'linux', arch: 'ia32', env: {}, config: { variables: {} } },
  require(name) {
    if (name.startsWith('node:')) return require(name);
    missing.push(name);
    throw new Error('Missing addon');
  },
}), /native binding/);
assert.ok(!missing.some(name => /wasm|wasi/.test(name)));
process.stdout.write('Linux ia32 loader selected the matching optional package.\n');
