const fs = require('node:fs/promises');
const path = require('node:path');
if (process.argv.length !== 7) {
    throw new Error('Usage: node three-crate-speech-node.cjs CIGLET_NODE LIBLRHSMM_NODE SHIRO_NODE FIXTURES OUTPUT_JSON');
}
const C = require(path.join(path.resolve(process.argv[2]), 'ciglet_rs.js'));
const L = require(path.join(path.resolve(process.argv[3]), 'liblrhsmm_rs.js'));
const S = require(path.join(path.resolve(process.argv[4]), 'shiro_rs.js'));
import('./three-crate-speech-checks.mjs').then(async ({verifyThreeCrateSpeech}) => {
    const result = await verifyThreeCrateSpeech(C, L, S,
        name => fs.readFile(path.join(path.resolve(process.argv[5]), name)),
        message => console.log(message));
    await fs.writeFile(process.argv[6], JSON.stringify(result, null, 2) + '\n');
    console.log('Three-crate real-speech actual Node:', JSON.stringify(result));
}).catch(error => { console.error(error); process.exitCode = 1; });
