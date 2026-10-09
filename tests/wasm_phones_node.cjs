const fs = require('node:fs/promises');
const path = require('node:path');
const api = require(path.join(path.resolve(process.argv[2]), 'shiro_rs.js'));
Promise.all([import('./wasm_phones_checks.mjs'), import('./wasm_interchange_checks.mjs')]).then(async ([{verifyPhones}, {verifyInterchange}]) => {
    const load = name => fs.readFile(path.join(__dirname, 'fixtures', name));
    const phones = await verifyPhones(api, load), interchange = await verifyInterchange(api, load);
    if (JSON.stringify(await verifyPhones(api, load)) !== JSON.stringify(phones)) throw new Error('Repeated phones verification changed');
    if (JSON.stringify(await verifyInterchange(api, load)) !== JSON.stringify(interchange)) throw new Error('Repeated interchange verification changed');
    console.log('SHIRO actual Node.js phones and interchange:', JSON.stringify({passes: 2, phones, interchange}));
}).catch(error => {console.error(error); process.exitCode = 1;});
