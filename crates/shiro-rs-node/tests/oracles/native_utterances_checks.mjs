function assert(value, message) { if (!value) throw new Error(message); }
function equal(actual, expected, message) { assert(actual.length === expected.length && actual.every((value, index) => Object.is(value, expected[index])), message); }
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
function floats(bytes) { const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength); return Float32Array.from({length: bytes.length / 4}, (_, index) => view.getFloat32(index * 4, true)); }
function canonical(value) { return JSON.stringify(order(value)); }
function order(value) { return Array.isArray(value) ? value.map(order) : value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(key => [key, order(value[key])])) : value; }
function artifact(result, field, render) { const child = result[field](); const value = child ? render(child) : null; if (child) child.free(); return value; }
function reports(rows) { const values = []; for (let index = 0; index < rows.length; index++) { const report = rows.get(index), likelihoods = report.file_likelihoods(), files = []; for (let file = 0; file < likelihoods.length; file++) { const row = likelihoods.get(file); files.push(Array.from(row.values())); row.free(); } likelihoods.free(); values.push([report.iteration, report.temperature, report.mean_log_likelihood, files]); report.free(); } return values; }
function snapshot(result) {
    const value = {phones: JSON.parse(result.phones_json())};
    for (const field of ['phonemap', 'definition', 'initial_segmentation', 'alignment']) value[field] = artifact(result, field, child => JSON.parse(child.json()));
    for (const field of ['uninitialized_model', 'initialized_model', 'model']) value[field] = artifact(result, field, child => Array.from(child.write()));
    value.iterations = artifact(result, 'iterations', reports); value.labels = artifact(result, 'labels', child => new TextDecoder().decode(child.write())); return value;
}
export async function verifyUtterances(api, load) {
    const decode = bytes => new TextDecoder().decode(bytes), options = new api.UtteranceOptions(), defaults = {utterances: 1, hop_seconds: .1, minimum_silence_seconds: .3, minimum_voicing_seconds: .3, iterations: 15};
    for (const [key, value] of Object.entries(defaults)) assert(options[key] === value, 'native utterance default ' + key);
    Object.assign(options, {utterances: 2, iterations: 2}); const copied = options.cloned(); for (const key of Object.keys(defaults)) assert(copied[key] === options[key], 'all utterance settings cloned'); copied.free();
    const input = floats(new Uint8Array(await load('utterances-c-features.bin'))), features = new api.Features(40, 13, input), fresh = api.UtteranceModelSource.fresh();
    assert(fresh.kind === 0 && fresh.model() === null, 'fresh source has no model');
    const result = api.SegmentedUtterances.split_features(features, 'sample.param', options, fresh), reference = snapshot(result);
    let modelBytes = 0;
    for (const [field, name] of [['uninitialized_model', 'uninit'], ['initialized_model', 'flat'], ['model', 'trained']]) { const expected = new Uint8Array(await load('utterances-c-' + name + '.hsmm')); equal(reference[field], expected, 'complete C intermediate model ' + name); modelBytes += expected.length; }
    for (const [field, name] of [['phonemap', 'phonemap'], ['initial_segmentation', 'initial'], ['alignment', 'aligned']]) assert(canonical(reference[field]) === canonical(JSON.parse(decode(await load('utterances-c-' + name + '.json')))), 'complete C document ' + name);
    const definition = new api.ModelDefinition(decode(await load('utterances-c-definition.json'))), expectedDefinition = definition.build(), actualDefinition = result.definition(), actualBuilt = actualDefinition.build(); equal(actualBuilt.write(), expectedDefinition.write(), 'C model definition behavior'); actualBuilt.free(); expectedDefinition.free(); actualDefinition.free(); definition.free();
    equal(reference.phones, ['sil', 'utt', 'sil', 'utt', 'sil'], 'complete phones'); assert(reference.iterations.length === 2, 'two training reports');
    const labels = result.labels(), expectedLabels = decode(await load('utterances-c-labels.txt')).trim().split(/\r?\n/); assert(labels.length === expectedLabels.length, 'C label count');
    for (let index = 0; index < labels.length; index++) { const label = labels.get(index), fields = expectedLabels[index].split('\t'); assert(Math.abs(label.start - Number(fields[0])) < 1e-14 && Math.abs(label.end - Number(fields[1])) < 1e-14 && label.name === fields[2], 'complete C label'); label.free(); } labels.free();
    for (const [factory, name, kind] of [['initialized', 'flat', 1], ['trained', 'trained', 2]]) {
        const model = api.Model.read(new Uint8Array(await load('utterances-c-' + name + '.hsmm'))), source = api.UtteranceModelSource[factory](model), clonedSource = source.cloned(), original = model.write(); model.free(); source.free();
        assert(clonedSource.kind === kind, 'source variant preserved'); const child = clonedSource.model(); equal(child.write(), original, 'source owns complete model'); child.free();
        const loaded = api.SegmentedUtterances.split_features(features, 'sample.param', options, clonedSource), value = snapshot(loaded); equal(value.model, reference.model, 'existing source model'); assert(value.uninitialized_model === null && value.initialized_model === null && value.iterations.length === (kind === 1 ? 2 : 0), 'source stage presence'); assert(value.labels === reference.labels, 'existing source labels'); loaded.free(); clonedSource.free();
    }
    const wave = api.Wave.read(new Uint8Array(await load('utterances-input.wav')), 100000), sequence = api.DitherSequence.linux_gnu(), split = api.SegmentedWave.split_with_sequence(wave, 'sample.param', 13, 0, options, fresh, sequence);
    const audio = split.audio(), expectedAudio = floats(new Uint8Array(await load('utterances-c-audio.bin'))); equal(audio.samples(), expectedAudio, 'complete C utterance audio'); assert(audio.sample_rate === 16000, 'utterance audio rate'); audio.free();
    const extracted = split.features(); assert(extracted.frames === 40 && extracted.columns === 13, 'utterance feature shape'); let maximumNormalizedError = 0;
    extracted.values().forEach((value, index) => { const error = Math.abs(value - input[index]) / Math.max(Math.abs(input[index]), 1); maximumNormalizedError = Math.max(maximumNormalizedError, error); assert(error <= 2e-5, 'utterance C feature tolerance'); });
    const splitResult = split.utterances(), splitLabels = splitResult.labels(); assert(decode(splitLabels.write()) === reference.labels, 'wave-to-trained utterance labels'); splitLabels.free();
    const trainedModel = splitResult.model(), trainedSource = api.UtteranceModelSource.trained(trainedModel), repeated = api.SegmentedUtterances.split_features(extracted, 'sample.param', options, trainedSource); assert(snapshot(repeated).labels === reference.labels, 'wave model inference after reuse'); repeated.free(); trainedModel.free(); trainedSource.free(); extracted.free(); splitResult.free();
    const callbackSequence = api.DitherSequence.linux_gnu(); let draws = 0; const callbackSplit = api.SegmentedWave.split(wave, 'sample.param', 13, 0, options, fresh, () => {draws++; return callbackSequence.next_uniform();}); assert(draws === wave.samples().length, 'wave random consumption'); const callbackResult = callbackSplit.utterances(); assert(snapshot(callbackResult).labels === reference.labels, 'live callback waveform result'); callbackResult.free(); callbackSplit.free(); callbackSequence.free();
    const thrown = {utterances: 'identity'}; let caught; draws = 0; try { api.SegmentedWave.split(wave, 'sample.param', 13, 0, options, fresh, () => {draws++; throw thrown;}); } catch (value) {caught = value;} assert(caught === thrown && draws === 1, 'wave callback failure stops with identity');
    const zeroOptions = options.cloned(); Object.assign(zeroOptions, {utterances: 1, hop_seconds: .125, minimum_silence_seconds: .25, minimum_voicing_seconds: .375, iterations: 0});
    const zero = api.SegmentedUtterances.split_features(features, 'unicode-\u97f3.param', zeroOptions, fresh), zeroSnapshot = snapshot(zero);
    assert(zeroSnapshot.iterations.length === 0 && zeroSnapshot.phones.length === 3 && zeroSnapshot.labels.length > 0 && zeroSnapshot.alignment.file_list[0].filename === 'unicode-\u97f3.param', 'all nondefault options and UTF-8 filename'); equal(zeroSnapshot.model, zeroSnapshot.initialized_model, 'zero iterations retain initialized model'); zero.free();
    let featureKinds = 0;
    for (const kind of [0, 1, 2]) {
        const random = api.DitherSequence.linux_gnu(), alternate = api.SegmentedWave.split_with_sequence(wave, 'other.param', 13, kind, zeroOptions, fresh, random), feature = alternate.features(), utterance = alternate.utterances(), reports = utterance.iterations();
        assert(feature.frames === 32 && feature.columns > 0 && reports.length === 0, 'all waveform feature kinds and fractional hop');
        const sourceModel = utterance.model(), source = api.UtteranceModelSource.trained(sourceModel), aligned = api.SegmentedUtterances.split_features(feature, 'other.param', zeroOptions, source);
        assert(snapshot(aligned).labels === snapshot(utterance).labels, 'all feature kinds produce reusable trained models');
        for (const owner of [random, alternate, feature, utterance, reports, sourceModel, source, aligned]) owner.free(); featureKinds++;
    }
    zeroOptions.free();
    for (const [dimensions, kind] of [[1, 0], [13, 3]]) { draws = 0; rejects(() => api.SegmentedWave.split(wave, 'sample.param', dimensions, kind, options, fresh, () => {draws++; return .5;}), 'feature setup rejection'); assert(draws === 0, 'invalid feature setup consumes no draws'); }
    for (const [key, value] of [['utterances', 0xffffffff], ['hop_seconds', 0], ['hop_seconds', 1e-30], ['minimum_silence_seconds', NaN], ['minimum_voicing_seconds', Infinity], ['iterations', 0xffffffff]]) { const saved = options[key]; options[key] = value; rejects(() => api.SegmentedUtterances.split_features(features, 'sample.param', options, fresh), 'invalid option ' + key); options[key] = saved; }
    const invalid = new api.Features(40, 13, input.slice(1)); rejects(() => api.SegmentedUtterances.split_features(invalid, 'sample.param', options, fresh), 'invalid feature shape'); invalid.set_values(new Float32Array(input.length).fill(NaN)); rejects(() => api.SegmentedUtterances.split_features(invalid, 'sample.param', options, fresh), 'nonfinite features'); invalid.free();
    const saved = result.cloned();
    for (const field of ['phonemap', 'definition', 'initial_segmentation', 'model', 'iterations', 'alignment', 'labels']) { const child = result[field](); saved['set_' + field](child); child.free(); }
    for (const field of ['uninitialized_model', 'initialized_model']) { const child = result[field](); saved['clear_' + field](); assert(saved[field]() === null, 'optional artifact cleared'); saved['set_' + field](child); equal(child.write(), reference[field], 'borrowed optional setter keeps source'); child.free(); }
    saved.set_phones_json(result.phones_json()); rejects(() => saved.set_phones_json('[1]'), 'phone replacement atomic'); assert(canonical(snapshot(saved)) === canonical(reference), 'all ten utterance fields edited independently');
    const parentClone = split.cloned(); for (const field of ['audio', 'features', 'utterances']) { const child = split[field](); parentClone['set_' + field](child); child.free(); }
    split.free(); const surviving = parentClone.utterances(); assert(snapshot(surviving).labels === reference.labels, 'all wave fields survive parent release'); surviving.free(); parentClone.free();
    const model = result.model(), map = result.phonemap(), def = result.definition(), initial = result.initial_segmentation(), aligned = result.alignment(), arbitrary = new api.SegmentedUtterances(model, map, def, initial, aligned);
    arbitrary.set_phones_json('["arbitrary\\u0000name"]'); const emptyReports = new api.IterationReports(), emptyLabels = new api.Labels(); arbitrary.set_iterations(emptyReports); arbitrary.set_labels(emptyLabels); assert(JSON.parse(arbitrary.phones_json())[0] === 'arbitrary\u0000name' && artifact(arbitrary, 'iterations', value => value.length) === 0 && artifact(arbitrary, 'labels', value => value.length) === 0, 'arbitrary result construction and field replacement');
    const rawAudio = new api.Audio(0, new Float32Array([NaN])), rawFeatures = new api.Features(0, 99, new Float32Array()), rawWave = new api.SegmentedWave(rawAudio, rawFeatures, arbitrary); rawAudio.free(); rawFeatures.free(); arbitrary.free(); const rawChild = rawWave.audio(); assert(rawChild.sample_rate === 0 && Number.isNaN(rawChild.samples()[0]), 'arbitrary wave result fields'); rawChild.free(); rawWave.free();
    result.free(); assert(canonical(snapshot(saved)) === canonical(reference), 'complete results survive source release');
    for (const owner of [model, map, def, initial, aligned, emptyReports, emptyLabels, saved, features, options, fresh, wave, sequence]) owner.free();
    return {originalCModelBytes: modelBytes, featureValues: input.length, audioSamples: expectedAudio.length, maximumNormalizedError, labels: expectedLabels.length, sourceVariants: 3, featureKinds, optionFields: 5, resultFields: 10, waveResultFields: 3};
}
