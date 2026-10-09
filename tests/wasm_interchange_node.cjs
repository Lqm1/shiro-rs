const fs = require('node:fs/promises');
const path = require('node:path');
const api = require(path.join(path.resolve(process.argv[2]), 'shiro_rs.js'));
import('./wasm_interchange_checks.mjs').then(async ({verifyInterchange}) => {
    const load = name => fs.readFile(path.join(__dirname, 'fixtures', name));
    const result = await verifyInterchange(api, load);
    const repeated = await verifyInterchange(api, load);
    if (JSON.stringify(result) !== JSON.stringify(repeated)) throw new Error('Repeated interchange verification changed');
    console.log('SHIRO actual Node.js interchange:', JSON.stringify({passes: 2, ...result}));
}).catch(error => {console.error(error); process.exitCode = 1;});
