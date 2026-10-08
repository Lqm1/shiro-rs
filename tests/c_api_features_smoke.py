"""Actual ctypes full feature settings/results and all72 original C cases."""
import ctypes as c
import math
from pathlib import Path
import struct
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, F = c.c_void_p, c.c_size_t, c.c_uint32, c.c_float
symbols = {'feature_options_default', 'features_create', 'features_extract',
           'features_get_info', 'features_get_values', 'features_clone', 'features_release'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO features ctypes: all7 symbols absent without c-api')
    sys.exit(0)

class Options(c.Structure):
    _fields_ = [('kind', U), ('order', N), ('channels', N), ('frame_length', N),
                ('hop', F), ('sample_rate_hz', F), ('minimum_bandwidth_hz', F), ('warp', F),
                ('include_dc', U), ('energy', U), ('delta', U), ('acceleration', U)]

class Info(c.Structure):
    _fields_ = [('frames', N), ('columns', N)]

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
default = function('feature_options_default', [c.POINTER(Options)])
create = function('features_create', [N, N, P, c.POINTER(P)])
extract = function('features_extract', [P, c.POINTER(Options), c.POINTER(P)])
get_info = function('features_get_info', [P, c.POINTER(Info)])
get_values = function('features_get_values', [P, c.POINTER(P)])
clone = function('features_clone', [P, c.POINTER(P)])
release = function('features_release', [c.POINTER(P)])

def options():
    result = Options()
    assert default(c.byref(result)) == 0
    return result

def array(values):
    result = P()
    assert array_create(values, len(values), c.byref(result)) == 0
    return result

def copied(owner):
    count = N()
    assert array_length(owner, c.byref(count)) == 0
    result = (F * count.value)()
    assert array_copy(owner, 0, result, count.value) == 0
    return result

def extracted(signal, settings):
    owner, result, snapshot = array(signal), P(), P()
    assert extract(owner, c.byref(settings), c.byref(result)) == 0
    assert array_release(c.byref(owner)) == 0
    assert clone(result, c.byref(snapshot)) == 0 and release(c.byref(result)) == 0
    info, values = Info(), P()
    assert get_info(snapshot, c.byref(info)) == 0 and get_values(snapshot, c.byref(values)) == 0
    assert release(c.byref(snapshot)) == 0
    output = copied(values)
    assert array_release(c.byref(values)) == 0
    return info, output

settings = options()
assert [getattr(settings, name) for name, _ in Options._fields_] == [0, 12, 36, 1024, 256, 32000, 400, 1, 0, 0, 0, 0]
wire = (Path(__file__).parent / 'fixtures' / 'c-xxcc.bin').read_bytes()
offset = 0
def integer():
    global offset
    value, = struct.unpack_from('<I', wire, offset)
    offset += 4
    return value
def scalar():
    global offset
    value, = struct.unpack_from('<f', wire, offset)
    offset += 4
    return value
assert wire[:4] == b'XCC1'
offset = 4
assert integer() == 72
total = 0
for record in range(72):
    settings = options()
    settings.kind, settings.energy, flags = integer(), integer(), integer()
    settings.frame_length, settings.hop, count = integer(), scalar(), integer()
    signal = (F * count)(*[scalar() for _ in range(count)])
    frames, columns = integer(), integer()
    settings.channels, settings.sample_rate_hz, settings.warp = 12, 16000, 0.85
    settings.include_dc, settings.delta, settings.acceleration = flags & 1, (flags >> 1) & 1, (flags >> 2) & 1
    info, values = extracted(signal, settings)
    assert [info.frames, info.columns, len(values)] == [frames, columns, frames * columns]
    for index, actual in enumerate(values):
        expected = scalar()
        if math.isnan(expected):
            assert math.isnan(actual), (record, index)
        elif math.isinf(expected):
            assert actual == expected, (record, index)
        else:
            assert abs(actual - expected) / max(abs(expected), 1) < 2e-5, (record, index, actual, expected)
        total += 1
assert offset == len(wire)

# Arbitrary public fields preserve all integer and float bits independently.
bits = [0x80000000, 1, 0x7fc12345, 0x7f800000]
raw = struct.pack('<4I', *bits)
source = array((F * 4).from_buffer_copy(raw))
result, snapshot, values = P(), P(), P()
assert create(N(-1).value, N(-2).value, source, c.byref(result)) == 0
assert clone(result, c.byref(snapshot)) == 0 and release(c.byref(result)) == 0
assert array_release(c.byref(source)) == 0
info = Info()
assert get_info(snapshot, c.byref(info)) == 0
assert [info.frames, info.columns] == [N(-1).value, N(-2).value]
assert get_values(snapshot, c.byref(values)) == 0
retained = P(snapshot.value)
source = array((F * 0)())
for field, value in [('kind', 3), ('energy', 3), ('include_dc', 2), ('delta', 2), ('acceleration', 2)]:
    settings = options()
    setattr(settings, field, value)
    assert extract(source, c.byref(settings), c.byref(retained)) == 2 and retained.value == snapshot.value
settings = options()
settings.hop = 0
assert extract(source, c.byref(settings), c.byref(retained)) == 3 and retained.value == snapshot.value
assert extract(source, None, c.byref(retained)) == 1 and retained.value == snapshot.value
assert create(0, 0, None, c.byref(retained)) == 1 and retained.value == snapshot.value
assert get_info(None, c.byref(info)) == 1 and info.frames == N(-1).value
assert get_values(snapshot, None) == 1 and clone(snapshot, None) == 1 and default(None) == 1 and release(None) == 1
assert release(c.byref(snapshot)) == 0 and bytes(copied(values)) == raw
assert array_release(c.byref(values)) == 0 and array_release(c.byref(source)) == 0
assert release(c.byref(snapshot)) == 0
settings = options()
settings.hop, settings.include_dc, settings.energy, settings.delta, settings.acceleration = 17.5, 1, 1, 1, 1
info, values = extracted((F * 0)(), settings)
assert [info.frames, info.columns, len(values)] == [0, 42, 0]
assert symbols <= called, symbols - called
print(f'SHIRO features ctypes: all7 exports, all72 C cases/{total} values, complete fields/bits and ownership passed')
