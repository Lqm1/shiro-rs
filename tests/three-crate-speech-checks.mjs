function check(value, message) { if (!value) throw new Error(message); }
function same(a, b, message) {
    check(a.length === b.length && a.every((value, index) => Object.is(value, b[index])), message);
}
function canonical(value) {
    return JSON.stringify(value, function (_key, item) {
        return item && !Array.isArray(item) && typeof item === 'object'
            ? Object.fromEntries(Object.keys(item).sort().map(key => [key, item[key]])) : item;
    });
}
const text = bytes => new TextDecoder().decode(bytes);

// Transport between independent Wasm memories uses bytes and typed arrays,
// never an owner pointer from another generated module.
function compareAlignment(S, L, model, observation, states, options) {
    const owned = [], retain = value => { owned.push(value); return value; };
    let aligned;
    try {
        const wire = model.write(), foreignModel = retain(L.ModelF32.read(wire));
        same(foreignModel.write(), wire, 'complete model across independent modules');
        const source = retain(S.Segmentation.from_states(states, model));
        const segmentation = retain(L.SegmentationF32.read(source.write()));
        const foreignObservation = retain(L.ObservationF32.read(observation.write()));
        same(foreignObservation.write(), observation.write(), 'complete observation transport');
        const preparation = retain(new L.PreparationOptionsF32());
        preparation.temperature = options.hsmm_temperature;
        preparation.duration_weight = options.duration_weight;
        const durations = options.duration_mode === 0
            ? Uint32Array.from(segmentation.duration_states().filter(value => value >= 0)) : new Uint32Array();
        const prepared = retain(foreignModel.prepare_selected_durations(preparation, durations));
        const explicit = options.duration_mode === 0;
        const pruning = retain(new L.EmissionPruningF32(explicit ? 1 : 2,
            explicit ? options.state_radius : options.pruning_slope));
        const emissions = retain(prepared.emission_log_probabilities(foreignObservation, segmentation,
            explicit ? options.hsmm_temperature : options.geometric_temperature, pruning));
        let result;
        if (explicit) {
            const settings = retain(new L.HsmmOptionsF32());
            Object.assign(settings, {temperature: options.hsmm_temperature, duration_weight: options.duration_weight,
                state_radius: options.state_radius, duration_extra: options.duration_extra,
                duration_extra_factor: options.duration_extra_factor});
            const path = retain(prepared.viterbi_hsmm(segmentation, emissions, settings));
            result = retain(segmentation.resegment(path.occurrences()));
        } else {
            const settings = retain(new L.GeometricOptionsF32());
            settings.temperature = options.geometric_temperature;
            settings.pruning_slope = options.pruning_slope;
            const path = retain(foreignModel.viterbi_geometric(segmentation, emissions, settings));
            result = retain(segmentation.cloned());
            result.set_boundaries(path.boundaries());
        }
        aligned = model.align_states(observation, states, options);
        const reference = retain(S.Segmentation.from_states(aligned, model));
        same(result.write(), reference.write(), 'standalone library and SHIRO complete inference');
        return aligned;
    } catch (error) {
        if (aligned) aligned.free();
        throw error;
    } finally {
        owned.reverse().forEach(value => value.free());
    }
}

