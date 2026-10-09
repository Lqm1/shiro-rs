function assert(value, message) { if (!value) throw new Error(message); }
function array(actual, expected, message) {
    assert(actual.length === expected.length, message + ' length');
    actual.forEach((value, index) => assert(Object.is(value, expected[index]), message + '[' + index + ']'));
}
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
function canonical(value) {
    if (Array.isArray(value)) return value.map(canonical);
    if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]));
    return value;
}
function json(actual, expected, message) { assert(JSON.stringify(canonical(actual)) === JSON.stringify(canonical(expected)), message); }
function bits(values) { return Array.from(new Uint32Array(values.buffer, values.byteOffset, values.length)); }
function groupSnapshot(group) {
    const observation = group.observation(), states = group.states(), streams = [];
    for (let index = 0; index < observation.streams; index++) { const stream = observation.stream(index); streams.push([stream.dimensions, bits(stream.values())]); stream.free(); }
    const result = {firstState: group.first_state, firstFrame: group.first_frame, frames: observation.frames, streams, states: JSON.parse(states.json())};
    observation.free(); states.free(); return result;
}

export async function verifyIsolation(api, load) {
    const model = api.Model.read(new Uint8Array(await load('init-c-aligned.hsmm')));
    const observation = api.Observation.from_model_rawfloat(new Uint8Array(await load('init-input.bin')), model, 12);
    const original = JSON.parse(new TextDecoder().decode(await load('align-c-isolated.json'))).file_list[0].states;
    let groupsChecked = 0, samplesChecked = 0;
    for (const mode of ['original', 'metadata-and-capping', 'same-phone-reset']) {
        const input = structuredClone(original);
        if (mode === 'metadata-and-capping') {
            input[0].time = 2.9;
            input[0].jmp = [{d: 1, p: .9, annotation: 'forward'}, {d: 4, p: .1, annotation: 'outside'}, {d: 0, p: .05, annotation: 'self'}];
            input[0].extra = {nested: [1, null, 'retained']}; input[3].ext.push({retained: true}); input[5].time = 40;
            input[3].jmp = [{d: -1, p: .1}, {d: 2, p: .2, extra: ['local']}];
        }
        if (mode === 'same-phone-reset') for (let index = 3; index < 6; index++) input[index].ext[0] = 'a';
        const states = new api.States(JSON.stringify(input)), groups = api.IsolatedGroups.split(model, observation, states), saved = groups.cloned();
        assert(groups.length === 2, mode + ' group count'); assert(groups.get(2) === undefined, 'absent group');
        states.replace('[]'); states.free(); groups.clear(); groups.free();
        for (let index = 0; index < 2; index++) {
            const group = saved.get(index), actual = groupSnapshot(group), start = index * 6, first = index * 3;
            assert(actual.firstState === first && actual.firstFrame === start && actual.frames === 6, 'original positions and capped interval');
            const expectedStates = input.slice(first, first + 3).map((state, local) => {
                const value = structuredClone(state); value.time = Math.trunc(state.time) - start;
                if (value.jmp) value.jmp = value.jmp.filter(jump => jump.d === 1 || (local + jump.d >= 0 && local + jump.d < 3));
                return value;
            });
            json(actual.states, expectedStates, 'complete local states and metadata');
            for (let stream = 0; stream < observation.streams; stream++) {
                const expected = [];
                for (let frame = start; frame < start + 6; frame++) { expected.push(...bits(observation.frame(stream, frame))); samplesChecked += observation.frame(stream, frame).length; }
                array(actual.streams[stream][1], expected, 'complete local samples');
            }
            const child = group.observation(), localStates = group.states(), detached = group.cloned(); group.free();
            localStates.replace('[]');
            const emptyStreams = new api.ObservationStreams(), emptyStream = new api.ObservationStream(1, new Float32Array());
            emptyStreams.push(emptyStream); child.replace(0, emptyStreams); emptyStream.free(); emptyStreams.free();
            json(groupSnapshot(detached), actual, 'returned children independent of group clone');
            child.free(); localStates.free(); detached.free(); groupsChecked++;
        }
        saved.free();
    }

    const states = new api.States(JSON.stringify(original)), arbitrary = new api.IsolatedGroup(0xffffffff, 0xfffffffe, observation, states);
    assert(arbitrary.first_state === 0xffffffff && arbitrary.first_frame === 0xfffffffe, 'full-width arbitrary positions');
    const expected = groupSnapshot(arbitrary), owners = new api.IsolatedGroups(); owners.push(arbitrary); const clone = owners.cloned();
    arbitrary.first_state = 7; arbitrary.first_frame = 9;
    const emptyStates = new api.States('[]'); arbitrary.set_states(emptyStates); emptyStates.free();
    const empty = new api.Observation(0, new Uint32Array([1])); arbitrary.set_observation(empty); empty.free();
    const replacementSnapshot = groupSnapshot(arbitrary); assert(replacementSnapshot.firstState === 7 && replacementSnapshot.firstFrame === 9 && replacementSnapshot.frames === 0 && replacementSnapshot.states.length === 0, 'all group fields editable');
    owners.replace(0, arbitrary); const changed = owners.get(0); json(groupSnapshot(changed), replacementSnapshot, 'complete indexed replacement'); changed.free();
    rejects(() => owners.replace(1, arbitrary), 'invalid group index'); owners.clear(); owners.free(); arbitrary.free();
    const retained = clone.get(0); clone.free(); json(groupSnapshot(retained), expected, 'copied push and collection clone');
    retained.set_observation(observation); retained.set_states(states); retained.free();

    for (const bad of [[], [{...original[0], ext: []}], [{...original[0], ext: ['a', -1]}], [{...original[0], time: 0}], [{...original[0], dur: 999}], [{...original[0], jmp: [{d: 2, p: 2}]}]]) {
        states.replace(JSON.stringify(bad)); rejects(() => api.IsolatedGroups.split(model, observation, states), 'invalid isolation input');
    }
    states.replace(JSON.stringify(original)); const noFrames = new api.Observation(0, new Uint32Array([2, 1])); rejects(() => api.IsolatedGroups.split(model, noFrames, states), 'empty acoustic interval'); noFrames.free();
    const retainedGroups = api.IsolatedGroups.split(model, observation, states); states.free(); model.free(); observation.free();
    const finalGroup = retainedGroups.get(1); retainedGroups.free(); assert(groupSnapshot(finalGroup).frames === 6, 'split survives all sources'); finalGroup.free();
    return {variants: 3, groupsChecked, samplesChecked, completeGroupFields: 4};
}
