function assert(value, message) { if (!value) throw new Error(message); }
function equal(actual, expected, message) { assert(JSON.stringify(actual) === JSON.stringify(expected), message); }
function rejects(run, message) { let error; try { run(); } catch (value) {error = value;} assert(error !== undefined, message); return String(error); }
function snapshot(entries) { const values = []; for (let index = 0; index < entries.length; index++) { const entry = entries.get(index); values.push({path: entry.stem, phonemes: JSON.parse(entry.phonemes_json())}); entry.free(); } return values; }
export async function verifyIndex(api, load) {
    const encoder = new TextEncoder(), decode = bytes => new TextDecoder().decode(bytes);
    const original = api.IndexEntries.read(new Uint8Array(await load('index-original.txt')), 'data', '["left"]', '["right"]'), expected = JSON.parse(decode(await load('index-original.json')));
    equal(snapshot(original), expected, 'complete original Lua index'); assert(original.length === 3, 'original index rows'); assert(original.get(99) === undefined, 'absent entry');
    const cases = [
        ['first,aa\r\n\r\nsecond,\r\n', '.', [], [], [{path: './first', phonemes: ['aa']}, {path: './second', phonemes: []}]],
        ['\n\r\n', '.', [], [], []],
        ['clip, aa  bb \n', '', [], [], [{path: 'clip', phonemes: ['', 'aa', '', 'bb', '']}]],
        ['\u97f3,\u3042 \u3044\n/absolute,z\n', 'dir', ['', '\u5de6'], ['\u53f3', ''], [{path: 'dir/\u97f3', phonemes: ['', '\u5de6', '\u3042', '\u3044', '\u53f3', '']}, {path: '/absolute', phonemes: ['', '\u5de6', 'z', '\u53f3', '']}]],
        ['sub/../clip,aa\tbb\n', 'root/', [], [], [{path: 'root/sub/../clip', phonemes: ['aa\tbb']}]],
    ];
    for (const [text, directory, left, right, wanted] of cases) { const entries = api.IndexEntries.read(encoder.encode(text), directory, JSON.stringify(left), JSON.stringify(right)); equal(snapshot(entries), wanted, 'native index padding/token/path rules'); entries.free(); }
    for (const text of ['first,aa\n\nbad\n', 'first,aa\n\nbad,aa,bb\n', 'first,aa\n\n,aa\n']) assert(rejects(() => api.IndexEntries.read(encoder.encode(text), '.', '[]', '[]'), 'invalid row').includes('line 3'), 'physical error line retained');
    rejects(() => api.IndexEntries.read(new Uint8Array([255]), '.', '[]', '[]'), 'invalid UTF-8 index');
    for (const [left, right] of [['[1]', '[]'], ['[]', '{}'], ['[', '[]']]) rejects(() => api.IndexEntries.read(encoder.encode('a,b'), '.', left, right), 'invalid padding JSON');
    for (const [stem, suffix, wanted] of [['data/sub/clip.two', '.param', 'data/sub/clip.two.param'], ['', '.raw', '.raw'], ['\u97f3', '\u58f0', '\u97f3\u58f0'], ['a/', '/b', 'a//b'], ['a\u0000b', '\u0000', 'a\u0000b\u0000']]) assert(api.index_append_suffix(stem, suffix) === wanted, 'literal suffix addition');
    const entry = new api.IndexEntry('source', '["", "arbitrary\\u0000phone"]'), copy = entry.cloned(), entries = new api.IndexEntries(); entries.push(entry); entry.stem = 'changed'; entry.set_phonemes_json('["changed"]');
    equal(snapshot(entries), [{path: 'source', phonemes: ['', 'arbitrary\u0000phone']}], 'push copies all fields'); equal(JSON.parse(copy.phonemes_json()), ['', 'arbitrary\u0000phone'], 'complete entry clone');
    rejects(() => entry.set_phonemes_json('[1]'), 'atomic phoneme replacement'); equal(JSON.parse(entry.phonemes_json()), ['changed'], 'failed replacement retains values');
    entries.replace(0, entry); equal(snapshot(entries), [{path: 'changed', phonemes: ['changed']}], 'both fields editable'); rejects(() => entries.replace(1, copy), 'out of range replacement'); equal(snapshot(entries), [{path: 'changed', phonemes: ['changed']}], 'failed replacement atomic');
    const collectionCopy = entries.cloned(); entries.clear(); assert(entries.length === 0, 'collection clear'); entries.free(); entry.free(); copy.free(); equal(snapshot(collectionCopy), [{path: 'changed', phonemes: ['changed']}], 'collection survives source release'); collectionCopy.free();
    rejects(() => new api.IndexEntry('', '[false]'), 'invalid entry phonemes'); const recovered = api.IndexEntries.read(encoder.encode('ok,yes'), '.', '[]', '[]'); assert(recovered.length === 1, 'recovery after invalid input'); recovered.free(); original.free();
    return {originalLuaRows: 3, additionalCases: cases.length, entryFields: 2, literalSuffixCases: 5};
}
