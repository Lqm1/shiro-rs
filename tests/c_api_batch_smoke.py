"""Actual ctypes complete native, SPTK and Lua batch extractors and full fields."""
import ctypes as c
import os
from pathlib import Path
import struct
import sys
import tempfile

lib = c.CDLL(sys.argv[1])
P, N, U, B, F = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8, c.c_float
symbols = {'batch_preset_feature_options', 'batch_options_default', 'batch_options_create',
           'batch_options_get_audio', 'batch_options_get_input_extension', 'batch_options_clone',
           'batch_options_release', 'extractor_native', 'extractor_sptk_default', 'extractor_sptk',
           'extractor_lua', 'extractor_kind', 'extractor_get_preset', 'extractor_get_path',
           'extractor_clone', 'extractor_release', 'batch_outputs_create', 'batch_outputs_has_mfcc',
           'batch_outputs_get_path', 'batch_outputs_clone', 'batch_outputs_release', 'batch_extract_file'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO batch ctypes: all22 symbols absent without c-api')
    sys.exit(0)
called = set()

def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke

class Audio(c.Structure):
    _fields_ = [('normalize', U), ('dither_level', F), ('has_output_sample_rate', U),
               ('output_sample_rate', U), ('boundary', U), ('kernel', U)]

class Feature(c.Structure):
    _fields_ = [('kind', U), ('order', N), ('channels', N), ('frame_length', N),
               ('hop', F), ('sample_rate_hz', F), ('minimum_bandwidth_hz', F), ('warp', F),
               ('include_dc', U), ('energy', U), ('delta', U), ('acceleration', U)]

Uniform = c.CFUNCTYPE(U, P, c.POINTER(F))
create_bytes = function('bytes_create', [c.POINTER(B), N, c.POINTER(P)])
byte_length = function('bytes_length', [P, c.POINTER(N)])
copy = function('bytes_copy', [P, N, c.POINTER(B), N])
release_bytes = function('bytes_release', [c.POINTER(P)])
create_path = function('path_from_native_bytes', [P, c.POINTER(P)])
native_bytes = function('path_native_bytes', [P, c.POINTER(P)])
release_path = function('path_release', [c.POINTER(P)])
encoding = function('path_native_encoding', [])()
preset_settings = function('batch_preset_feature_options', [U, c.POINTER(Feature)])
default_options = function('batch_options_default', [c.POINTER(P)])
create_options = function('batch_options_create', [c.POINTER(Audio), P, c.POINTER(P)])
get_audio = function('batch_options_get_audio', [P, c.POINTER(Audio)])
get_extension = function('batch_options_get_input_extension', [P, c.POINTER(P)])
clone_options = function('batch_options_clone', [P, c.POINTER(P)])
release_options = function('batch_options_release', [c.POINTER(P)])
native_extractor = function('extractor_native', [U, c.POINTER(P)])
sptk_default = function('extractor_sptk_default', [c.POINTER(P)])
sptk = function('extractor_sptk', [P, P, P, c.POINTER(P)])
lua = function('extractor_lua', [P, P, P, c.POINTER(P)])
get_kind = function('extractor_kind', [P, c.POINTER(U)])
get_preset = function('extractor_get_preset', [P, c.POINTER(U)])
extractor_path = function('extractor_get_path', [P, N, c.POINTER(P)])
clone_extractor = function('extractor_clone', [P, c.POINTER(P)])
release_extractor = function('extractor_release', [c.POINTER(P)])
create_outputs = function('batch_outputs_create', [P, P, P, c.POINTER(P)])
has_mfcc = function('batch_outputs_has_mfcc', [P, c.POINTER(U)])
output_path = function('batch_outputs_get_path', [P, N, c.POINTER(P)])
clone_outputs = function('batch_outputs_clone', [P, c.POINTER(P)])
release_outputs = function('batch_outputs_release', [c.POINTER(P)])
extract = function('batch_extract_file', [P, P, P, Uniform, P, c.POINTER(P)])

def owned(data):
    owner = P()
    assert create_bytes((B * len(data)).from_buffer_copy(data), len(data), c.byref(owner)) == 0
    return owner

def copied(owner):
    count = N()
    assert byte_length(owner, c.byref(count)) == 0
    output = (B * count.value)()
    assert copy(owner, 0, output, count.value) == 0
    return bytes(output)

def native(text):
    return os.fspath(text).encode('utf-16le' if encoding == 2 else 'utf-8')

def path_wire(owner):
    snapshot = P()
    assert native_bytes(owner, c.byref(snapshot)) == 0
    result = copied(snapshot)
    assert release_bytes(c.byref(snapshot)) == 0
    return result

def path(data):
    source, owner = owned(data), P()
    assert create_path(source, c.byref(owner)) == 0 and release_bytes(c.byref(source)) == 0
    return owner

def fields(owner, stem, mfcc):
    flag = U(99)
    assert has_mfcc(owner, c.byref(flag)) == 0 and flag.value == mfcc
    for i, suffix in enumerate(['.raw', '.param', '.mfcc']):
        snapshot = P()
        assert output_path(owner, i, c.byref(snapshot)) == (0 if i < 2 or mfcc else 2)
        if snapshot.value:
            assert path_wire(snapshot) == native(str(stem) + suffix)
            assert release_path(c.byref(snapshot)) == 0

options, snapshot = P(), P()
assert default_options(c.byref(options)) == 0 and get_extension(options, c.byref(snapshot)) == 0
assert copied(snapshot) == b'.wav' and release_bytes(c.byref(snapshot)) == 0
audio = Audio()
assert get_audio(options, c.byref(audio)) == 0
assert [getattr(audio, name) for name, _ in Audio._fields_] == [0, 0, 0, 0, 0, 0]
assert release_options(c.byref(options)) == 0
audio = Audio(1, 0.125, 1, 8000, 1, 1)
source = owned(b'.source\0.wav')
assert create_options(c.byref(audio), source, c.byref(options)) == 0 and release_bytes(c.byref(source)) == 0
cloned = P()
assert clone_options(options, c.byref(cloned)) == 0 and release_options(c.byref(options)) == 0
actual_audio = Audio()
assert get_audio(cloned, c.byref(actual_audio)) == 0
assert [getattr(actual_audio, name) for name, _ in Audio._fields_] == [getattr(audio, name) for name, _ in Audio._fields_]
assert get_extension(cloned, c.byref(snapshot)) == 0 and release_options(c.byref(cloned)) == 0
assert copied(snapshot) == b'.source\0.wav' and release_bytes(c.byref(snapshot)) == 0
units = struct.pack('<4H', 0x66, 0xd800, 0, 0xdc00) if encoding == 2 else b'f\xff\0\x80'
source, empty = path(units), path(b'')
for make, expected_kind in [(sptk, 1), (lua, 2)]:
    extractor, clone = P(), P()
    assert make(source, source, source, c.byref(extractor)) == 0
    assert clone_extractor(extractor, c.byref(clone)) == 0 and release_extractor(c.byref(extractor)) == 0
    kind, code = U(99), U(99)
    assert get_kind(clone, c.byref(kind)) == 0 and kind.value == expected_kind
    assert get_preset(clone, c.byref(code)) == 2 and code.value == 99
    snapshots = [P(), P(), P()]
    for i, snapshot in enumerate(snapshots):
        assert extractor_path(clone, i, c.byref(snapshot)) == 0
    assert release_extractor(c.byref(clone)) == 0
    for snapshot in snapshots:
        assert path_wire(snapshot) == units and release_path(c.byref(snapshot)) == 0
for present in [0, 1]:
    output, clone = P(), P()
    assert create_outputs(source, source, empty if present else None, c.byref(output)) == 0
    assert clone_outputs(output, c.byref(clone)) == 0 and release_outputs(c.byref(output)) == 0
    flag = U(99)
    assert has_mfcc(clone, c.byref(flag)) == 0 and flag.value == present
    for i in range(3):
        snapshot = P()
        assert output_path(clone, i, c.byref(snapshot)) == (0 if i < 2 or present else 2)
        if snapshot.value:
            assert path_wire(snapshot) == (units if i < 2 else b'')
            assert release_path(c.byref(snapshot)) == 0
    assert release_outputs(c.byref(clone)) == 0
assert release_path(c.byref(source)) == 0 and release_path(c.byref(empty)) == 0
extractor = P()
assert sptk_default(c.byref(extractor)) == 0
for i, expected in enumerate(['frame', 'mfcc', 'delta']):
    snapshot = P()
    assert extractor_path(extractor, i, c.byref(snapshot)) == 0 and path_wire(snapshot) == native(expected)
    assert release_path(c.byref(snapshot)) == 0
assert release_extractor(c.byref(extractor)) == 0
fixtures = Path(__file__).parent / 'fixtures'
no_uniform = Uniform()
with tempfile.TemporaryDirectory(prefix='shiro-batch-') as temporary:
    root, options = Path(temporary), P()
    stem = root / 'clip with spaces.v1'
    Path(str(stem) + '.wav').write_bytes((fixtures / 'c-audio-input.wav').read_bytes())
    stem_owner = path(native(stem))
    assert default_options(c.byref(options)) == 0
    for code, original in enumerate(['c-fextr-mfcc12-da.bin', 'c-fextr-mfcc12-dae.bin', 'c-fextr-plpcc12-da.bin']):
        feature, extractor, output = Feature(), P(), P()
        assert preset_settings(code, c.byref(feature)) == 0
        assert [getattr(feature, name) for name, _ in Feature._fields_] == [2 if code == 2 else 0, 12, 36, 512, 80, 16000, 400, 1, 0, 1 if code == 1 else 0, 1, 1]
        assert native_extractor(code, c.byref(extractor)) == 0
        actual_code, kind = U(99), U(99)
        assert get_preset(extractor, c.byref(actual_code)) == 0 and actual_code.value == code
        assert get_kind(extractor, c.byref(kind)) == 0 and kind.value == 0
        assert extract(stem_owner, options, extractor, no_uniform, None, c.byref(output)) == 0
        fields(output, stem, 0)
        assert Path(str(stem) + '.raw').read_bytes() == (fixtures / 'c-audio-input.plain.raw').read_bytes()
        actual, expected = Path(str(stem) + '.param').read_bytes(), (fixtures / original).read_bytes()
        assert len(actual) == len(expected)
        for (a,), (e,) in zip(struct.iter_unpack('<f', actual), struct.iter_unpack('<f', expected)):
            assert abs(a - e) / max(abs(e), 1) < 2e-5
        retained = P(output.value)
        assert native_extractor(3, c.byref(extractor)) == 2
        assert extract(stem_owner, options, extractor, no_uniform, None, None) == 1
        missing = path(native(root / 'missing'))
        assert extract(missing, options, extractor, no_uniform, None, c.byref(retained)) == 3 and retained.value == output.value
        assert release_path(c.byref(missing)) == 0
        assert release_outputs(c.byref(output)) == 0 and release_extractor(c.byref(extractor)) == 0
    suffix = '.exe' if encoding == 2 else ''
    tools = Path(os.environ['SHIRO_TEST_SPTK_DIRECTORY'])
    programs = [path(native(tools / (name + suffix))) for name in ['frame', 'mfcc', 'delta']]
    extractor, output = P(), P()
    assert sptk(*programs, c.byref(extractor)) == 0
    assert extract(stem_owner, options, extractor, no_uniform, None, c.byref(output)) == 0
    fields(output, stem, 1)
    for extension in ['.raw', '.mfcc', '.param']:
        assert Path(str(stem) + extension).read_bytes() == (fixtures / 'c-audio-input.plain.raw').read_bytes()
    assert release_extractor(c.byref(extractor)) == 0
    failing = path(native(tools / ('mfcc-fail' + suffix)))
    assert sptk(programs[0], failing, programs[2], c.byref(extractor)) == 0
    retained = P(output.value)
    assert extract(stem_owner, options, extractor, no_uniform, None, c.byref(retained)) == 3 and retained.value == output.value
    assert release_extractor(c.byref(extractor)) == 0 and release_outputs(c.byref(output)) == 0
    for program in programs + [failing]:
        assert release_path(c.byref(program)) == 0
    script = root / 'extractor.lua'
    script.write_text("return function(try_execute, stem, rawfile, prefix) assert(try_execute == _G.try_execute); assert(#prefix > 0); local f = assert(io.open(rawfile, 'rb')); local bytes = f:read('*a'); f:close(); local o = assert(io.open(stem .. '.param', 'wb')); o:write(bytes); o:close() end\n")
    paths = [path(native(os.environ['SHIRO_TEST_LUA'])), path(native(script)), path(native(root))]
    assert lua(*paths, c.byref(extractor)) == 0
    assert extract(stem_owner, options, extractor, no_uniform, None, c.byref(output)) == 0
    fields(output, stem, 0)
    assert Path(str(stem) + '.param').read_bytes() == (fixtures / 'c-audio-input.plain.raw').read_bytes()
    script.write_text('return 42\n')
    retained = P(output.value)
    assert extract(stem_owner, options, extractor, no_uniform, None, c.byref(retained)) == 3 and retained.value == output.value
    assert release_extractor(c.byref(extractor)) == 0 and release_outputs(c.byref(output)) == 0
    for value in paths:
        assert release_path(c.byref(value)) == 0
    assert release_options(c.byref(options)) == 0
    audio = Audio(0, 0.125, 0, 0, 0, 0)
    source = owned(b'.wav')
    assert create_options(c.byref(audio), source, c.byref(options)) == 0 and release_bytes(c.byref(source)) == 0
    assert native_extractor(0, c.byref(extractor)) == 0
    draws = []
    @Uniform
    def uniform(context, value):
        draws.append(1)
        value[0] = 0.5
        return 0
    assert extract(stem_owner, options, extractor, uniform, None, c.byref(output)) == 0 and draws
    raw = Path(str(stem) + '.raw').read_bytes()
    assert raw == (fixtures / 'c-audio-input.plain.raw').read_bytes()
    @Uniform
    def failed(context, value):
        return 7
    retained = P(output.value)
    assert extract(stem_owner, options, extractor, failed, None, c.byref(retained)) == 3 and retained.value == output.value
    assert Path(str(stem) + '.raw').read_bytes() == raw
    assert release_outputs(c.byref(output)) == 0 and release_extractor(c.byref(extractor)) == 0
    assert release_options(c.byref(options)) == 0 and release_path(c.byref(stem_owner)) == 0
assert symbols <= called, symbols - called
print('SHIRO batch ctypes: all22 exports, all3 presets/full fields/actual Lua/SPTK protocol/RNG/ownership/failures passed')