export async function verifyThreeCrateSpeech(C, L, S, load, progress = () => {}) {
    const historicalBytes = new Uint8Array(await load('cmu-arctic-all-speakers.hsmm'));
    const historical = S.Model.read(historicalBytes);
    same(historical.write_with_encoding(1), historicalBytes, 'historical C model exact legacy encoding');
    const mapOptions = new S.PhoneMapOptions();
    mapOptions.states_per_phone = 5; mapOptions.streams = 3;
    const map = S.PhoneMap.create(text(await load('cmu-arpabet-phoneset.csv')), mapOptions);
    mapOptions.free();
    const rows = text(await load('cmu-slt-index.csv')).trim().split(/\r?\n/);
    const references = [];
    for (const mode of ['hmm', 'hsmm']) references.push(JSON.parse(text(await load(`cmu-slt-c-${mode}.json`))).file_list);
    const featuresOptions = S.batch_feature_options(0), files = new S.FeatureFiles(), records = [], samples = [];
    let featureValues = 0, maximumFeatureError = 0, historicalStates = 0;
    for (let index = 0; index < rows.length; index++) {
        const comma = rows[index].indexOf(','), stem = rows[index].slice(0, comma);
        progress(`speech ${index + 1}/${rows.length}: extraction`);
        const original = C.wave_read_f32(new Uint8Array(await load(`cmu-slt-${stem}.wav`)), 100000);
        const wave = new S.Wave(original.sample_rate, original.bits_per_sample, original.channels, original.encoding, original.samples);
        same(wave.samples(), original.samples, 'independent waveform memory snapshot');
        original.free();
        const audioOptions = new S.AudioOptions(), sequence = S.DitherSequence.linux_gnu();
        const audio = S.Audio.prepare_with_sequence(wave, audioOptions, sequence);
        wave.free(); audioOptions.free(); sequence.free();
        const feature = S.Features.extract(audio.samples(), featuresOptions); audio.free();
        check(feature.columns === 36, 'three twelve-dimensional streams');
        const values = feature.values(), expected = S.rawfloat_read(new Uint8Array(await load(`cmu-slt-${stem}.param`)), 1000000);
        check(values.length === expected.length, 'complete original C feature dimensions');
        values.forEach((value, offset) => {
            const difference = Math.abs(value - expected[offset]) / Math.max(1, Math.abs(expected[offset]));
            maximumFeatureError = Math.max(maximumFeatureError, difference);
            check(difference <= 5e-5, 'existing real-speech C feature tolerance');
        });
        featureValues += values.length;
        const raw = S.rawfloat_write(values), observation = S.Observation.from_model_rawfloat(raw, historical, 100000);
        const phones = ['sil', ...rows[index].slice(comma + 1).split(/\s+/), 'sil'];
        const initial = map.initial(phones, feature.frames); feature.free();
        const settings = new S.AlignmentOptions(); settings.duration_mode = 1;
        progress(`speech ${index + 1}/${rows.length}: historical HMM`);
        const hmm = compareAlignment(S, L, historical, observation, initial, settings);
        check(canonical(JSON.parse(hmm.json())) === canonical(references[0][index].states), 'complete original C HMM states');
        settings.duration_mode = 0; settings.state_radius = 10; settings.duration_extra = 50;
        progress(`speech ${index + 1}/${rows.length}: historical HSMM`);
        const hsmm = compareAlignment(S, L, historical, observation, hmm, settings);
        check(canonical(JSON.parse(hsmm.json())) === canonical(references[1][index].states), 'complete original C HSMM states');
        historicalStates += JSON.parse(hsmm.json()).length;
        const labels = S.Labels.from_states(hsmm, .005, false);
        check(labels.length === phones.length, 'all spoken labels retained');
        const first = labels.get(0), last = labels.get(labels.length - 1);
        check(first.start === 0 && last.end === observation.frames * .005, 'complete label time span');
        first.free(); last.free(); labels.free(); settings.free(); hmm.free(); hsmm.free();
        const filename = `/virtual/${stem}.param`; files.set(filename, raw);
        records.push({filename, states: JSON.parse(initial.json())});
        samples.push({observation, initial});
    }
    featuresOptions.free(); historical.free();
    progress('real-speech corpus: initialization');
    const definition = map.to_definition(12, .005), uninitialized = definition.build(); definition.free(); map.free();
    const document = new S.SegmentationDocument(JSON.stringify({file_list: records}));
    const dataset = S.Dataset.load(document, uninitialized, files, 100000), initialization = new S.InitializationOptions();
    Object.assign(initialization, {flat_start: true, globally_tied: true, variance_floor_ratio: 1});
    const initialized = uninitialized.initialize(dataset, initialization);
    dataset.free(); initialization.free(); uninitialized.free();
    const datasets = S.Datasets.load_training_files(document, initialized, files, 100000, false);
    const reports = [];
    for (const mode of [0, 1]) {
        progress(`real-speech corpus: training mode ${mode}`);
        const options = new S.TrainingOptions();
        Object.assign(options, {iterations: 2, duration_mode: mode, deterministic_annealing: true,
            termination_threshold: 0, pruning_slope: .8});
        const result = initialized.train(datasets, options), iterations = result.iterations(); options.free();
        check(iterations.length === 2, 'both full learning iterations');
        for (let index = 0; index < iterations.length; index++) {
            const row = iterations.get(index), likelihoods = row.file_likelihoods();
            check(row.iteration === index && Number.isFinite(row.mean_log_likelihood) && likelihoods.length === rows.length,
                'finite complete corpus reports');
            for (let file = 0; file < likelihoods.length; file++) {
                const item = likelihoods.get(file), values = item.values();
                check(values.length === 1 && values.every(Number.isFinite), 'complete file likelihood row'); item.free();
            }
            reports.push(row.mean_log_likelihood); likelihoods.free(); row.free();
        }
        iterations.free();
        check(text(result.write_likelihood_csv()).trim().split('\n').length === rows.length * 2, 'all CSV file rows');
        const learned = result.model(), bytes = learned.write(), reloaded = S.Model.read(bytes), foreign = L.ModelF32.read(bytes);
        same(reloaded.write(), bytes, 'saved complete learned model');
        same(foreign.write(), bytes, 'standalone library learned model interoperation'); foreign.free();
        const settings = new S.AlignmentOptions(); settings.state_radius = 10; settings.duration_extra = 50;
        for (let index = 0; index < samples.length; index++) {
            progress(`real-speech corpus: reloaded mode ${mode}, file ${index + 1}`);
            const {observation, initial} = samples[index];
            const before = compareAlignment(S, L, learned, observation, initial, settings);
            const after = compareAlignment(S, L, reloaded, observation, initial, settings);
            check(canonical(JSON.parse(before.json())) === canonical(JSON.parse(after.json())), 'learned inference before/after reload');
            before.free(); after.free();
        }
        settings.free(); learned.free(); reloaded.free(); result.free();
    }
    initialized.free(); datasets.free(); document.free(); files.free();
    samples.forEach(({observation, initial}) => { observation.free(); initial.free(); });
    return {files: rows.length, featureValues, maximumFeatureError, historicalStates,
        trainingModes: 2, iterations: reports.length, meanLogLikelihoods: reports,
        independentModules: 3, completeModelAndObservationTransport: true};
}
