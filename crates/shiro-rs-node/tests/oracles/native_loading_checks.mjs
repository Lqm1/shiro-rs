function assert(value, message) { if (!value) throw new Error(message); }
function array(actual, expected, message) {
    assert(actual.length === expected.length, message + ' length');
    actual.forEach((value, index) => assert(Object.is(value, expected[index]), message + '[' + index + ']'));
}
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
function snapshot(dataset) {
    const observations = dataset.observations(), segmentations = dataset.segmentations(), values = [[], []];
    for (let index = 0; index < observations.length; index++) { const value = observations.get(index); values[0].push(Array.from(value.write())); value.free(); }
    for (let index = 0; index < segmentations.length; index++) { const value = segmentations.get(index); values[1].push(Array.from(value.write())); value.free(); }
    observations.free(); segmentations.free(); return JSON.stringify(values);
}

export async function verifyLoading(api, load) {
    const raw = new Uint8Array(await load('init-input.bin')), ten = raw.subarray(0, 120), files = new api.FeatureFiles();
    files.set('input.f', raw); files.set('features/\u97f3\u58f0.f', ten); assert(files.length === 2, 'feature count'); array(files.names(), ['features/\u97f3\u58f0.f', 'input.f'], 'stable exact names');
    const detached = files.get('input.f'); detached.fill(0); array(files.get('input.f'), raw, 'copied file bytes');
    const inputCopy = raw.slice(); files.set('temporary', inputCopy); inputCopy.fill(0); array(files.get('temporary'), raw, 'copied file input');
    assert(files.remove('temporary') && !files.remove('temporary') && files.get('temporary') === null, 'file removal and absence');
    const source = new api.ModelDefinition(new TextDecoder().decode(await load('init-definition.json'))), model = source.build(); source.free();
    const initJson = new TextDecoder().decode(await load('init-segmentation.json')), initDocument = new api.SegmentationDocument(initJson);
    const initializedData = api.Dataset.load(initDocument, model, files, 12), options = new api.InitializationOptions(), initialized = model.initialize(initializedData, options);
    array(initialized.write(), new Uint8Array(await load('init-c-aligned.hsmm')), 'document to observations to exact original C initialization'); initialized.free(); initializedData.free(); options.free(); initDocument.free();

    const states = JSON.parse(new TextDecoder().decode(await load('align-c-isolated.json'))).file_list[0].states;
    states[0].jmp = [{d: 4, p: .1, retained: 'outside'}, {d: 2, p: .2, retained: 'inside'}]; states[0].extra = {nested: [true, null, 'retained']};
    const input = {file_list: [{filename: 'input.f', states, custom: 'first'}, {filename: 'features/\u97f3\u58f0.f', states, custom: 'second'}, {filename: 'input.f', states, custom: 'repeated'}], extra: {retained: true}};
    const document = new api.SegmentationDocument(JSON.stringify(input)), originalDocument = document.json(), loaded = api.Dataset.load(document, model, files, 12);
    const observations = loaded.observations(), segmentations = loaded.segmentations(); assert(observations.length === 3 && segmentations.length === 3, 'repeated files preserve document order');
    for (let index = 0; index < 3; index++) {
        const expectedObservation = api.Observation.from_model_rawfloat(index === 1 ? ten : raw, model, 12), stateOwner = new api.States(JSON.stringify(states)), expectedSegmentation = api.Segmentation.from_states(stateOwner, model);
        const observation = observations.get(index), segmentation = segmentations.get(index);
        assert(observation.frames === (index === 1 ? 10 : 12), 'ordered acoustic frame counts'); array(observation.write(), expectedObservation.write(), 'complete loaded observation'); array(segmentation.write(), expectedSegmentation.write(), 'complete loaded segmentation');
        for (const owner of [expectedObservation, stateOwner, expectedSegmentation, observation, segmentation]) owner.free();
    }
    observations.free(); segmentations.free();
    let groupedSamples = 0;
    for (const isolated of [false, true]) {
        const trainingFiles = api.Datasets.load_training_files(document, model, files, 12, isolated), clone = trainingFiles.cloned(); assert(trainingFiles.length === 3, 'training file count');
        assert(trainingFiles.get(3) === null, 'absent training dataset'); trainingFiles.clear(); trainingFiles.free();
        for (let index = 0; index < 3; index++) {
            const dataset = clone.get(index), expected = new api.Dataset(), observation = api.Observation.from_model_rawfloat(index === 1 ? ten : raw, model, 12), stateOwner = new api.States(JSON.stringify(states));
            const expectedObservations = new api.Observations(), expectedSegmentations = new api.Segmentations();
            if (isolated) {
                const groups = api.IsolatedGroups.split(model, observation, stateOwner); assert(groups.length === 2, 'two isolated samples per file');
                for (let groupIndex = 0; groupIndex < groups.length; groupIndex++) {
                    const group = groups.get(groupIndex), localObservation = group.observation(), localStates = group.states(), localSegmentation = api.Segmentation.from_states(localStates, model);
                    expectedObservations.push(localObservation); expectedSegmentations.push(localSegmentation);
                    for (const owner of [group, localObservation, localStates, localSegmentation]) owner.free(); groupedSamples++;
                }
                groups.free();
            } else {
                expectedObservations.push(observation); const segmentation = api.Segmentation.from_states(stateOwner, model); expectedSegmentations.push(segmentation); segmentation.free();
            }
            expected.replace(expectedObservations, expectedSegmentations); assert(snapshot(dataset) === snapshot(expected), 'complete training-file fields and local transition filtering');
            clone.replace(index, expected); rejects(() => clone.replace(3, expected), 'dataset collection index');
            for (const owner of [dataset, expected, observation, stateOwner, expectedObservations, expectedSegmentations]) owner.free();
        }
        const copied = clone.get(0), expectedCopy = snapshot(copied); clone.free(); assert(snapshot(copied) === expectedCopy, 'training dataset survives collection release'); copied.free();
    }
    const before = snapshot(loaded), retainedFiles = files.cloned(); files.clear(); assert(files.length === 0, 'clear feature files');
    for (const isolated of [false, true]) rejects(() => api.Datasets.load_training_files(document, model, files, 12, isolated), 'missing training feature file');
    rejects(() => api.Dataset.load(document, model, files, 12), 'missing feature file');
    files.set('INPUT.f', raw); rejects(() => api.Dataset.load(document, model, files, 12), 'no filename normalization');
    retainedFiles.set('features/\u97f3\u58f0.f', new Uint8Array([1, 2, 3])); rejects(() => api.Dataset.load(document, model, retainedFiles, 12), 'late malformed feature input');
    retainedFiles.set('features/\u97f3\u58f0.f', ten); rejects(() => api.Dataset.load(document, model, retainedFiles, 11), 'per-file frame budget');
    assert(document.json() === originalDocument && snapshot(loaded) === before, 'failed loading retains documents and existing datasets');
    const emptyDocument = new api.SegmentationDocument('{"file_list":[]}'), emptyData = api.Dataset.load(emptyDocument, model, files, 0), emptyTraining = api.Datasets.load_training_files(emptyDocument, model, files, 0, true);
    const emptyObservations = emptyData.observations(); assert(emptyObservations.length === 0 && emptyTraining.length === 0, 'empty document results'); emptyObservations.free();
    const rebuilt = api.Dataset.load(document, model, retainedFiles, 12); assert(snapshot(rebuilt) === before, 'replacement feature bytes restore complete output');
    for (const owner of [files, retainedFiles, document, model, emptyDocument, emptyData, emptyTraining, loaded]) owner.free(); assert(snapshot(rebuilt) === before, 'loaded dataset independent of all inputs'); rebuilt.free();
    return {orderedFiles: 3, groupedSamples, originalCInitialization: true, completeDatasetFields: 2};
}
