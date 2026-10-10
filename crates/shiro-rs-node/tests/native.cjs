const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const os = require('node:os');
const { pathToFileURL } = require('node:url');
const { test } = require('node:test');
const api = require('..');
const root = path.resolve(__dirname, '../../..');
const load = name => fs.readFile(path.join(root, 'tests/fixtures', name));

test('all original C/Lua computational families and connected workflow', async t => {
  const names = ['alignment', 'audio', 'batch', 'data', 'features', 'index', 'initialization', 'interchange', 'isolation', 'loading', 'models', 'phones', 'training', 'untying', 'utterances', 'tool_workflow'];
  for (const name of names) {
    await t.test(name, async () => {
      const checks = await import(pathToFileURL(path.join(__dirname, `oracles/native_${name}_checks.mjs`)));
      const verifier = Object.values(checks).find(value => typeof value === 'function');
      assert.ok(verifier, `verifier for ${name}`);
      await verifier(api, load);
    });
  }
});

test('native file extraction and saved model loading', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'shiro-node-'));
  try {
    const stem = path.join(directory, 'speech');
    await fs.writeFile(`${stem}.wav`, await load('utterances-input.wav'));
    const outputs = api.batch_extract_file(stem, new api.BatchOptions(), api.Extractor.native(0), () => 0.5);
    assert.ok((await fs.stat(outputs.raw)).size > 0);
    assert.ok((await fs.stat(outputs.parameters)).size > 0);
    const model = api.Model.read(await load('init-c-aligned.hsmm'));
    const destination = path.join(directory, 'model.hsmm');
    model.write_file(destination, 0);
    const restored = api.Model.read_file(destination);
    assert.deepEqual(restored.write(), model.write());
    assert.equal(api.version(), '0.1.0');
  } finally {
    await fs.rm(directory, { recursive: true, force: true });
  }
});
test('failed dither callback leaves no output files', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'shiro-failure-'));
  try {
    const stem = path.join(directory, 'speech');
    await fs.writeFile(`${stem}.wav`, await load('utterances-input.wav'));
    const options = new api.BatchOptions();
    const audio = options.audio();
    audio.dither_level = 0.01;
    options.set_audio(audio);
    const failure = { reason: 'stop before output' };
    let calls = 0;
    assert.throws(() => api.batch_extract_file(stem, options, api.Extractor.native(0), () => {
      calls += 1;
      throw failure;
    }), error => error === failure);
    assert.equal(calls, 1);
    assert.deepEqual(await fs.readdir(directory), ['speech.wav']);
  } finally {
    await fs.rm(directory, { recursive: true, force: true });
  }
});
test('native dataset paths and alignment', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'shiro-data-'));
  try {
    const dataPath = path.join(directory, 'input.f');
    await fs.writeFile(dataPath, await load('init-input.bin'));
    const source = JSON.parse(await load('align-c-isolated.json'));
    for (const entry of source.file_list) entry.filename = dataPath;
    const document = new api.SegmentationDocument(JSON.stringify(source));
    const model = api.Model.read(await load('init-c-aligned.hsmm'));
    assert.ok(api.Dataset.load_files(document, model, 12).observations().length > 0);
    assert.ok(api.Datasets.load_training_paths(document, model, 12, false).length > 0);
    const aligned = model.align_document_files(document, new api.AlignmentOptions());
    assert.equal(JSON.parse(aligned.json()).file_list[0].filename, dataPath);
  } finally {
    await fs.rm(directory, { recursive: true, force: true });
  }
});
