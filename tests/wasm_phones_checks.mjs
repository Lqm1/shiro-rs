function assert(value, message) { if (!value) throw new Error(message); }
function equal(actual, expected, message) { assert(Object.is(actual, expected), `${message}: ${actual} != ${expected}`); }
function compare(actual, expected, message) {
    if (typeof expected === 'number') {
        assert(typeof actual === 'number' && Math.abs(actual - expected) <= 1e-14, message);
    } else if (Array.isArray(expected)) {
        assert(Array.isArray(actual), message); equal(actual.length, expected.length, message + ' length');
        expected.forEach((value, index) => compare(actual[index], value, `${message}[${index}]`));
    } else if (expected && typeof expected === 'object') {
        equal(Object.keys(actual).sort().join(','), Object.keys(expected).sort().join(','), message + ' fields');
        for (const key of Object.keys(expected)) compare(actual[key], expected[key], message + '.' + key);
    } else equal(actual, expected, message);
}
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }

export async function verifyPhones(api, load) {
    const text = async name => new TextDecoder().decode(await load(name));
    const cases = JSON.parse(await text('phones-original.json')), input = await text('phones-input.txt');
    let statesChecked = 0, definitionsChecked = 0;
    const names = ['bb', 'aa', 'bb', 'cc', 'aa'];
    for (const fixture of cases) {
        const options = new api.PhoneMapOptions();
        equal(options.states_per_phone, 3, 'default state count'); equal(options.streams, 3, 'default stream count');
        equal(options.topology, undefined, 'absent topology'); equal(options.weak_skips, false, 'default weak skips');
        options.states_per_phone = fixture.count; options.streams = 2; options.topology = fixture.topology; options.weak_skips = true;
        const copied = options.cloned(); options.states_per_phone = 1; options.streams = 1; options.topology = undefined; options.weak_skips = false;
        equal(copied.states_per_phone, fixture.count, 'copied state count'); equal(copied.streams, 2, 'copied stream count');
        equal(copied.topology, fixture.topology, 'copied topology'); equal(copied.weak_skips, true, 'copied weak skips'); options.free();
        const map = api.PhoneMap.create(input, copied); copied.free(); compare(JSON.parse(map.json()), fixture.map, 'original Lua phone map');
        const before = map.json(), states = map.initial(names, 41);
        compare(JSON.parse(states.json()), fixture.segmentation.file_list[0].states, 'original Lua complete states/jumps');
        statesChecked += fixture.segmentation.file_list[0].states.length;
        if (fixture.definition) {
            const definition = map.to_definition(12, .01), expected = structuredClone(fixture.definition);
            expected.dur_attr.sort((a, b) => a.index - b.index);
            compare(JSON.parse(definition.json()), expected, 'original Lua complete model definition'); definitionsChecked++;
            const duplicate = definition.cloned(); definition.replace('{"ndurstate":1,"streamdef":[]}'); definition.free();
            compare(JSON.parse(duplicate.json()), expected, 'independent definition snapshot'); duplicate.free();
        }
        equal(map.json(), before, 'calculations preserve phone map'); const clone = map.cloned(); map.free();
        compare(JSON.parse(states.json()), fixture.segmentation.file_list[0].states, 'initial states survive source release');
        states.free(); clone.free();
    }
    equal(cases.length, 7, 'all original topology cases');
    const options = new api.PhoneMapOptions(), defaults = api.PhoneMap.create('aa\nbb\n', options);
    const defaultValue = JSON.parse(defaults.json()); equal(defaultValue.phone_map.bb.states[0].dur, 3, 'input order IDs');
    compare(defaultValue.phone_map.aa.states[0].out, [0, 0, 0], 'default outputs');
    compare(defaultValue.phone_map.aa.topology, undefined, 'default topology attribute absent');
    for (const topology of ['', 'unrecognized', 'type-c']) {
        options.topology = topology; const map = api.PhoneMap.create('aa', options); equal(JSON.parse(map.json()).phone_map.aa.topology, topology, 'present topology including empty'); map.free();
    }
    options.topology = undefined;
    for (const invalid of ['aa\naa', 'aa durfloor', 'aa durceil NaN', 'aa durfloor inf']) rejects(() => api.PhoneMap.create(invalid, options), 'invalid phone input');
    for (const field of ['states_per_phone', 'streams']) {
        const previous = options[field]; for (const value of [0, 0x80000000]) { options[field] = value; rejects(() => api.PhoneMap.create('aa', options), 'invalid option ' + field); } options[field] = previous;
    }
    const empty = api.PhoneMap.create('', options), noStates = empty.initial([], 0); compare(JSON.parse(noStates.json()), [], 'empty state sequence'); empty.free(); noStates.free();
    rejects(() => defaults.initial(['unknown'], 10), 'unknown phone'); rejects(() => defaults.initial(['aa'], 0x80000000), 'legacy frame range');
    for (const hop of [0, -1, NaN, Infinity]) rejects(() => defaults.to_definition(12, hop), 'invalid definition hop');
    for (const dimensions of [0, 0x80000000]) rejects(() => defaults.to_definition(dimensions, .01), 'invalid dimensions');
    const shared = {
        phone_map: {
            aa: {states: [{dur: 2, out: [4, 0], state_extra: true}], durfloor: [.02], durceil: [.1], phone_extra: 'kept'},
            bb: {states: [{dur: 2, out: [1, 6]}], durfloor: [.03], durceil: [.08]},
        }, source: {extra: true},
    };
    const map = new api.PhoneMap(JSON.stringify(shared)), original = map.json(), definition = map.to_definition(12, .01);
    compare(JSON.parse(definition.json()), {ndurstate: 3, streamdef: [{nstate: 5, ndim: 12, nmix: 1, weight: 1}, {nstate: 7, ndim: 12, nmix: 1, weight: 1}], dur_attr: [{index: 2, floor: 3, ceil: 8}]}, 'intersected shared duration and independent output counts');
    definition.free(); equal(map.json(), original, 'attributes retained after definition');
    shared.phone_map.bb.durceil = [-1]; map.replace(JSON.stringify(shared)); const inactive = map.to_definition(12, .01);
    equal(JSON.parse(inactive.json()).dur_attr[0].ceil, 10, 'inactive shared ceiling'); inactive.free();
    shared.phone_map.bb.durceil = [.08]; shared.phone_map.bb.durfloor = [.2]; map.replace(JSON.stringify(shared));
    const contradictory = map.json(); rejects(() => map.to_definition(12, .01), 'contradictory duration'); equal(map.json(), contradictory, 'failed calculation preserves map');
    const invalidMaps = [
        {phone_map: {aa: {states: [{dur: 0, out: [0]}], pskip: 1.1}}},
        {phone_map: {aa: {states: [{dur: 0, out: []}]}}},
        {phone_map: {aa: {states: [{dur: 0, out: [0]}]}, bb: {states: [{dur: 1, out: [0, 1]}]}}},
    ];
    for (let index = 0; index < invalidMaps.length; index++) {
        const bad = new api.PhoneMap(JSON.stringify(invalidMaps[index]));
        if (index === 0) rejects(() => bad.initial(['aa'], 10), 'invalid skip probability');
        else rejects(() => bad.to_definition(12, .01), 'invalid output layout');
        bad.free();
    }
    map.free(); defaults.free(); options.free();
    return {luaCases: cases.length, statesChecked, definitionsChecked, optionFields: 4};
}
