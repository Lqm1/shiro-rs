function assert(value, message) { if (!value) throw new Error(message); }
function array(actual, expected, message) {
    assert(actual.length === expected.length, message + ' length');
    actual.forEach((value, index) => assert(Object.is(value, expected[index]), message + '[' + index + ']'));
}
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
function bits(values) { return new Uint32Array(values.buffer, values.byteOffset, values.length); }
function snapshot(segmentation) {
    const outputs = [], outgoing = [];
    for (let stream = 0; stream < segmentation.streams; stream++) outputs.push(Array.from(segmentation.output_states(stream)));
    for (let state = 0; state < segmentation.segments; state++) {
        const jumps = segmentation.outgoing(state);
        outgoing.push([Array.from(jumps.deltas()), Array.from(bits(jumps.probabilities()))]); jumps.free();
    }
    return JSON.stringify([Array.from(segmentation.boundaries()), Array.from(segmentation.duration_states()), outputs, outgoing]);
}

export async function verifyData(api, load) {
    const definition = new api.ModelDefinition(new TextDecoder().decode(await load('init-definition.json'))), model = definition.build(); definition.free();
    const bytes = new Uint8Array(await load('init-input.bin'));
    const observation = api.Observation.from_model_rawfloat(bytes, model, 12);
    assert(observation.frames === 12 && observation.streams === 2, 'original C observation shape');
    array(observation.frame(0, 0), [-5, -1.5], 'original C first stream');
    array(observation.frame(1, 0), [0], 'original C second stream');
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    for (let frame = 0; frame < 12; frame++) {
        array(bits(observation.frame(0, frame)), [view.getUint32(frame * 12, true), view.getUint32(frame * 12 + 4, true)], 'ordered first stream bits');
        array(bits(observation.frame(1, frame)), [view.getUint32(frame * 12 + 8, true)], 'ordered second stream bits');
    }
    const hugeBudget = api.Observation.read_rawfloat(bytes, new Uint32Array([2, 1]), 0x7fffffff);
    array(hugeBudget.write(), observation.write(), 'large budget with small input'); hugeBudget.free();
    for (const bad of [bytes.subarray(0, 3), bytes.subarray(0, 8)]) rejects(() => api.Observation.read_rawfloat(bad, new Uint32Array([2, 1]), 12), 'partial frame');
    rejects(() => api.Observation.from_model_rawfloat(bytes, model, 11), 'frame budget');
    for (const dimensions of [[], [0], [0x80000000]]) rejects(() => new api.Observation(0, new Uint32Array(dimensions)), 'invalid observation dimensions');
    assert(observation.frame(2, 0) === null && observation.frame(0, 12) === null && observation.stream(2) === null, 'absent observation values');
    const child = observation.stream(0), copy = observation.cloned(), before = observation.write();
    child.replace(1, new Float32Array([1])); rejects(() => observation.set_stream(0, child), 'invalid stream replacement'); array(observation.write(), before, 'atomic observation replacement');
    const ieeeBits = new Uint32Array([0, 0x80000000, 1, 0x80000001, 0x7f800000, 0xffc00123]);
    const ieeeBytes = new Uint8Array(ieeeBits.buffer), ieee = api.Observation.read_rawfloat(ieeeBytes, new Uint32Array([2, 1]), 2);
    array(bits(ieee.frame(0, 0)), ieeeBits.subarray(0, 2), 'IEEE first frame'); array(bits(ieee.frame(1, 1)), ieeeBits.subarray(5, 6), 'IEEE NaN payload');
    const streams = new api.ObservationStreams(), raw = new api.ObservationStream(3, new Float32Array(ieeeBits.buffer)); streams.push(raw);
    const streamsCopy = streams.cloned(), rawCopy = raw.cloned(); streams.replace(0, rawCopy); raw.free(); rawCopy.free();
    observation.replace(2, streams); assert(observation.frames === 2 && observation.streams === 1, 'whole observation shape replacement');
    const detached = observation.frame(0, 1); detached.fill(4); array(bits(observation.frame(0, 1)), ieeeBits.subarray(3), 'detached observation frame');
    const streamCopy = streamsCopy.get(0); array(bits(streamCopy.values()), ieeeBits, 'stream collection copies'); streamCopy.free(); streamsCopy.free();
    rejects(() => streams.replace(1, child), 'collection invalid index'); streams.clear(); rejects(() => observation.replace(0, streams), 'empty stream collection'); streams.free();
    child.free(); observation.free(); assert(copy.frames === 12, 'observation survives source release'); copy.validate(); copy.free(); ieee.free();
    const empty = new api.Observation(0, new Uint32Array([1])); empty.validate(); empty.free();
    const many = api.Observation.read_rawfloat(new Uint8Array(280), new Uint32Array(70).fill(1), 1); assert(many.streams === 70, 'no artificial stream ceiling'); many.free();

    const document = JSON.parse(new TextDecoder().decode(await load('init-segmentation.json'))), originalStates = document.file_list[0].states;
    const states = new api.States(JSON.stringify(originalStates)), segmentation = api.Segmentation.from_states(states, model);
    array(segmentation.boundaries(), [3, 7, 12], 'original C boundaries'); array(segmentation.duration_states(), [0, 1, 2], 'duration states');
    for (let stream = 0; stream < 2; stream++) array(segmentation.output_states(stream), [0, 1, 2], 'output states');
    const jumps = segmentation.outgoing(0); array(jumps.deltas(), [2, 3, 1], 'original C jump deltas');
    array(bits(jumps.probabilities()), [0x3ca3d70a, 0x3e19999a, 0x3f547ae2], 'original C jump binary32 values'); jumps.free();
    originalStates[0].time = 3.9; originalStates[0].jmp.push({d: 1, p: .9}); states.replace(JSON.stringify(originalStates));
    const fractional = api.Segmentation.from_states(states, model); array(fractional.boundaries(), [3, 7, 12], 'time truncation');
    const skipped = fractional.outgoing(0); assert(skipped.length === 3, 'explicit forward jump excluded'); skipped.free(); fractional.free();
    originalStates[0].jmp = [{d: 0, p: .1}, {d: -1, p: .2}]; states.replace(JSON.stringify(originalStates));
    const backward = api.Segmentation.from_states(states, model), backwardJumps = backward.outgoing(0);
    array(backwardJumps.deltas(), [0, -1, 1], 'self and backward jumps'); array(bits(backwardJumps.probabilities()), [0x3dcccccd, 0x3e4ccccd, 0x3f333333], 'residual operation order'); backwardJumps.free(); backward.free();
    for (const bad of [{time: -1}, {dur: 999}, {out: [0]}, {jmp: [{d: 2, p: .8}, {d: 3, p: .8}]}]) {
        states.replace(JSON.stringify([{...originalStates[0], ...bad}])); rejects(() => api.Segmentation.from_states(states, model), 'invalid state import');
    }
    const saved = segmentation.cloned(), savedSnapshot = snapshot(saved);
    const boundaries = segmentation.boundaries(); boundaries.fill(99); assert(snapshot(segmentation) === savedSnapshot, 'detached segmentation fields');
    segmentation.set_boundaries(new Int32Array([2, 5, 11])); segmentation.set_duration_states(new Int32Array([-1, 2, 3])); segmentation.set_output_states(0, new Int32Array([3, 2, 1]));
    const replacement = new api.JumpGroup(new Int32Array([-2, 1]), new Float32Array([.25, .75])), replacementCopy = replacement.cloned(); segmentation.set_outgoing(1, replacement); replacement.free(); replacementCopy.free();
    const edited = snapshot(segmentation);
    for (const run of [() => segmentation.set_boundaries(new Int32Array(1)), () => segmentation.set_duration_states(new Int32Array(1)), () => segmentation.set_output_states(0, new Int32Array(1)), () => segmentation.set_output_states(2, new Int32Array(3))]) rejects(run, 'invalid segmentation field shape');
    const invalidJumps = new api.JumpGroup(new Int32Array([0]), new Float32Array([1])); rejects(() => segmentation.set_outgoing(0, invalidJumps), 'missing forward transition'); invalidJumps.free();
    rejects(() => new api.JumpGroup(new Int32Array(1), new Float32Array(0)), 'jump arrays differ');
    assert(snapshot(segmentation) === edited, 'failed segmentation edits preserve all fields'); segmentation.validate(); segmentation.write();
    const zero = new api.Segmentation(0, 0); segmentation.replace(zero); assert(segmentation.segments === 0 && segmentation.streams === 0, 'whole segmentation shrink'); segmentation.replace(saved); assert(snapshot(segmentation) === savedSnapshot, 'whole segmentation recovery');
    assert(segmentation.outgoing(3) === null, 'absent outgoing group'); rejects(() => new api.Segmentation(0x80000000, 0), 'legacy stream size');
    for (const owner of [zero, segmentation, states, model]) owner.free(); assert(snapshot(saved) === savedSnapshot, 'segmentation survives source release'); saved.free();
    return {referenceFrames: 12, referenceSegments: 3, referenceJumpValues: 3, ieeeValues: ieeeBits.length, streamsWithoutArtificialCeiling: 70};
}
