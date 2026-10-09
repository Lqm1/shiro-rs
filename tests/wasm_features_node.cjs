const fs = require('node:fs/promises');
const path = require('node:path');
const api = require(path.join(path.resolve(process.argv[2]), 'shiro_rs.js'));
Promise.all([import('./wasm_features_checks.mjs'), import('./wasm_untying_checks.mjs'), import('./wasm_alignment_checks.mjs'), import('./wasm_training_checks.mjs'), import('./wasm_loading_checks.mjs'), import('./wasm_initialization_checks.mjs'), import('./wasm_isolation_checks.mjs'), import('./wasm_data_checks.mjs'), import('./wasm_models_checks.mjs'), import('./wasm_phones_checks.mjs'), import('./wasm_interchange_checks.mjs')]).then(async ([{verifyFeatures}, {verifyUntying}, {verifyAlignment}, {verifyTraining}, {verifyLoading}, {verifyInitialization}, {verifyIsolation}, {verifyData}, {verifyModels}, {verifyPhones}, {verifyInterchange}]) => {
    const load = name => fs.readFile(path.join(__dirname, 'fixtures', name)), checks = [['features', verifyFeatures], ['untying', verifyUntying], ['alignment', verifyAlignment], ['training', verifyTraining], ['loading', verifyLoading], ['initialization', verifyInitialization], ['isolation', verifyIsolation], ['data', verifyData], ['models', verifyModels], ['phones', verifyPhones], ['interchange', verifyInterchange]], results = {};
    for (const [name, verify] of checks) results[name] = await verify(api, load);
    for (const [name, verify] of [...checks].reverse()) if (JSON.stringify(await verify(api, load)) !== JSON.stringify(results[name])) throw new Error('Repeated verification changed: ' + name);
    console.log('SHIRO actual Node.js features and prior families:', JSON.stringify({passes: 2, results}));
}).catch(error => {console.error(error); process.exitCode = 1;});
