import assert from 'node:assert/strict';
import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';

const root = path.resolve(import.meta.dirname, '..');
const config = JSON.parse(await fs.readFile(path.join(root, 'tools/release-config.json'), 'utf8'));
const { chromium, firefox, webkit } = createRequire(path.join(root, 'crates', `${config.project}-node`, 'package.json'))('playwright');
const contentTypes = { '.mjs': 'text/javascript', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const server = http.createServer(async (request, response) => {
  try {
    const url = new URL(request.url, 'http://localhost');
    if (url.pathname === '/') {
      response.setHeader('Content-Type', 'text/html');
      response.end('<!doctype html><title>Release verification</title>');
      return;
    }
    const filename = path.resolve(root, '.' + decodeURIComponent(url.pathname));
    if (!filename.startsWith(root + path.sep)) {
      response.writeHead(403).end();
      return;
    }
    response.setHeader('Content-Type', contentTypes[path.extname(filename)] ?? 'application/octet-stream');
    response.end(await fs.readFile(filename));
  } catch {
    response.writeHead(404).end();
  }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const address = server.address();
assert.ok(address && typeof address === 'object');
const base = `http://127.0.0.1:${address.port}`;
const reports = [];
try {
  for (const [name, launcher] of Object.entries({ chromium, firefox, webkit })) {
    const browser = await launcher.launch();
    try {
      const page = await browser.newPage();
      await page.goto(base);
      for (const target of ['web', 'bundler']) {
        const report = await page.evaluate(async ({ project, target }) => {
          const api = await import(target === 'web'
            ? '/target/distribution/release/wasm-package/web/index.js'
            : '/target/distribution/release/browser-bundler/main.js');
          if (target === 'web') await api.default();
          const load = async name => {
            const response = await fetch('/tests/fixtures/' + name);
            if (!response.ok) throw new Error('Fixture not found: ' + name);
            return new Uint8Array(await response.arrayBuffer());
          };
          if (project !== 'shiro-rs') {
            const { verifyAllBindings } = await import('/tests/wasm_all_checks.mjs');
            return await verifyAllBindings(api, load);
          }
          const families = ['alignment', 'audio', 'batch', 'data', 'features', 'index', 'initialization',
            'interchange', 'isolation', 'loading', 'models', 'phones', 'training', 'untying', 'utterances', 'tool_workflow'];
          for (const family of families) {
            const module = await import(`/tests/wasm_${family}_checks.mjs`);
            const verify = Object.values(module).find(value => typeof value === 'function');
            if (!verify) throw new Error('No verifier: ' + family);
            await verify(api, load);
          }
          return { families: families.length };
        }, { project: config.project, target });
        reports.push({ browser: name, target, passed: true, report });
      }
    } finally {
      await browser.close();
    }
  }
  await fs.writeFile(path.join(root, 'target/distribution/release/browser-tests.json'), JSON.stringify(reports, null, 2));
  process.stdout.write('Both WASM entrypoints passed in Chromium, Firefox and WebKit.\n');
} finally {
  server.close();
}
