function assert(value, message) { if (!value) throw new Error(message); }
function equal(a, b, message) { assert(Object.is(a, b), `${message}: ${a} != ${b}`); }
function array(a, b, message) { equal(a.length, b.length, message + ' length'); a.forEach((value, index) => equal(value, b[index], message + '[' + index + ']')); }
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
function bits(values) { return Array.from(new Uint32Array(values.buffer, values.byteOffset, values.length)); }
function snapshot(model) {
    const streams = [], durations = [];
    for (let index = 0; index < model.streams; index++) {
        const stream = model.stream(index), emissions = [];
        for (let state = 0; state < stream.emissions; state++) {
            const gaussian = stream.emission(state);
            emissions.push({dimensions: gaussian.dimensions, components: gaussian.components, weights: bits(gaussian.weights()), means: bits(gaussian.means()), variances: bits(gaussian.variances()), variance_floors: bits(gaussian.variance_floors())}); gaussian.free();
        }
        streams.push({weight: bits(stream.weight()), emissions}); stream.free();
    }
    for (let index = 0; index < model.durations; index++) { const duration = model.duration(index); durations.push({values: bits(duration.values()), constraints: Array.from(duration.constraints())}); duration.free(); }
    return {streams, durations};
}
function budget(model, encoding) {
    let total = 2 + model.streams + model.durations * 7;
    for (let index = 0; index < model.streams; index++) {
        const stream = model.stream(index); total += 2 + stream.emissions;
        for (let state = 0; state < stream.emissions; state++) { const gaussian = stream.emission(state); total += 4 + gaussian.components + gaussian.components * gaussian.dimensions * (encoding ? 2 : 3); gaussian.free(); }
        stream.free();
    }
    return total;
}

