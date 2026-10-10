function assert(value, message) { if (!value) throw new Error(message); }
function equal(actual, expected, message) { assert(Object.is(actual, expected), `${message}: ${actual} != ${expected}`); }
function canonical(value) {
    if (Array.isArray(value)) return value.map(canonical);
    if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]));
    return value;
}
function json(actual, expected, message) { equal(JSON.stringify(canonical(actual)), JSON.stringify(canonical(expected)), message); }
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }

export async function verifyInterchange(api, load) {
    const text = async name => new TextDecoder().decode(await load(name));
    const patterns = [0, 0x80000000, 0x3f800000, 0xbf800000, 0x7f800000, 0xff800000, 0x7fc01234, 1];
    const bits = Uint32Array.from({length: 9000}, (_, index) => patterns[index % patterns.length]);
    const values = new Float32Array(bits.buffer), bytes = api.rawfloat_write(values);
    for (let index = 0; index < bits.length; index++) equal(new DataView(bytes.buffer, bytes.byteOffset).getUint32(index * 4, true), bits[index], 'rawfloat wire bits');
    const decoded = api.rawfloat_read(bytes, bits.length), decodedBits = new Uint32Array(decoded.buffer);
    for (let index = 0; index < bits.length; index++) equal(decodedBits[index], bits[index], 'rawfloat decoded bits');
    decoded.fill(3); equal(new DataView(bytes.buffer, bytes.byteOffset).getUint32(4, true), 0x80000000, 'detached decoded values');
    for (const missing of [1, 2, 3]) rejects(() => api.rawfloat_read(bytes.subarray(0, bytes.length - missing), bits.length), 'partial scalar');
    rejects(() => api.rawfloat_read(bytes, bits.length - 1), 'sample budget'); equal(api.rawfloat_read(new Uint8Array(), 0).length, 0, 'empty read'); equal(api.rawfloat_write(new Float32Array()).length, 0, 'empty write');

    const map = new api.PhoneMap(await text('labels-phones.json'));
    const input = api.Labels.parse(await text('labels-input.txt'));
    const states = input.to_states(map, .01), expectedDocument = JSON.parse(await text('labels-original-seg.json'));
    json(JSON.parse(states.json()), expectedDocument.file_list[0].states, 'unchanged Lua states and every field');
    let labelRows = 0;
    for (const [includeStates, filename] of [[false, 'labels-original-phones.txt'], [true, 'labels-original-states.txt']]) {
        const labels = api.Labels.from_states(states, .01, includeStates), reference = api.Labels.parse(await text(filename));
        equal(labels.length, reference.length, 'Lua label count');
        for (let index = 0; index < labels.length; index++) {
            const actual = labels.get(index), expected = reference.get(index);
            equal(actual.name, expected.name, 'Lua label name');
            assert(Math.abs(actual.start - expected.start) < 1e-14, 'Lua label start'); assert(Math.abs(actual.end - expected.end) < 1e-14, 'Lua label end');
            actual.free(); expected.free(); labelRows++;
        }
        const output = labels.write(), rows = new TextDecoder().decode(output);
        equal((rows.match(/\r\n/g) || []).length, labels.length, 'CRLF label output');
        const reread = api.Labels.parse(rows); equal(reread.length, labels.length, 'label output roundtrip'); reread.free(); labels.free(); reference.free();
    }
    for (const hop of [0, -1, NaN, Infinity]) { rejects(() => input.to_states(map, hop), 'invalid label hop'); rejects(() => api.Labels.from_states(states, hop, false), 'invalid state hop'); }
    for (const row of ['NaN 1 aa', '1 0 aa', 'x 1 aa', '0 1']) rejects(() => api.Labels.parse(row), 'invalid label parsing');
    const malformed = JSON.parse(states.json()); malformed[0].ext = []; const invalidStates = new api.States(JSON.stringify(malformed));
    rejects(() => api.Labels.from_states(invalidStates, .01, false), 'invalid metadata'); invalidStates.free();
    const statesCopy = states.cloned(); states.replace('[]'); states.free(); input.free(); map.free();
    equal(JSON.parse(statesCopy.json()).length, 7, 'independent state owner after source release'); statesCopy.free();

    const owner = new api.Label(1 + Number.EPSILON, 2, 'original'), labels = new api.Labels(); labels.push(owner);
    owner.start = -0; owner.end = NaN; owner.name = 'changed';
    const child = labels.get(0); equal(child.start, 1 + Number.EPSILON, 'selected label time precision'); equal(child.name, 'original', 'copied label name');
    const clone = labels.cloned(); labels.replace(0, owner); rejects(() => labels.replace(1, owner), 'label index'); equal(labels.get(1), null, 'missing label');
    const replacement = labels.get(0); equal(replacement.start, -0, 'raw signed zero time'); assert(Number.isNaN(replacement.end), 'raw invalid time retained for native validation'); replacement.free();
    labels.clear(); equal(labels.length, 0, 'clear labels'); labels.free(); owner.free(); child.free();
    const retained = clone.get(0); clone.free(); equal(retained.name, 'original', 'child lifetime'); retained.free();
    const bad = new api.Label(0, 1, 'bad\tname'), badLabels = new api.Labels(); badLabels.push(bad); rejects(() => badLabels.write(), 'invalid label output name'); bad.free(); badLabels.free();

    const documents = [
        [api.PhoneMap, {phone_map: {aa: {states: [{dur: 3, out: [7], state_extra: true}], phone_extra: [1]}}, source: {extra: true}}],
        [api.States, [{time: 4, dur: 2, out: [5], jmp: [[-1, .25]], ext: ['aa', 0, {extra: true}], state_extra: [1]}]],
        [api.SegmentationDocument, {file_list: [{filename: 'sound.wav', states: [{time: 4, ext: ['aa', 0], state_extra: true}], file_extra: 3}], source: 'retained'}],
        [api.ModelDefinition, {ndurstate: 2, streamdef: [{nstate: 3, ndim: 4, nmix: 2, weight: .5}], dur_attr: [{index: 1, floor: 2, ceil: 8}]}],
    ];
    for (const [Owner, value] of documents) {
        const original = new Owner(JSON.stringify(value)), copy = original.cloned(); json(JSON.parse(copy.json()), value, 'complete document fields');
        rejects(() => original.replace('{'), 'invalid JSON replacement'); json(JSON.parse(original.json()), value, 'atomic JSON replacement');
        original.free(); json(JSON.parse(copy.json()), value, 'independent document lifetime'); copy.free();
    }
    for (const [filename, expected] of [['dir.name/clip.features.f', 'dir.name/clip.features.txt'], ['dir.name\\clip', 'dir.name\\clip.txt'], ['.f', '.txt'], ['clip', 'clip.txt']]) {
        equal(api.label_output_path(filename, '.txt'), expected, 'legacy label path');
    }
    return {rawfloatValues: bits.length, luaStates: 7, luaLabelRows: labelRows, documentKinds: documents.length};
}
