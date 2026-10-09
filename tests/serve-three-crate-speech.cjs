const http = require('node:http');
const fs = require('node:fs/promises');
const path = require('node:path');
if (process.argv.length !== 5 && process.argv.length !== 6) {
    throw new Error('Usage: node serve-three-crate-speech.cjs CIGLET_WEB LIBLRHSMM_WEB SHIRO_WEB [FIXTURES]');
}
const work = __dirname;
const roots = {
    ciglet: path.resolve(process.argv[2]),
    liblrhsmm: path.resolve(process.argv[3]),
    shiro: path.resolve(process.argv[4]),
    fixtures: path.resolve(process.argv[5] || path.join(work, 'fixtures')),
};
const mime = {'.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.wasm': 'application/wasm'};
const server = http.createServer(async (request, response) => {
    try {
        const url = new URL(request.url, 'http://127.0.0.1');
        const segments = decodeURIComponent(url.pathname).split('/').filter(Boolean);
        if (segments.some(value => value === '.' || value === '..' || /[\\\0]/.test(value))) {
            response.writeHead(400).end(); return;
        }
        let file;
        if (segments.length === 0) file = path.join(work, 'three-crate-speech-browser.html');
        else if (segments.length === 1 && segments[0] === 'three-crate-speech-checks.mjs') file = path.join(work, segments[0]);
        else if (segments.length === 2 && Object.hasOwn(roots, segments[0])) file = path.join(roots[segments[0]], segments[1]);
        else { response.writeHead(404).end(); return; }
        const bytes = await fs.readFile(file);
        response.writeHead(200, {'Content-Type': mime[path.extname(file)] || 'application/octet-stream'}).end(bytes);
    } catch { response.writeHead(404).end(); }
});
server.listen(0, '127.0.0.1', () => console.log('Three-crate speech server port=' + server.address().port));
