function assert(value, message) { if (!value) throw new Error(message); }
function array(actual, expected, message) {
    assert(actual.length === expected.length, message + ' length');
    actual.forEach((value, index) => assert(Object.is(value, expected[index]), message + '[' + index + ']'));
}
function canonical(value) {
    if (Array.isArray(value)) return value.map(canonical);
    if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]));
    return value;
}
function json(actual, expected, message) { assert(JSON.stringify(canonical(actual)) === JSON.stringify(canonical(expected)), message); }
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }

export async function verifyAlignment(api, load) {
    const model = api.Model.read(new Uint8Array(await load('init-c-aligned.hsmm'))), modelWire = model.write(), raw = new Uint8Array(await load('init-input.bin'));
    const observation = api.Observation.from_model_rawfloat(raw, model, 12), observationWire = observation.write(), files = new api.FeatureFiles(); files.set('input.f', raw);
    const options = new api.AlignmentOptions(), defaults = {duration_mode: 0, isolated: false, hsmm_temperature: 1, duration_weight: 1, state_radius: 5, duration_extra: 30, duration_extra_factor: 1, geometric_temperature: 1, pruning_slope: Math.fround(.3)};
    for (const [key, value] of Object.entries(defaults)) assert(Object.is(options[key], value), 'native default ' + key);
    let statesChecked = 0;
    for (let index = 0; index < 8; index++) {
        const isolated = index >= 4, mode = index % 2, pruned = index % 4 >= 2, prefix = isolated ? 'isolated' : 'embedded';
        Object.assign(options, defaults, {isolated, duration_mode: mode, pruning_slope: pruned ? .5 : .8, state_radius: pruned ? 2 : 5, duration_extra: pruned ? 8 : 30});
        const input = JSON.parse(new TextDecoder().decode(await load('align-c-' + prefix + '.json'))), reference = JSON.parse(new TextDecoder().decode(await load('align-c-' + prefix + (mode ? '-hmm' : '-hsmm') + (pruned ? '-pruned' : '') + '.json')));
        for (const state of [...input.file_list[0].states, ...reference.file_list[0].states]) state.customState = {nested: [true, null, 'retained']};
        const states = new api.States(JSON.stringify(input.file_list[0].states)), copiedOptions = options.cloned();
        for (const key of Object.keys(defaults)) assert(Object.is(copiedOptions[key], options[key]), 'complete option clone');
        const aligned = model.align_states(observation, states, copiedOptions); json(JSON.parse(aligned.json()), reference.file_list[0].states, 'original C complete path ' + index); statesChecked += reference.file_list[0].states.length;
        input.attributes = {nested: [true, null, 'retained']}; input.file_list[0].custom = ['file', 2];
        const document = new api.SegmentationDocument(JSON.stringify(input)), original = document.json(), result = model.align_document(document, files, 12, copiedOptions);
        const expected = structuredClone(input); expected.file_list[0].states = reference.file_list[0].states;
        json(JSON.parse(result.json()), expected, 'document alignment retains every attribute'); assert(document.json() === original, 'source document unchanged');
        const saved = result.cloned(); result.free(); json(JSON.parse(saved.json()), expected, 'aligned document survives source release'); saved.free(); document.free();
        json(JSON.parse(states.json()), input.file_list[0].states, 'source states unchanged');
        for (const owner of [states, copiedOptions, aligned]) owner.free();
    }
    Object.assign(options, defaults, {duration_mode: 1});
    const fourInput = JSON.parse(new TextDecoder().decode(await load('align-c-four.json'))), fourExpected = JSON.parse(new TextDecoder().decode(await load('align-c-four-hmm.json'))), four = new api.States(JSON.stringify(fourInput.file_list[0].states)), fourResult = model.align_states(observation, four, options);
    json(JSON.parse(fourResult.json()), fourExpected.file_list[0].states, 'original C default HMM pruning'); statesChecked += fourExpected.file_list[0].states.length; fourResult.free(); four.free();
    const embeddedInput = JSON.parse(new TextDecoder().decode(await load('align-c-embedded.json'))), embedded = new api.States(JSON.stringify(embeddedInput.file_list[0].states));
    rejects(() => model.align_states(observation, embedded, options), 'native no-path failure');

    Object.assign(options, defaults, {isolated: true}); const isolatedInput = JSON.parse(new TextDecoder().decode(await load('align-c-isolated.json'))), originalIsolated = new api.States(JSON.stringify(isolatedInput.file_list[0].states)), baseline = model.align_states(observation, originalIsolated, options), baselineJson = baseline.json(); baseline.free();
    isolatedInput.file_list[0].states[2].jmp = [{d: 2, p: .9}, {d: 0, p: 0}]; const enhanced = new api.States(JSON.stringify(isolatedInput.file_list[0].states)), filtered = model.align_states(observation, enhanced, options); json(JSON.parse(filtered.json()), JSON.parse(baselineJson), 'out-of-group transition filtering'); filtered.free();
    delete isolatedInput.file_list[0].states[2].jmp; isolatedInput.file_list[0].states[5].time = 999; enhanced.replace(JSON.stringify(isolatedInput.file_list[0].states)); const capped = model.align_states(observation, enhanced, options); assert(JSON.parse(capped.json()).at(-1).time === 12, 'capped isolated interval'); capped.free();
    isolatedInput.file_list[0].states[5].time = 5; enhanced.replace(JSON.stringify(isolatedInput.file_list[0].states)); rejects(() => model.align_states(observation, enhanced, options), 'nonpositive isolated interval');
    enhanced.replace('[{"time":1,"dur":0,"out":[0,0],"ext":[]}]'); rejects(() => model.align_states(observation, enhanced, options), 'missing isolation identity');
    const repeatedStates = structuredClone(embeddedInput.file_list[0].states); for (const state of repeatedStates) { state.ext = ['repeated', 0]; delete state.jmp; }
    enhanced.replace(JSON.stringify(repeatedStates)); const repeated = model.align_states(observation, enhanced, options); array(JSON.parse(repeated.json()).map(state => state.time), [3, 7, 12], 'same-name local index reset intervals'); repeated.free();

    const baseDocument = new api.SegmentationDocument(JSON.stringify(embeddedInput)); Object.assign(options, defaults);
    for (const [key, value] of [['duration_mode', 2], ['hsmm_temperature', -1], ['duration_weight', -1]]) { options[key] = value; rejects(() => model.align_states(observation, embedded, options), 'invalid alignment setting ' + key); Object.assign(options, defaults); }
    options.hsmm_temperature = 0; const zeroTemperature = model.align_states(observation, embedded, options); zeroTemperature.free(); Object.assign(options, defaults);
    const emptyObservation = new api.Observation(0, new Uint32Array([2, 1])); rejects(() => model.align_states(emptyObservation, embedded, options), 'empty acoustic interval'); emptyObservation.free();
    const emptyStates = new api.States('[]'); rejects(() => model.align_states(observation, emptyStates, options), 'empty states'); emptyStates.free();
    rejects(() => model.align_document(baseDocument, files, 11, options), 'document frame budget'); files.set('input.f', new Uint8Array([1, 2, 3])); rejects(() => model.align_document(baseDocument, files, 12, options), 'malformed feature file'); files.clear(); rejects(() => model.align_document(baseDocument, files, 12, options), 'missing feature file'); files.set('input.f', raw);
    const manyInput = structuredClone(embeddedInput); manyInput.file_list.push(structuredClone(manyInput.file_list[0])); const many = new api.SegmentationDocument(JSON.stringify(manyInput)), manyResult = model.align_document(many, files, 12, options); assert(JSON.parse(manyResult.json()).file_list.length === 2, 'prepared corpus reuse and repeated file order'); manyResult.free(); many.free();
    const copiedModel = model.cloned(), unusedDuration = new api.Duration(); copiedModel.set_duration(3, unusedDuration); unusedDuration.free();
    const unusedResult = copiedModel.align_states(observation, embedded, options); unusedResult.free();
    const activeDuration = copiedModel.duration(0), activeValues = activeDuration.values(); activeValues[1] = 0; activeDuration.set_values(activeValues); copiedModel.set_duration(0, activeDuration); activeDuration.free();
    rejects(() => copiedModel.align_states(observation, embedded, options), 'invalid selected explicit duration'); options.duration_mode = 1; options.pruning_slope = .8;
    const geometricWithoutDensity = copiedModel.align_states(observation, embedded, options); geometricWithoutDensity.free(); copiedModel.free(); Object.assign(options, defaults);
    array(model.write(), modelWire, 'source model unchanged'); array(observation.write(), observationWire, 'source observation unchanged');
    const finalStates = model.align_states(observation, embedded, options), expectedFinal = finalStates.json();
    for (const owner of [baseDocument, embedded, enhanced, originalIsolated, observation, model, files, options]) owner.free(); assert(finalStates.json() === expectedFinal, 'alignment result independent of all inputs'); finalStates.free();
    return {originalCModes: 9, statesChecked, optionFields: 9, completeDocumentMetadata: true};
}
