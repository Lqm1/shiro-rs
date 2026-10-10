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
function assignmentSnapshot(assignments) {
    const values = [];
    for (let index = 0; index < assignments.length; index++) { const value = assignments.get(index); values.push([value.state, value.file, value.segment]); value.free(); }
    return values;
}

export async function verifyUntying(api, load) {
    const model = api.Model.read(new Uint8Array(await load('init-c-aligned.hsmm'))), originalModel = model.write();
    const input = JSON.parse(new TextDecoder().decode(await load('align-c-isolated.json'))), document = new api.SegmentationDocument(JSON.stringify(input)), originalDocument = document.json();
    const expectedWire = new Uint8Array(await load('untie-c.hsmm')), expectedDocument = JSON.parse(new TextDecoder().decode(await load('untie-c.json'))), expectedSummary = new Uint8Array(await load('untie-c-summary.txt'));
    const result = model.untie(document), untied = result.model(), segmentation = result.segmentation(), assignments = result.assignments();
    array(untied.write(), expectedWire, 'original C complete model'); json(JSON.parse(segmentation.json()), expectedDocument, 'original C complete document'); array(result.write_summary(), expectedSummary, 'original C summary bytes');
    json(assignmentSnapshot(assignments), Array.from({length: 6}, (_, index) => [index, 0, index]), 'all ordered assignments'); assert(assignments.get(6) === null, 'absent assignment');
    const sourceCopy = result.cloned(); result.free(); array(sourceCopy.write_summary(), expectedSummary, 'result clone survives source release'); sourceCopy.free();
    for (const owner of [untied, segmentation, assignments]) owner.free();

    const weighted = api.Model.read(new Uint8Array(await load('untie-c-weighted-input.hsmm'))), weightedResult = weighted.untie(document), weightedModel = weightedResult.model();
    for (const [index, value] of [[0, .25], [1, 1.75]]) {
        const stream = weightedModel.stream(index); array(stream.weight(), [value], 'preserved stream weight correction'); stream.set_weight(new Float32Array([1])); weightedModel.set_stream(index, stream); stream.free();
    }
    array(weightedModel.write(), expectedWire, 'all remaining parameters match C after documented weight correction'); weightedModel.free(); weightedResult.free();

    const enhanced = structuredClone(input); enhanced.corpus = {nested: [true, null, 'retained']}; enhanced.file_list[0].filename = 'not-opened.f'; enhanced.file_list[0].custom = ['retained'];
    enhanced.file_list[0].states[0].jmp = [{d: 2, p: .1, edge: 'retained'}]; enhanced.file_list[0].states[0].annotation = {nested: [1, 'retained']}; enhanced.file_list.push(structuredClone(enhanced.file_list[0]));
    const enhancedDocument = new api.SegmentationDocument(JSON.stringify(enhanced)), many = weighted.untie(enhancedDocument), manyModel = many.model(), manyDocument = many.segmentation(), manyAssignments = many.assignments();
    assert(manyModel.durations === 12 && manyModel.streams === 2, 'complete multi-file model shape');
    json(assignmentSnapshot(manyAssignments), Array.from({length: 12}, (_, index) => [index, Math.floor(index / 6), index % 6]), 'complete global/file/local order');
    const expectedMany = structuredClone(enhanced); expectedMany.file_list.forEach((file, fileIndex) => file.states.forEach((state, index) => { state.dur = fileIndex * 6 + index; state.out = [state.dur, state.dur]; }));
    json(JSON.parse(manyDocument.json()), expectedMany, 'all nested attributes and jumps retained');
    const stream = manyModel.stream(0), first = stream.emission(0), duplicate = stream.emission(6), originalMeans = duplicate.means();
    const changedMeans = first.means(); changedMeans[0] = 100; first.replace(first.dimensions, first.weights(), changedMeans, first.variances(), first.variance_floors()); stream.set_emission(0, first); manyModel.set_stream(0, stream);
    const rereadDuplicate = stream.emission(6); array(rereadDuplicate.means(), originalMeans, 'untied repeated parameters remain independent'); rereadDuplicate.free();
    for (const owner of [stream, first, duplicate]) owner.free();
    many.set_model(manyModel); many.set_segmentation(manyDocument); many.set_assignments(manyAssignments);
    const saved = many.cloned(); for (const owner of [many, manyModel, manyDocument, manyAssignments, enhancedDocument, weighted]) owner.free();
    const savedModel = saved.model(), savedStream = savedModel.stream(0), savedEmission = savedStream.emission(0); assert(savedEmission.means()[0] === 100, 'complete model setter and clone'); savedEmission.free(); savedStream.free(); savedModel.free(); saved.free();

    const row = new api.Assignment(0xffffffff, 0, 0), rowCopy = row.cloned(), rows = new api.Assignments(); rows.push(row); row.state = 2; row.file = 1; row.segment = 3;
    json([row.state, row.file, row.segment], [2, 1, 3], 'all assignment fields mutable'); rows.replace(0, rowCopy); rowCopy.free(); row.free();
    const rowsCopy = rows.cloned(), arbitrary = new api.UntiedModel(model, document, rowsCopy); rows.clear(); rows.free(); rowsCopy.free();
    assert(new TextDecoder().decode(arbitrary.write_summary()).startsWith('4294967295 0 0 a 0\n'), 'arbitrary full-width assignment state');
    const copiedRows = arbitrary.assignments(), invalid = new api.Assignment(1, 999, 0); copiedRows.replace(0, invalid); rejects(() => copiedRows.replace(1, invalid), 'assignment index error'); arbitrary.set_assignments(copiedRows); rejects(() => arbitrary.write_summary(), 'invalid summary assignment'); invalid.free(); copiedRows.free();
    const emptyRows = new api.Assignments(); arbitrary.set_assignments(emptyRows); emptyRows.free(); assert(arbitrary.write_summary().length === 0, 'full assignment shrink'); arbitrary.free();
    for (const bad of [{dur: 999}, {out: [0]}, {out: [0, 999]}, {dur: null}]) {
        const value = structuredClone(input); Object.assign(value.file_list[0].states[0], bad); const badDocument = new api.SegmentationDocument(JSON.stringify(value)); rejects(() => model.untie(badDocument), 'invalid state reference'); badDocument.free();
    }
    for (const metadata of [[], ['a'], [3, 0], ['a', 0x80000000], ['phone with space', -1.75]]) {
        const value = structuredClone(input); value.file_list[0].states[0].ext = metadata; const metadataDocument = new api.SegmentationDocument(JSON.stringify(value)), metadataResult = model.untie(metadataDocument);
        if (metadata.length === 0) assert(new TextDecoder().decode(metadataResult.write_summary()).startsWith('0 0 0\n'), 'optional summary metadata');
        else if (typeof metadata[0] === 'string' && metadata.length === 2 && metadata[1] === -1.75) assert(new TextDecoder().decode(metadataResult.write_summary()).startsWith('0 0 0 phone with space -1\n'), 'native truncating signed summary index');
        else rejects(() => metadataResult.write_summary(), 'invalid summary metadata'); metadataResult.free(); metadataDocument.free();
    }
    const emptyDocument = new api.SegmentationDocument('{"file_list":[]}'), empty = model.untie(emptyDocument), emptyModel = empty.model(); assert(emptyModel.durations === 0 && emptyModel.streams === 2 && empty.write_summary().length === 0, 'empty untying retains stream count'); emptyModel.free(); empty.free(); emptyDocument.free();
    array(model.write(), originalModel, 'input model unchanged'); assert(document.json() === originalDocument, 'input document unchanged');
    const finalResult = model.untie(document); model.free(); document.free(); array(finalResult.write_summary(), expectedSummary, 'summary survives all source release'); finalResult.free();
    return {originalCModelBytes: expectedWire.length, originalCSummaryBytes: expectedSummary.length, originalAssignments: 6, multiFileAssignments: 12, resultFields: 3, assignmentFields: 3};
}
