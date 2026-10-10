function assert(value, message) { if (!value) throw new Error(message); }
function array(actual, expected, message) {
    assert(actual.length === expected.length, message + ' length');
    actual.forEach((value, index) => assert(Object.is(value, expected[index]), message + '[' + index + ']'));
}
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }

export async function verifyInitialization(api, load) {
    const definition = new api.ModelDefinition(new TextDecoder().decode(await load('init-definition.json'))), model = definition.build(); definition.free();
    const raw = new Uint8Array(await load('init-input.bin'));
    const states = new api.States(JSON.stringify(JSON.parse(new TextDecoder().decode(await load('init-segmentation.json'))).file_list[0].states));
    const observation = api.Observation.from_model_rawfloat(raw, model, 12), segmentation = api.Segmentation.from_states(states, model);
    const observations = new api.Observations(), segmentations = new api.Segmentations(), dataset = new api.Dataset();
    observations.push(observation); segmentations.push(segmentation); dataset.replace(observations, segmentations);
    const originalModel = model.write(), originalObservation = observation.write(), originalSegmentation = segmentation.write();
    const options = new api.InitializationOptions(); assert(!options.flat_start && !options.globally_tied && options.variance_floor_ratio === Math.fround(.1), 'all native option defaults');
    let wireBytes = 0, cModels = 0;
    for (const [flat, tied, floor, filename] of [[false, false, .1, 'init-c-aligned.hsmm'], [true, false, .1, 'init-c-flat.hsmm'], [false, true, .1, 'init-c-tied.hsmm'], [true, true, .4, 'init-c-flat-tied.hsmm']]) {
        options.flat_start = flat; options.globally_tied = tied; options.variance_floor_ratio = floor;
        const copiedOptions = options.cloned(); assert(copiedOptions.flat_start === flat && copiedOptions.globally_tied === tied && copiedOptions.variance_floor_ratio === Math.fround(floor), 'complete option clone');
        const initialized = model.initialize(dataset, copiedOptions), expected = new Uint8Array(await load(filename)); copiedOptions.free();
        array(initialized.write(), expected, 'complete original C initialization ' + filename);
        const recovered = api.Model.read(initialized.write()); array(recovered.write(), expected, 'initialized model reload'); initialized.free(); recovered.free();
        wireBytes += expected.length; cModels++;
        array(model.write(), originalModel, 'source model unchanged'); array(observation.write(), originalObservation, 'source observation unchanged'); array(segmentation.write(), originalSegmentation, 'source segmentation unchanged');
    }
    const ten = api.Observation.from_model_rawfloat(raw.subarray(0, 120), model, 10); observations.replace(0, ten); dataset.set_observations(observations);
    options.flat_start = true; options.globally_tied = false; options.variance_floor_ratio = .1;
    const tenModel = model.initialize(dataset, options), tenExpected = new Uint8Array(await load('init-c-flat-ten.hsmm'));
    array(tenModel.write(), tenExpected, 'C binary32 flat-start duration rounding'); wireBytes += tenExpected.length; cModels++; tenModel.free(); ten.free();

    observations.replace(0, observation); observations.push(observation); segmentations.push(segmentation);
    const capped = segmentation.cloned(); capped.set_boundaries(new Int32Array([3, 7, 999])); segmentations.replace(1, capped); capped.free(); dataset.replace(observations, segmentations);
    options.flat_start = false;
    const corpus = model.initialize(dataset, options), expectedCorpus = api.Model.read(new Uint8Array(await load('init-c-multi.hsmm'))), duration = expectedCorpus.duration(3);
    const values = duration.values(); array(values.subarray(0, 2), [8, 64], 'original C overwritten corpus count'); values[0] = 4; values[1] = 16; duration.set_values(values); expectedCorpus.set_duration(3, duration); duration.free();
    array(corpus.write(), expectedCorpus.write(), 'corrected full-corpus fallback and boundary capping'); corpus.free(); expectedCorpus.free();

    const copiedData = dataset.cloned(), copiedObservations = copiedData.observations(), copiedSegmentations = copiedData.segmentations();
    assert(copiedObservations.length === 2 && copiedSegmentations.length === 2, 'complete sample arrays');
    const firstObservation = copiedObservations.get(0), firstSegmentation = copiedSegmentations.get(0);
    array(firstObservation.write(), originalObservation, 'copied complete observation'); array(firstSegmentation.write(), originalSegmentation, 'copied complete segmentation');
    firstObservation.free(); firstSegmentation.free();
    assert(copiedObservations.get(2) === null && copiedSegmentations.get(2) === null, 'absent samples');
    rejects(() => copiedObservations.replace(2, observation), 'observation collection index'); rejects(() => copiedSegmentations.replace(2, segmentation), 'segmentation collection index');
    const observationsClone = copiedObservations.cloned(), segmentationsClone = copiedSegmentations.cloned(); copiedObservations.clear(); copiedSegmentations.clear();
    dataset.set_observations(copiedObservations); dataset.set_segmentations(copiedSegmentations);
    rejects(() => model.initialize(dataset, options), 'empty dataset'); dataset.replace(observationsClone, copiedSegmentations); rejects(() => model.initialize(dataset, options), 'unpaired samples retained but rejected at use');
    dataset.replace(observationsClone, segmentationsClone);
    for (const owner of [observationsClone, segmentationsClone, copiedObservations, copiedSegmentations, copiedData]) owner.free();
    for (const floor of [NaN, Infinity, -.1]) { options.variance_floor_ratio = floor; rejects(() => model.initialize(dataset, options), 'invalid floor'); }
    options.variance_floor_ratio = .1;
    const invalid = segmentation.cloned(); invalid.set_boundaries(new Int32Array([3, -1, 12])); segmentations.replace(0, invalid); invalid.free(); dataset.set_segmentations(segmentations);
    rejects(() => model.initialize(dataset, options), 'invalid interval'); array(model.write(), originalModel, 'source model survives invalid initialization');
    segmentations.replace(0, segmentation); dataset.set_segmentations(segmentations);
    const retained = dataset.cloned();
    for (const owner of [dataset, observations, segmentations, states, observation, segmentation]) owner.free();
    const finalModel = model.initialize(retained, options); finalModel.free(); retained.free(); model.free(); options.free();
    return {originalCModels: cModels, wireBytes, optionFields: 3, datasetFields: 2, correctedCorpusFallback: true};
}