export async function verifyModels(api, load) {
    const definition = new api.ModelDefinition(new TextDecoder().decode(await load('modeldef.json'))), built = definition.build();
    const originalEmpty = new Uint8Array(await load('empty-c.hsmm')); array(built.write(), originalEmpty, 'original C model construction');
    array(built.dimensions(), [2, 1], 'complete ordered heterogeneous dimensions');
    const detachedDimensions = built.dimensions(); detachedDimensions.fill(99); array(built.dimensions(), [2, 1], 'copied model dimensions');
    const saved = built.cloned(); definition.replace('{"ndurstate":1,"streamdef":[]}'); rejects(() => definition.build(), 'invalid definition');
    definition.free(); built.free(); array(saved.write(), originalEmpty, 'constructed model source independence'); saved.free();
    let wireBytes = originalEmpty.length, modelsChecked = 0;
    const fixtures = [['empty-c.hsmm', 0], ['cmu-arctic-all-speakers.hsmm', 1], ['init-c-multi.hsmm', 0], ['untie-c-weighted-input.hsmm', 0], ['rest-c-isolated-daem.hsmm', 0], ['utterances-c-trained.hsmm', 0]];
    for (const [filename, encoding] of fixtures) {
        const bytes = new Uint8Array(await load(filename)), model = api.Model.read(bytes), expected = snapshot(model), entryBudget = budget(model, encoding);
        array(model.write_with_encoding(encoding), bytes, 'complete original C wire ' + filename); wireBytes += bytes.length;
        const bounded = api.Model.read_with_limits(bytes, entryBudget); equal(JSON.stringify(snapshot(bounded)), JSON.stringify(expected), 'bounded complete model'); bounded.free();
        rejects(() => api.Model.read_with_limits(bytes, entryBudget - 1), 'one-entry-short budget'); rejects(() => model.write_with_encoding(2), 'invalid encoding');
        const joined = new Uint8Array(bytes.length * 2); joined.set(bytes); joined.set(bytes, bytes.length); rejects(() => api.Model.read(joined), 'strict sequential data');
        const first = api.Model.read_prefix_with_limits(joined, entryBudget); equal(first.position, BigInt(bytes.length), 'prefix byte position');
        const copy = first.parameters(), moved = first.into_parameters(); equal(JSON.stringify(snapshot(copy)), JSON.stringify(expected), 'prefix copied parameters'); copy.free(); moved.free();
        const second = api.Model.read_prefix(joined.subarray(bytes.length)), recovered = second.into_parameters(); equal(JSON.stringify(snapshot(recovered)), JSON.stringify(expected), 'second sequential model'); recovered.free();
        rejects(() => api.Model.read_prefix_with_limits(joined, entryBudget - 1), 'prefix budget');
        const ends = filename === 'empty-c.hsmm' ? Array.from({length: bytes.length}, (_, index) => index) : [0, 1, 2, Math.floor(bytes.length / 2), bytes.length - 1];
        for (const end of ends) rejects(() => api.Model.read_prefix(bytes.subarray(0, end)), 'truncated model');
        const current = model.write(), reread = api.Model.read(current); equal(JSON.stringify(snapshot(reread)), JSON.stringify(expected), 'complete current schema roundtrip'); reread.free();
        const cloned = model.cloned(); model.free(); equal(JSON.stringify(snapshot(cloned)), JSON.stringify(expected), 'all fields survive source release'); cloned.free(); modelsChecked++;
    }

    const duration = new api.Duration(), durationBits = new Uint32Array([0xffc00123, 0x80000000, 0x7f800000]);
    duration.set_values(new Float32Array(durationBits.buffer)); duration.set_constraints(new Int32Array([-7, 12, 4]));
    rejects(() => duration.set_values(new Float32Array([1])), 'duration value shape'); array(bits(duration.values()), durationBits, 'all duration IEEE fields');
    rejects(() => duration.set_constraints(new Int32Array([1])), 'duration constraint shape'); array(duration.constraints(), [-7, 12, 4], 'all duration constraints');
    const gaussian = new api.Gaussian(1, 1), scalarBits = new Uint32Array([0xffc00123, 0x80000000]), raw = new Float32Array(scalarBits.buffer);
    gaussian.replace(2, new Float32Array([1]), raw, new Float32Array([2, 3]), raw);
    array(bits(gaussian.means()), scalarBits, 'Gaussian IEEE means'); array(bits(gaussian.variance_floors()), scalarBits, 'Gaussian IEEE floors');
    const detached = gaussian.means(); detached.fill(123); array(bits(gaussian.means()), scalarBits, 'detached Gaussian array');
    rejects(() => gaussian.replace(1, new Float32Array([1]), raw, raw, raw), 'Gaussian shape'); equal(gaussian.dimensions, 2, 'atomic Gaussian dimensions');
    const gaussians = new api.Gaussians(); gaussians.push(gaussian); const gaussianCopy = gaussians.cloned();
    const finite = new api.Gaussian(1, 1); finite.replace(1, new Float32Array([1]), new Float32Array([2]), new Float32Array([3]), new Float32Array([.25])); gaussians.replace(0, finite);
    rejects(() => gaussians.replace(1, finite), 'Gaussian collection index'); const stream = new api.Stream(0, 0, 0);
    stream.replace(new Float32Array(new Uint32Array([0x80000000]).buffer), gaussians); const streams = new api.Streams(), durations = new api.Durations(); streams.push(stream); durations.push(duration);
    const model = new api.Model(); model.replace(streams, durations); const original = snapshot(model);
    const streamsClone = streams.cloned(), durationsClone = durations.cloned();
    streams.push(stream); durations.push(duration); model.replace(streams, durations);
    equal(model.streams, 2, 'whole model stream growth'); equal(model.durations, 2, 'whole model duration growth');
    model.replace(streamsClone, durationsClone); streamsClone.free(); durationsClone.free();
    equal(model.streams, 1, 'whole model stream shrink'); equal(model.durations, 1, 'whole model duration shrink');
    model.set_stream(0, stream); model.set_duration(0, duration);
    equal(JSON.stringify(snapshot(model)), JSON.stringify(original), 'complete child replacement');
    rejects(() => stream.set_weight(new Float32Array()), 'stream weight shape'); rejects(() => stream.replace(new Float32Array(), gaussianCopy), 'stream replacement shape');
    array(bits(stream.weight()), [0x80000000], 'retained stream weight bits'); rejects(() => stream.set_emission(1, finite), 'emission index');
    rejects(() => model.set_stream(1, stream), 'model stream index'); rejects(() => model.set_duration(1, duration), 'model duration index');
    equal(JSON.stringify(snapshot(model)), JSON.stringify(original), 'failed replacements preserve full model');
    rejects(() => model.write_with_encoding(1), 'historical floors cannot be dropped');
    const wire = model.write(), reread = api.Model.read(wire); equal(JSON.stringify(snapshot(reread)), JSON.stringify(original), 'all IEEE fields survive native wire'); reread.free();
    const streamChild = model.stream(0), durationChild = model.duration(0); equal(model.stream(1), null, 'absent stream'); equal(model.duration(1), null, 'absent duration');
    for (const owner of [duration, gaussian, finite, gaussians, stream, streams, durations]) owner.free(); model.free();
    const emissionChild = streamChild.emission(0); streamChild.free(); array(emissionChild.means(), [2], 'nested child release'); emissionChild.free(); array(bits(durationChild.values()), durationBits, 'duration child release'); durationChild.free();
    const rawCopy = gaussianCopy.get(0); gaussianCopy.clear(); gaussianCopy.free(); array(bits(rawCopy.means()), scalarBits, 'Gaussian collection clone retention'); rawCopy.free();
    const emptyModel = new api.Model(), emptyStreams = new api.Streams(), emptyDurations = new api.Durations(); emptyModel.replace(emptyStreams, emptyDurations); emptyModel.validate();
    equal(emptyModel.streams, 0, 'empty model streams'); equal(emptyModel.durations, 0, 'empty model durations'); emptyStreams.free(); emptyDurations.free(); emptyModel.free();
    const noStreams = new api.Model(); rejects(() => noStreams.dimensions(), 'dimensions require streams'); noStreams.free();
    const noEmissions = new api.Model(), loneStream = new api.Stream(0, 0, 0), loneStreams = new api.Streams(), noDurations = new api.Durations(); loneStreams.push(loneStream); noEmissions.replace(loneStreams, noDurations);
    rejects(() => noEmissions.dimensions(), 'dimensions require emission states'); noEmissions.free(); loneStream.free(); loneStreams.free(); noDurations.free();
    return {modelsChecked, wireBytes, parameterKinds: 4, collectionKinds: 3};
}
