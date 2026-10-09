function assert(value, message) { if (!value) throw new Error(message); }
function array(actual, expected, message) {
    assert(actual.length === expected.length, message + ' length');
    actual.forEach((value, index) => assert(Object.is(value, expected[index]), message + '[' + index + ']'));
}
function rejects(run, message) { let failed = false; try { run(); } catch { failed = true; } assert(failed, message); }
function reportSnapshot(report) {
    const rows = report.file_likelihoods(), values = [];
    for (let index = 0; index < rows.length; index++) { const row = rows.get(index); values.push(Array.from(row.values())); row.free(); }
    rows.free(); return {iteration: report.iteration, temperature: report.temperature, mean: report.mean_log_likelihood, rows: values};
}
function resultSnapshot(result) {
    const model = result.model(), reports = result.iterations(), values = [];
    for (let index = 0; index < reports.length; index++) { const report = reports.get(index); values.push(reportSnapshot(report)); report.free(); }
    const value = JSON.stringify([Array.from(model.write()), values]); model.free(); reports.free(); return value;
}

export async function verifyTraining(api, load) {
    const model = api.Model.read(new Uint8Array(await load('init-c-aligned.hsmm'))), sourceWire = model.write();
    const document = new api.SegmentationDocument(new TextDecoder().decode(await load('align-c-isolated.json'))), features = new api.FeatureFiles(); features.set('input.f', new Uint8Array(await load('init-input.bin')));
    const embedded = api.Datasets.load_training_files(document, model, features, 12, false), isolated = api.Datasets.load_training_files(document, model, features, 12, true);
    const options = new api.TrainingOptions(), defaults = {iterations: 1, duration_mode: 0, hsmm_temperature: 1, duration_weight: 1, state_radius: 5, duration_extra: 30, duration_extra_factor: 1, geometric_temperature: 1, pruning_slope: Math.fround(.3), termination_threshold: 1, deterministic_annealing: false, mean_frame_likelihood: false, workers: 1};
    for (const [key, value] of Object.entries(defaults)) assert(Object.is(options[key], value), 'native default ' + key);
    const cases = [
        ['rest-c-hsmm-one', 1, 0, false, false, false], ['rest-c-hsmm-two', 2, 0, false, false, false],
        ['rest-c-daem', 3, 0, true, false, false], ['rest-c-hmm', 2, 1, false, false, false],
        ['rest-c-hsmm-one', 1, 0, false, true, false, 'rest-c-mean'], ['rest-c-isolated', 2, 0, false, false, true],
        ['rest-c-isolated-hmm', 2, 1, false, false, true], ['rest-c-isolated-daem', 3, 0, true, false, true],
        ['rest-c-isolated-mean', 2, 0, false, true, true],
    ];
    let wireBytes = 0, reportCount = 0, likelihoodCount = 0, maximumLikelihoodError = 0;
    for (const [name, iterations, mode, anneal, mean, groups, likelihoodName] of cases) {
        Object.assign(options, defaults, {iterations, duration_mode: mode, deterministic_annealing: anneal, mean_frame_likelihood: mean, termination_threshold: 0, pruning_slope: .8});
        const copiedOptions = options.cloned(); for (const key of Object.keys(defaults)) assert(Object.is(copiedOptions[key], options[key]), 'all option fields cloned');
        const callbacks = [], result = model.train_with_progress(groups ? isolated : embedded, copiedOptions, report => { callbacks.push(report); }); copiedOptions.free();
        const expectedWire = new Uint8Array(await load(name + '.hsmm')), trained = result.model(); array(trained.write(), expectedWire, 'original C complete trained model ' + name); wireBytes += expectedWire.length; trained.free();
        const expectedRows = new TextDecoder().decode(await load((likelihoodName || name) + '.likelihood')).trim().split(/\r?\n/).map(row => row.split(',').map(Number));
        const reports = result.iterations(); assert(reports.length === iterations && callbacks.length === iterations, 'complete progress/result iteration counts');
        for (let index = 0; index < iterations; index++) {
            const report = reports.get(index), actual = reportSnapshot(report); assert(JSON.stringify(actual) === JSON.stringify(reportSnapshot(callbacks[index])), 'independent callback/result report fields');
            assert(actual.iteration === index && actual.rows.length === 1 && actual.rows[0].length === expectedRows[index].length, 'report shape and index');
            const temperature = anneal ? Math.fround(Math.sqrt(Math.fround((index + 1) / iterations))) : 1; assert(actual.temperature === temperature, 'native annealing temperature');
            let rowMean = 0;
            for (let group = 0; group < actual.rows[0].length; group++) {
                const error = Math.abs(actual.rows[0][group] - Math.fround(expectedRows[index][group])); maximumLikelihoodError = Math.max(maximumLikelihoodError, error); assert(error <= 1e-5, 'original C likelihood tolerance');
                rowMean = Math.fround(rowMean + Math.fround(actual.rows[0][group] / actual.rows[0].length)); likelihoodCount++;
            }
            assert(actual.mean === Math.fround(rowMean / temperature), 'complete report aggregate');
            report.free(); callbacks[index].free(); reportCount++;
        }
        reports.free(); const saved = result.cloned(), savedSnapshot = resultSnapshot(saved); result.free(); assert(resultSnapshot(saved) === savedSnapshot, 'training result survives source release'); saved.free();
        array(model.write(), sourceWire, 'source model unchanged by training');
    }
    Object.assign(options, defaults, {iterations: 5}); const stopped = model.train(embedded, options), stopReports = stopped.iterations(); assert(stopReports.length === 2, 'native convergence stopping'); stopReports.free(); stopped.free();
    options.iterations = 0; const noFiles = new api.Datasets(), zero = model.train(noFiles, options), zeroModel = zero.model(), zeroReports = zero.iterations(); array(zeroModel.write(), sourceWire, 'zero iteration model'); assert(zeroReports.length === 0, 'zero iteration reports'); zeroModel.free(); zeroReports.free(); zero.free();
    options.iterations = 1; rejects(() => model.train(noFiles, options), 'empty training files');
    for (const [key, value] of [['duration_mode', 2], ['workers', 0], ['termination_threshold', NaN], ['iterations', 0x80000000]]) {
        Object.assign(options, defaults); options[key] = value; rejects(() => model.train(embedded, options), 'invalid option ' + key);
    }
    Object.assign(options, defaults, {iterations: 3, termination_threshold: 0}); const sentinel = {message: 'callback stopped'}, retainedReports = []; let caught;
    try { model.train_with_progress(embedded, options, report => { retainedReports.push(report); throw sentinel; }); } catch (error) { caught = error; }
    assert(caught === sentinel && retainedReports.length === 1 && retainedReports[0].iteration === 0, 'callback exception identity and immediate stop'); retainedReports[0].free(); array(model.write(), sourceWire, 'callback failure retains source model');
    const recovered = model.train_with_progress(embedded, options, report => { report.free(); }); recovered.free();

    const fourFiles = embedded.cloned(), single = embedded.get(0); for (let index = 0; index < 3; index++) fourFiles.push(single); single.free();
    Object.assign(options, defaults, {iterations: 2, termination_threshold: 0, workers: 3}); const ordered = model.train(fourFiles, options), repeated = model.train(fourFiles, options); assert(resultSnapshot(ordered) === resultSnapshot(repeated), 'repeatable native worker reduction on browser backend'); ordered.free(); repeated.free(); fourFiles.free();
    const values = new api.FileLikelihoods(new Float32Array([-0, Infinity, NaN])), rows = new api.LikelihoodRows(); rows.push(values); const rowClone = values.cloned(); values.replace(new Float32Array([1, 2])); rows.replace(0, rowClone); rowClone.free(); values.free();
    const report = new api.IterationReport(0xffffffff, -0, NaN, rows), reportClone = report.cloned(), reports = new api.IterationReports(); reports.push(report); const reportsClone = reports.cloned();
    report.iteration = 2; report.temperature = 3; report.mean_log_likelihood = 4;
    const emptyRows = new api.LikelihoodRows(); report.set_file_likelihoods(emptyRows); emptyRows.free();
    assert(report.iteration === 2 && report.temperature === 3 && report.mean_log_likelihood === 4, 'all report scalar setters'); reports.replace(0, report); rejects(() => reports.replace(1, report), 'report collection index'); report.free(); reports.clear();
    const arbitrary = new api.TrainingResult(model, reportsClone); arbitrary.set_iterations(reports); arbitrary.set_model(model); const copied = arbitrary.cloned(); arbitrary.free();
    const emptyReports = copied.iterations(); assert(emptyReports.length === 0, 'complete result field setters'); emptyReports.free(); copied.free();
    const rawRows = reportClone.file_likelihoods(), rawRow = rawRows.get(0); array(rawRow.values(), [-0, Infinity, NaN], 'all arbitrary likelihood values'); rawRow.free(); rawRows.free(); reportClone.free();
    for (const owner of [rows, reports, reportsClone, noFiles, document, features, embedded, isolated, options, model]) owner.free();
    return {originalCModels: cases.length, wireBytes, reportCount, likelihoodCount, maximumLikelihoodError, optionFields: 13, reportFields: 4, resultFields: 2};
}
