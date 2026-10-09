function assert(value, message) { if (!value) throw new Error(message); }
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
export async function verifyFeatures(api, load) {
    const bytes = new Uint8Array(await load('c-xxcc.bin')), view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength); let position = 0;
    const integer = () => { const value = view.getUint32(position, true); position += 4; return value; };
    const scalar = () => { const value = view.getFloat32(position, true); position += 4; return value; };
    assert(integer() === 0x31434358 && integer() === 72, 'original C fixture header');
    let valuesChecked = 0, nonfinite = 0, maximumNormalizedError = 0;
    const defaults = {kind: 0, order: 12, channels: 36, frame_length: 1024, hop: 256, sample_rate_hz: 32000, minimum_bandwidth_hz: 400, warp: 1, include_dc: false, energy: 0, delta: false, acceleration: false};
    const options = new api.FeatureOptions(); for (const [key, value] of Object.entries(defaults)) assert(options[key] === value, 'native default ' + key);
    for (let record = 0; record < 72; record++) {
        const kind = integer(), energy = integer(), flags = integer(), frameLength = integer(), hop = scalar(), samples = integer(), input = new Float32Array(samples);
        for (let index = 0; index < samples; index++) input[index] = scalar();
        const frames = integer(), columns = integer();
        Object.assign(options, defaults, {kind, energy, frame_length: frameLength, hop, channels: 12, sample_rate_hz: 16000, warp: .85, include_dc: Boolean(flags & 1), delta: Boolean(flags & 2), acceleration: Boolean(flags & 4)});
        const copied = options.cloned(); for (const key of Object.keys(defaults)) assert(copied[key] === options[key], 'complete option clone');
        const result = api.Features.extract(input, copied), values = result.values(); copied.free(); assert(result.frames === frames && result.columns === columns && values.length === frames * columns, 'complete C feature shape');
        for (const actual of values) {
            const expected = scalar(); valuesChecked++;
            if (!Number.isFinite(expected)) { assert(Object.is(actual, expected), 'matching nonfinite C value'); nonfinite++; }
            else { const error = Math.abs(actual - expected) / Math.max(Math.abs(expected), 1); maximumNormalizedError = Math.max(maximumNormalizedError, error); assert(error < 2e-5, 'C normalized feature tolerance'); }
        }
        const clone = result.cloned(); values.fill(99); result.free(); assert(clone.frames === frames && clone.values().length === frames * columns, 'independent returned features'); clone.free();
    }
    assert(position === bytes.length, 'complete C fixture consumed');
    Object.assign(options, defaults); const empty = api.Features.extract(new Float32Array(), options); assert(empty.frames === 0 && empty.columns === 12 && empty.values().length === 0, 'empty signal dimensions'); empty.free();
    for (const [key, value] of [['kind', 3], ['energy', 3], ['hop', 0], ['warp', 5], ['channels', 0], ['sample_rate_hz', -1]]) {
        options[key] = value; rejects(() => api.Features.extract(new Float32Array([1]), options), 'invalid feature setting ' + key); Object.assign(options, defaults);
    }
    rejects(() => api.Features.extract(new Float32Array([NaN]), options), 'nonfinite input');
    const rawBits = new Uint32Array([0x80000000, 0x7fc00123, 0x7f800000]), arbitrary = new api.Features(7, 9, new Float32Array(rawBits.buffer));
    const clone = arbitrary.cloned(); arbitrary.frames = 1; arbitrary.columns = 2; arbitrary.set_values(new Float32Array([3, 4])); assert(arbitrary.frames === 1 && arbitrary.columns === 2 && arbitrary.values()[1] === 4, 'all native result fields editable'); arbitrary.free();
    const values = clone.values(), actualBits = new Uint32Array(values.buffer, values.byteOffset, values.length); rawBits.forEach((value, index) => assert(value === actualBits[index], 'arbitrary IEEE values preserved')); assert(clone.frames === 7 && clone.columns === 9, 'arbitrary shape preserved'); clone.free(); options.free();
    return {originalCCases: 72, valuesChecked, nonfinite, maximumNormalizedError, optionFields: 12, resultFields: 3};
}
