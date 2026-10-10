function assert(value, message) { if (!value) throw new Error(message); }
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
function bits(values) { return new Uint32Array(values.buffer, values.byteOffset, values.length); }
function equal(actual, expected, message) { assert(actual.length === expected.length && actual.every((value, index) => Object.is(value, expected[index])), message); }
function floats(bytes) { const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength); return Float32Array.from({length: bytes.length / 4}, (_, index) => view.getFloat32(index * 4, true)); }
export async function verifyAudio(api, load) {
    const options = new api.AudioOptions(), defaults = {normalize: false, dither_level: 0, output_sample_rate: null, boundary: 0, kernel: 0};
    for (const [key, value] of Object.entries(defaults)) assert(options[key] === value, 'native audio default ' + key);
    const wire = new Uint8Array(await load('c-audio-input.wav')), wave = api.Wave.read(wire, 1024);
    assert(wave.sample_rate === 16000 && wave.samples().length > 0, 'bounded original WAV read');
    rejects(() => api.Wave.read(wire, 1), 'WAV frame budget'); rejects(() => api.Wave.read(wire.slice(0, 20), 1024), 'truncated WAV');
    let valuesChecked = 0, maximumNormalizedError = 0;
    for (const [normalize, rate, name] of [[false, null, 'plain'], [true, null, 'normalized'], [false, 32000, 'up'], [false, 8000, 'down'], [true, 8000, 'normalized-down']]) {
        Object.assign(options, defaults, {normalize, output_sample_rate: rate, boundary: 1, kernel: 1});
        const copied = options.cloned(); for (const key of Object.keys(defaults)) assert(copied[key] === options[key], 'complete option clone');
        const result = api.Audio.prepare(wave, copied, () => {throw new Error('unexpected dither draw');}), actual = result.samples(), expected = floats(new Uint8Array(await load('c-audio-input.' + name + '.raw')));
        assert(result.sample_rate === (rate ?? 16000) && actual.length === expected.length, 'C audio shape');
        actual.forEach((value, index) => { const error = Math.abs(value - expected[index]) / Math.max(Math.abs(expected[index]), 1); maximumNormalizedError = Math.max(maximumNormalizedError, error); assert(error < 2e-7, 'C audio sample tolerance'); valuesChecked++; });
        const clone = result.cloned(), snapshot = actual.slice(); actual.fill(99); result.free(); equal(clone.samples(), snapshot, 'audio clone retains independent samples'); clone.free(); copied.free();
    }
    let exactDitherValues = 0;
    for (const [factory, name] of [['windows', 'windows'], ['linux_gnu', 'linux']]) {
        const bytes = new Uint8Array(await load('c-dither-' + name + '.bin')), view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        assert(view.getUint32(0, true) === 0x31485444 && view.getUint32(4, true) === 64, 'original C dither header');
        const sequence = api.DitherSequence[factory]();
        for (let index = 0; index < 64; index++) { assert(Object.is(sequence.next_uniform(), view.getFloat32(16 + index * 8, true)), 'exact C uniform sequence'); exactDitherValues++; } sequence.free();
        const fresh = api.DitherSequence[factory](), zero = new api.Wave(8000, 32, 1, 1, new Float32Array(64)); Object.assign(options, defaults, {dither_level: 1});
        const noise = api.Audio.prepare_with_sequence(zero, options, fresh), values = noise.samples();
        values.forEach((value, index) => { assert(Object.is(value, view.getFloat32(12 + 64 * 8 + index * 4, true)), 'exact C signed noise'); exactDitherValues++; });
        const continued = sequenceReference(api, factory, 64); assert(fresh.next_uniform() === continued, 'sequence consumption retained'); noise.free(); zero.free(); fresh.free();
    }
    const sequence = api.DitherSequence.linux_gnu(); Object.assign(options, defaults, {dither_level: .125});
    const dithered = api.Audio.prepare_with_sequence(wave, options, sequence), expected = floats(new Uint8Array(await load('c-audio-input.dither-linux.raw')));
    equal(bits(dithered.samples()), bits(expected), 'complete exact C WAV dither'); dithered.free(); sequence.free();
    const small = new api.Wave(8000, 32, 1, 1, new Float32Array([-.25, .5, 0])); Object.assign(options, defaults, {normalize: true, dither_level: .125});
    let calls = 0; const ordered = api.Audio.prepare(small, options, () => [0, .5, 1][calls++]); equal(ordered.samples(), [-.625, 1, .125], 'normalization precedes dither'); assert(calls === 3, 'one callback per sample'); ordered.free();
    const failure = {audio: 'callback identity'}; calls = 0; let caught;
    try { api.Audio.prepare(small, options, () => {calls++; throw failure;}); } catch (value) {caught = value;}
    assert(caught === failure && calls === 1, 'exact callback exception and immediate stop');
    for (const draw of [NaN, Infinity, -.1, 1.1, '0.5', undefined]) { calls = 0; rejects(() => api.Audio.prepare(small, options, () => {calls++; return draw;}), 'invalid random draw'); assert(calls === 1, 'invalid draw stops immediately'); }
    Object.assign(options, defaults); const silent = new api.Wave(8000, 32, 1, 1, new Float32Array([0, -0])); options.normalize = true;
    const normalized = api.Audio.prepare(silent, options, () => {throw failure;}); equal(normalized.samples(), [0, -0], 'silent normalization retains signed zero'); normalized.free(); silent.free();
    options.dither_level = -1; const skipped = api.Audio.prepare(small, options, () => {throw failure;}); skipped.free();
    const one = new api.Wave(8000, 32, 1, 1, new Float32Array([.75]));
    for (const kernel of [0, 1]) for (const boundary of [0, 1]) {
        Object.assign(options, defaults, {kernel, boundary, output_sample_rate: 16000}); const result = api.Audio.prepare(one, options, () => {throw failure;});
        assert(result.sample_rate === 16000 && result.samples().length === 2, 'both kernel/boundary combinations'); if (boundary === 0) assert(result.samples()[0] === .75, 'stable first sample'); else equal(result.samples(), [0, 0], 'legacy skipped boundary'); result.free();
    }
    for (const [key, value] of [['boundary', 2], ['kernel', 2], ['output_sample_rate', 0], ['output_sample_rate', 0xffffffff], ['dither_level', NaN]]) {
        Object.assign(options, defaults, {[key]: value}); rejects(() => api.Audio.prepare(one, options, () => .5), 'invalid audio setting ' + key);
    }
    Object.assign(options, defaults); const empty = new api.Wave(8000, 32, 1, 1, new Float32Array()); const preparedEmpty = api.Audio.prepare(empty, options, () => {throw failure;}); assert(preparedEmpty.samples().length === 0, 'empty audio'); preparedEmpty.free(); empty.free();
    const clone = small.cloned(); small.sample_rate = 0; small.bits_per_sample = 16; small.channels = 2; small.encoding = 0; small.set_samples(new Float32Array([4]));
    assert(small.sample_rate === 0 && small.bits_per_sample === 16 && small.channels === 2 && small.encoding === 0 && small.samples()[0] === 4, 'all WAV fields editable');
    rejects(() => {small.encoding = 2;}, 'invalid encoding setter'); assert(small.encoding === 0, 'encoding setter atomic'); rejects(() => new api.Wave(1, 1, 1, 2, new Float32Array()), 'invalid constructor encoding');
    rejects(() => api.Audio.prepare(small, options, () => .5), 'invalid input rate'); small.sample_rate = 8000; small.set_samples(new Float32Array([NaN])); rejects(() => api.Audio.prepare(small, options, () => .5), 'nonfinite input'); small.free();
    assert(clone.sample_rate === 8000 && clone.bits_per_sample === 32 && clone.channels === 1 && clone.encoding === 1, 'all WAV header fields cloned');
    equal(clone.samples(), [-.25, .5, 0], 'WAV clone independent'); const copiedSamples = clone.samples(); copiedSamples.fill(9); equal(clone.samples(), [-.25, .5, 0], 'WAV copied arrays'); clone.free();
    const raw = new api.Audio(0xffffffff, new Float32Array([NaN, -0, Infinity])), rawClone = raw.cloned(); raw.sample_rate = 123; raw.set_samples(new Float32Array([7])); assert(raw.sample_rate === 123 && raw.samples()[0] === 7, 'both audio fields editable'); raw.free();
    assert(rawClone.sample_rate === 0xffffffff, 'arbitrary native audio rate'); equal(rawClone.samples(), [NaN, -0, Infinity], 'arbitrary native IEEE audio'); rawClone.free();
    one.free(); cloneSafeFree(wave); options.free();
    return {originalCCases: 5, valuesChecked, maximumNormalizedError, exactDitherValues, optionFields: 5, waveFields: 5, audioFields: 2};
}
function sequenceReference(api, factory, count) { const sequence = api.DitherSequence[factory](); for (let index = 0; index < count; index++) sequence.next_uniform(); const value = sequence.next_uniform(); sequence.free(); return value; }
function cloneSafeFree(wave) { const copy = wave.cloned(); wave.free(); assert(copy.samples().length > 0, 'WAV survives source release'); copy.free(); }
