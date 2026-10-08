"""Actual ctypes complete audio, legacy random streams and controlled callbacks."""
import ctypes as c
from pathlib import Path
import struct
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, H, B, F = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint16, c.c_uint8, c.c_float
symbols = {'audio_options_default', 'audio_create', 'audio_prepare', 'audio_prepare_wave_bytes',
           'audio_sample_rate', 'audio_get_samples', 'audio_clone', 'audio_release',
           'dither_windows', 'dither_linux_gnu', 'dither_next_uniform', 'dither_release'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO audio ctypes: all12 symbols absent without c-api')
    sys.exit(0)

class WaveInfo(c.Structure):
    _fields_ = [('sample_rate', U), ('bits_per_sample', H), ('channels', H), ('encoding', U)]
class Options(c.Structure):
    _fields_ = [('normalize', U), ('dither_level', F), ('has_output_sample_rate', U),
                ('output_sample_rate', U), ('boundary', U), ('kernel', U)]
Callback = c.CFUNCTYPE(U, P, c.POINTER(F))
called = set()
def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke
array_create = function('array_f32_create', [c.POINTER(F), N, c.POINTER(P)])
array_length = function('array_f32_length', [P, c.POINTER(N)])
array_copy = function('array_f32_copy', [P, N, c.POINTER(F), N])
array_release = function('array_f32_release', [c.POINTER(P)])
bytes_create = function('bytes_create', [c.POINTER(B), N, c.POINTER(P)])
bytes_release = function('bytes_release', [c.POINTER(P)])
default = function('audio_options_default', [c.POINTER(Options)])
create = function('audio_create', [U, P, c.POINTER(P)])
prepare = function('audio_prepare', [P, c.POINTER(WaveInfo), c.POINTER(Options), Callback, P, c.POINTER(P)])
prepare_wave = function('audio_prepare_wave_bytes', [P, N, c.POINTER(Options), Callback, P, c.POINTER(P)])
sample_rate = function('audio_sample_rate', [P, c.POINTER(U)])
get_samples = function('audio_get_samples', [P, c.POINTER(P)])
clone = function('audio_clone', [P, c.POINTER(P)])
release = function('audio_release', [c.POINTER(P)])
windows = function('dither_windows', [c.POINTER(P)])
linux = function('dither_linux_gnu', [c.POINTER(P)])
next_uniform = function('dither_next_uniform', [P, c.POINTER(F)])
free_rng = function('dither_release', [c.POINTER(P)])
no_callback = Callback()

def settings():
    output = Options()
    assert default(c.byref(output)) == 0
    return output
def array(values):
    output = P()
    assert array_create(values, len(values), c.byref(output)) == 0
    return output
def owned(data):
    output = P()
    assert bytes_create((B * len(data)).from_buffer_copy(data), len(data), c.byref(output)) == 0
    return output
def copied(owner):
    count = N()
    assert array_length(owner, c.byref(count)) == 0
    output = (F * count.value)()
    assert array_copy(owner, 0, output, count.value) == 0
    return output
def snapshot(owner):
    cloned, samples, rate = P(), P(), U()
    assert clone(owner, c.byref(cloned)) == 0 and release(c.byref(owner)) == 0
    assert sample_rate(cloned, c.byref(rate)) == 0 and get_samples(cloned, c.byref(samples)) == 0
    assert release(c.byref(cloned)) == 0
    result = copied(samples)
    assert array_release(c.byref(samples)) == 0
    return rate.value, result

options = settings()
assert [getattr(options, name) for name, _ in Options._fields_] == [0, 0, 0, 0, 0, 0]
root = Path(__file__).parent / 'fixtures'
wire = owned((root / 'c-audio-input.wav').read_bytes())
for normalize, rate, name in [(0, None, 'plain'), (1, None, 'normalized'),
                             (0, 32000, 'up'), (0, 8000, 'down'), (1, 8000, 'normalized-down')]:
    options = settings()
    options.normalize, options.boundary, options.kernel = normalize, 1, 1
    if rate is not None:
        options.has_output_sample_rate, options.output_sample_rate = 1, rate
    output = P()
    assert prepare_wave(wire, 1024, c.byref(options), no_callback, None, c.byref(output)) == 0
    actual_rate, actual = snapshot(output)
    expected_raw = (root / f'c-audio-input.{name}.raw').read_bytes()
    expected = struct.unpack('<' + 'f' * (len(expected_raw) // 4), expected_raw)
    assert actual_rate == (rate or 16000) and len(actual) == len(expected)
    assert all(abs(a - e) / max(abs(e), 1) < 2e-7 for a, e in zip(actual, expected))

@Callback
def sequence(context, output):
    return next_uniform(context, output)

for constructor, filename in [(windows, 'c-dither-windows.bin'), (linux, 'c-dither-linux.bin')]:
    fixture = (root / filename).read_bytes()
    rng = P()
    assert constructor(c.byref(rng)) == 0 and next_uniform(rng, None) == 1
    for index in range(64):
        draw = F()
        assert next_uniform(rng, c.byref(draw)) == 0
        assert bytes(draw) == fixture[12 + index * 8 + 4:12 + index * 8 + 8]
    assert free_rng(c.byref(rng)) == 0 and constructor(c.byref(rng)) == 0
    source, output = array((F * 64)()), P()
    header, options = WaveInfo(8000, 32, 1, 1), settings()
    options.dither_level = 1
    assert prepare(source, c.byref(header), c.byref(options), sequence, rng, c.byref(output)) == 0
    rate, actual = snapshot(output)
    assert rate == 8000 and bytes(actual) == fixture[12 + 64 * 8:]
    assert array_release(c.byref(source)) == 0 and free_rng(c.byref(rng)) == 0
    assert free_rng(c.byref(rng)) == 0
rng, output = P(), P()
assert linux(c.byref(rng)) == 0
options = settings()
options.dither_level = 0.125
assert prepare_wave(wire, 1024, c.byref(options), sequence, rng, c.byref(output)) == 0
assert bytes(snapshot(output)[1]) == (root / 'c-audio-input.dither-linux.raw').read_bytes()
assert free_rng(c.byref(rng)) == 0

state = {'position': 0, 'fail_at': None}
@Callback
def controlled(context, output):
    position = state['position']
    state['position'] += 1
    if position == state['fail_at'] or position >= 3:
        return 7
    output[0] = [0, 0.5, 1][position]
    return 0
source = array((F * 3)(-0.25, 0.5, 0))
header = WaveInfo(8000, 0, 65535, 0)
options = settings()
options.normalize, options.dither_level = 1, 0.125
output = P()
assert prepare(source, c.byref(header), c.byref(options), controlled, None, c.byref(output)) == 0
rate, actual = snapshot(output)
assert rate == 8000 and list(actual) == [-0.625, 1, 0.125] and state['position'] == 3

raw = struct.pack('<4I', 0x80000000, 1, 0x7fc12345, 0x7f800000)
arbitrary = array((F * 4).from_buffer_copy(raw))
owner = P()
assert create(U(-1).value, arbitrary, c.byref(owner)) == 0 and array_release(c.byref(arbitrary)) == 0
retained = P(owner.value)
state.update(position=0, fail_at=1)
assert prepare(source, c.byref(header), c.byref(options), controlled, None, c.byref(retained)) == 3
assert retained.value == owner.value and state['position'] == 2
assert prepare(source, c.byref(header), c.byref(options), no_callback, None, c.byref(retained)) == 3
assert retained.value == owner.value
for field in ['normalize', 'has_output_sample_rate', 'boundary', 'kernel']:
    invalid = settings()
    setattr(invalid, field, 2)
    assert prepare(source, c.byref(header), c.byref(invalid), no_callback, None, c.byref(retained)) == 2
    assert retained.value == owner.value
options = settings()
options.has_output_sample_rate = 1
assert prepare(source, c.byref(header), c.byref(options), no_callback, None, c.byref(retained)) == 3
assert retained.value == owner.value
options.has_output_sample_rate, options.output_sample_rate, options.dither_level = 0, U(-1).value, -1
state['position'] = 0
assert prepare(source, c.byref(header), c.byref(options), controlled, None, c.byref(output)) == 0
assert state['position'] == 0 and list(snapshot(output)[1]) == [-0.25, 0.5, 0]
assert prepare_wave(wire, 1, c.byref(options), no_callback, None, c.byref(retained)) == 3
assert retained.value == owner.value
assert prepare(source, None, c.byref(options), no_callback, None, c.byref(retained)) == 1
assert retained.value == owner.value
assert default(None) == 1 and get_samples(owner, None) == 1 and clone(owner, None) == 1
assert create(0, None, c.byref(retained)) == 1 and retained.value == owner.value
assert windows(None) == 1 and linux(None) == 1 and free_rng(None) == 1 and release(None) == 1
rate, actual = snapshot(owner)
assert rate == U(-1).value and bytes(actual) == raw
assert array_release(c.byref(source)) == 0 and bytes_release(c.byref(wire)) == 0
assert symbols <= called, symbols - called
print('SHIRO audio ctypes: all12 exports, original5 wave cases/both64-draw sequences/full fields/callbacks and ownership passed')
