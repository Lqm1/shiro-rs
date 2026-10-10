function assert(value, message) { if (!value) throw new Error(message); }
function equal(a, b, message) { assert(a.length === b.length && a.every((v, i) => Object.is(v, b[i])), message); }
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
function json(value) { return JSON.parse(value.json()); }
const encode = text => new TextEncoder().encode(text), decode = bytes => new TextDecoder().decode(bytes);

// This checks connected transport and ownership, in addition to the independent
// original C/Lua numerical oracles in the individual family verifiers.
export async function verifyToolWorkflow(api, load) {
    const input = new Uint8Array(await load('utterances-input.wav'));
    const virtual = new Map([['/virtual/input.wav', input]]);
    const wave = api.Wave.read(virtual.get('/virtual/input.wav'), 100000), audioOptions = new api.AudioOptions();
    audioOptions.output_sample_rate = 16000; audioOptions.dither_level = .01;
    const sequence = api.DitherSequence.linux_gnu(), audio = api.Audio.prepare_with_sequence(wave, audioOptions, sequence);
    sequence.free(); audioOptions.free();
    const raw = api.rawfloat_write(audio.samples());
    virtual.set('/virtual/input.raw', raw);
    equal(raw, new Uint8Array(await load('utterances-c-audio.bin')), 'connected wav2raw original C bytes');
    const decoded = api.rawfloat_read(raw, 100000), featureOptions = new api.FeatureOptions();
    Object.assign(featureOptions, {order: 12, hop: 1600, sample_rate_hz: 16000, energy: 1});
    const extracted = api.Features.extract(decoded, featureOptions), features = api.rawfloat_write(extracted.values());
    assert(extracted.frames === 40 && extracted.columns === 13, 'connected xxcc shape');
    const reference = api.rawfloat_read(new Uint8Array(await load('utterances-c-features.bin')), 520);
    let maximumFeatureError = 0;
    extracted.values().forEach((value, i) => {
        const error = Math.abs(value - reference[i]) / Math.max(Math.abs(reference[i]), 1);
        maximumFeatureError = Math.max(maximumFeatureError, error);
        assert(error <= 2e-5, 'connected C feature tolerance');
    });
    featureOptions.free();

    // fextr is another consumer of this waveform, retaining its own preset.
    const batchOptions = new api.BatchOptions(), batch = api.BatchExtraction.extract(wave, '/virtual/batch', batchOptions, 0, () => .5);
    const batchAudio = batch.audio(), batchFeatures = batch.features(), batchOutputs = batch.outputs();
    const preset = api.batch_feature_options(0), recomputed = api.Features.extract(batchAudio.samples(), preset);
    equal(batchFeatures.values(), recomputed.values(), 'connected complete fextr preset values');
    assert(batchOutputs.raw === '/virtual/batch.raw' && batchOutputs.parameters === '/virtual/batch.param', 'connected fextr outputs');
    virtual.set(batchOutputs.raw, api.rawfloat_write(batchAudio.samples()));
    virtual.set(batchOutputs.parameters, api.rawfloat_write(batchFeatures.values()));
    for (const owner of [batchAudio, batchFeatures, batchOutputs, preset, recomputed, batch, batchOptions]) owner.free();

    const phoneOptions = new api.PhoneMapOptions();
    phoneOptions.states_per_phone = 1; phoneOptions.streams = 1;
    const map = api.PhoneMap.create('sil durfloor 0.3\nutt durfloor 0.3\n', phoneOptions);
    phoneOptions.free();
    const definition = map.to_definition(13, .1), uninitialized = definition.build();
    equal(uninitialized.write(), new Uint8Array(await load('utterances-c-uninit.hsmm')), 'connected mkpm/pm2md/mkhsmm C model');
    definition.free();
    const entries = api.IndexEntries.read(encode('first,utt sil utt\nsecond,utt sil utt\n'), '/virtual', '["sil"]', '["sil"]');
    const files = new api.FeatureFiles(), documents = [];
    for (let i = 0; i < entries.length; i++) {
        const entry = entries.get(i), filename = api.index_append_suffix(entry.stem, '.param');
        virtual.set(filename, features.slice()); files.set(filename, virtual.get(filename));
        const frames = api.feature_frame_count(BigInt(virtual.get(filename).length), 13);
        const states = map.initial(JSON.parse(entry.phonemes_json()), frames);
        documents.push({filename, states: json(states), source: i});
        states.free(); entry.free();
    }
    entries.free();
    const document = new api.SegmentationDocument(JSON.stringify({file_list: documents, workflow: 'connected'}));
    const dataset = api.Dataset.load(document, uninitialized, files, 40), init = new api.InitializationOptions();
    Object.assign(init, {flat_start: true, globally_tied: true, variance_floor_ratio: 1});
    const initialized = uninitialized.initialize(dataset, init); dataset.free(); init.free(); uninitialized.free();
    const trainingFiles = api.Datasets.load_training_files(document, initialized, files, 40, false);
    let reportsChecked = 0, alignedStates = 0;
    for (const mode of [0, 1]) {
        const options = new api.TrainingOptions();
        Object.assign(options, {iterations: 2, duration_mode: mode, termination_threshold: 0, deterministic_annealing: true, workers: 2, state_radius: 0, duration_extra: 100});
        const callbacks = [], result = initialized.train_with_progress(trainingFiles, options, report => callbacks.push(report));
        const reports = result.iterations(), rows = decode(result.write_likelihood_csv()).trim().split('\n');
        assert(reports.length === 2 && callbacks.length === 2 && rows.length === 4, 'connected rest complete iterations/file rows');
        for (let i = 0; i < reports.length; i++) {
            const report = reports.get(i), likelihoods = report.file_likelihoods();
            assert(report.iteration === i && Number.isFinite(report.mean_log_likelihood) && likelihoods.length === 2, 'connected rest ordered report');
            for (let file = 0; file < 2; file++) {
                const csv = rows[i * 2 + file].split(',').map(Number); assert(csv.length === 1 && csv.every(Number.isFinite), 'connected multi-file CSV');
                const row = likelihoods.get(file), values = row.values();
                assert(values.length === 1 && Math.abs(values[0] - csv[0]) <= 1e-5, 'connected CSV rounding/order'); row.free();
            }
            assert(callbacks[i].iteration === i && callbacks[i].mean_log_likelihood === report.mean_log_likelihood, 'connected callback snapshot');
            report.free(); callbacks[i].free(); likelihoods.free(); reportsChecked++;
        }
        reports.free();
        const trained = result.model(), wire = trained.write(), prefix = mode ? '/virtual/hmm' : '/virtual/hsmm';
        virtual.set(prefix + '.hsmm', wire.slice());
        virtual.set(prefix + '.likelihood.csv', result.write_likelihood_csv());
        const reloaded = api.Model.read(virtual.get(prefix + '.hsmm'));
        equal(reloaded.write(), wire, 'connected complete saved model bytes');
        const alignment = new api.AlignmentOptions();
        Object.assign(alignment, {duration_mode: mode, state_radius: 0, duration_extra: 100});
        const before = trained.align_document(document, files, 40, alignment), after = reloaded.align_document(document, files, 40, alignment);
        assert(before.json() === after.json(), 'connected inference before/after model reload');
        const aligned = json(after); assert(aligned.workflow === 'connected' && aligned.file_list.length === 2, 'connected inference metadata');
        assert(aligned.file_list.every((file, i) => file.source === i), 'connected ordered file metadata');
        virtual.set(prefix + '.aligned.json', encode(after.json()));
        const outputs = new Map();
        for (const file of aligned.file_list) {
            const states = new api.States(JSON.stringify(file.states)), labels = api.Labels.from_states(states, .1, false);
            const path = api.label_output_path(file.filename, '.lab'), bytes = labels.write(); outputs.set(path, bytes);
            const reread = api.Labels.parse(decode(bytes)), restored = reread.to_states(map, .1);
            assert(reread.length === 5 && json(restored).at(-1).time === file.states.at(-1).time, 'connected seg2lab/lab2seg boundary roundtrip');
            alignedStates += file.states.length;
            for (const owner of [states, labels, reread, restored]) owner.free();
        }
        assert(outputs.size === 2 && [...outputs.keys()].some(value => value.replaceAll('\\', '/') === '/virtual/first.lab') && [...outputs.keys()].some(value => value.replaceAll('\\', '/') === '/virtual/second.lab'), 'connected two-file outputs');
        for (const [path, bytes] of outputs) virtual.set(prefix + path, bytes);
        const untied = reloaded.untie(after), untiedModel = untied.model(), untiedDocument = untied.segmentation(), assignments = untied.assignments();
        assert(assignments.length === 10 && json(untiedDocument).file_list.length === 2 && untied.write_summary().length > 0, 'connected untie complete outputs');
        virtual.set(prefix + '.untied.hsmm', untiedModel.write());
        virtual.set(prefix + '.untied.json', encode(untiedDocument.json()));
        virtual.set(prefix + '.summary.txt', untied.write_summary());
        const untiedReloaded = api.Model.read(virtual.get(prefix + '.untied.hsmm')); equal(untiedReloaded.write(), untiedModel.write(), 'connected untied model reload');
        for (const owner of [untiedReloaded, untiedModel, untiedDocument, assignments, untied, before, after, alignment, trained, reloaded, result, options]) owner.free();
    }
    // The utterance tool consumes the same decoded waveform and complete feature
    // matrix. Its separate end-to-end and decomposed entry points must agree.
    const utteranceOptions = new api.UtteranceOptions(); Object.assign(utteranceOptions, {utterances: 2, iterations: 2});
    const source = api.UtteranceModelSource.fresh(), random = api.DitherSequence.linux_gnu();
    const split = api.SegmentedWave.split_with_sequence(wave, '/virtual/first.param', 13, 0, utteranceOptions, source, random);
    const splitFeatures = split.features(); equal(splitFeatures.values(), extracted.values(), 'connected wavsplit/xxcc exact features');
    const direct = api.SegmentedUtterances.split_features(extracted, '/virtual/first.param', utteranceOptions, source), nested = split.utterances();
    const directModel = direct.model(), nestedModel = nested.model(), directLabels = direct.labels(), nestedLabels = nested.labels();
    equal(directModel.write(), nestedModel.write(), 'connected wavsplit complete model'); equal(directLabels.write(), nestedLabels.write(), 'connected wavsplit complete labels');
    // Failure after a valid run preserves the virtual inputs and model owners.
    const initialWire = initialized.write(); files.set(JSON.parse(document.json()).file_list[1].filename, features.subarray(0, features.length - 1));
    rejects(() => api.Dataset.load(document, initialized, files, 40), 'connected partial-frame failure');
    equal(initialized.write(), initialWire, 'connected failed import retains model');
    for (const owner of [directModel, nestedModel, directLabels, nestedLabels, splitFeatures, direct, nested, split, random, source, utteranceOptions, trainingFiles, initialized, document, files, map, extracted, audio, wave]) owner.free();
    assert(virtual.size === 22, 'complete distinct virtual input/output artifacts');
    return {files: 2, virtualArtifacts: virtual.size, featureValues: reference.length, maximumFeatureError, durationModes: 2, reportsChecked, alignedStates, toolComputations: 14};
}
