const path = require('node:path');
const api = require(path.join(path.resolve(process.argv[2]), 'shiro_rs.js'));
const owners = ['Label', 'Labels', 'PhoneMap', 'States', 'ModelDefinition', 'SegmentationDocument', 'PhoneMapOptions'];
const functions = ['rawfloat_read', 'rawfloat_write', 'label_output_path'];
for (const name of [...owners, ...functions]) if (name in api) throw new Error('Disabled WASM export: ' + name);
console.log('SHIRO WASM feature disabled:', JSON.stringify({owners: owners.length, functions: functions.length}));
