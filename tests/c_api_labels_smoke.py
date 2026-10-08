"""Actual ctypes complete labels and original Lua conversion checks."""
import ctypes as c
import json
from pathlib import Path
import struct
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B, D = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8, c.c_double
symbols = {'labels_create', 'labels_length', 'labels_get_info', 'labels_get_name',
           'labels_clone', 'labels_release', 'labels_parse', 'labels_to_states',
           'labels_from_states', 'labels_write_bytes', 'labels_output_path'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO labels ctypes: all11 symbols absent without c-api')
    sys.exit(0)

class Input(c.Structure):
    _fields_ = [('start', D), ('end', D), ('name', P)]

class Info(c.Structure):
    _fields_ = [('start', D), ('end', D)]

called = set()
def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke

create = function('bytes_create', [c.POINTER(B), N, c.POINTER(P)])
length = function('bytes_length', [P, c.POINTER(N)])
copy = function('bytes_copy', [P, N, c.POINTER(B), N])
release = function('bytes_release', [c.POINTER(P)])
labels_create = function('labels_create', [c.POINTER(Input), N, c.POINTER(P)])
labels_length = function('labels_length', [P, c.POINTER(N)])
info = function('labels_get_info', [P, N, c.POINTER(Info)])
name = function('labels_get_name', [P, N, c.POINTER(P)])
clone = function('labels_clone', [P, c.POINTER(P)])
free = function('labels_release', [c.POINTER(P)])
parse = function('labels_parse', [P, c.POINTER(P)])
to_states = function('labels_to_states', [P, P, D, c.POINTER(P)])
from_states = function('labels_from_states', [P, D, U, c.POINTER(P)])
write = function('labels_write_bytes', [P, c.POINTER(P)])
output_path = function('labels_output_path', [P, P, c.POINTER(P)])
map_read = function('phone_map_read_json', [P, c.POINTER(P)])
map_free = function('phone_map_release', [c.POINTER(P)])
states_write = function('states_write_json', [P, c.POINTER(P)])
states_free = function('states_release', [c.POINTER(P)])

def owned(data):
    output = P()
    assert create((B * len(data)).from_buffer_copy(data), len(data), c.byref(output)) == 0
    return output

def copied(owner):
    count = N()
    assert length(owner, c.byref(count)) == 0
    values = (B * count.value)()
    assert copy(owner, 0, values, count.value) == 0
    return bytes(values)

def rows(owner):
    count = N()
    assert labels_length(owner, c.byref(count)) == 0
    values = []
    for index in range(count.value):
        times, encoded = Info(), P()
        assert info(owner, index, c.byref(times)) == 0
        assert name(owner, index, c.byref(encoded)) == 0
        values.append((times.start, times.end, copied(encoded).decode()))
        assert release(c.byref(encoded)) == 0
    return values

def original_rows(text):
    return [(float(a), float(b), n) for a, b, n in (line.split('\t') for line in text.splitlines())]

root = Path(__file__).parent / 'fixtures'
text, labels = owned((root / 'labels-input.txt').read_bytes()), P()
assert parse(text, c.byref(labels)) == 0 and release(c.byref(text)) == 0
assert rows(labels) == original_rows((root / 'labels-input.txt').read_text())
cloned = P()
assert clone(labels, c.byref(cloned)) == 0 and free(c.byref(labels)) == 0
encoded, map_owner = owned((root / 'labels-phones.json').read_bytes()), P()
assert map_read(encoded, c.byref(map_owner)) == 0 and release(c.byref(encoded)) == 0
states = P()
assert to_states(cloned, map_owner, 0.01, c.byref(states)) == 0
retained_states = P(states.value)
for hop in [0, -1, float('nan'), float('inf')]:
    assert to_states(cloned, map_owner, hop, c.byref(retained_states)) == 3
    assert retained_states.value == states.value
assert free(c.byref(cloned)) == 0 and map_free(c.byref(map_owner)) == 0
assert states_write(states, c.byref(encoded)) == 0
expected_states = json.loads((root / 'labels-original-seg.json').read_text())['file_list'][0]['states']
assert json.loads(copied(encoded)) == expected_states and release(c.byref(encoded)) == 0
for include, fixture in [(0, 'labels-original-phones.txt'), (1, 'labels-original-states.txt')]:
    assert from_states(states, 0.01, include, c.byref(labels)) == 0
    actual, expected = rows(labels), original_rows((root / fixture).read_text())
    assert len(actual) == len(expected)
    for a, b in zip(actual, expected):
        assert a[2] == b[2] and abs(a[0] - b[0]) < 1e-14 and abs(a[1] - b[1]) < 1e-14
    assert write(labels, c.byref(encoded)) == 0
    wire = copied(encoded)
    assert wire.count(b'\r\n') == len(expected) and wire.count(b'\n') == len(expected)
    assert parse(encoded, c.byref(cloned)) == 0 and rows(cloned) == actual
    assert free(c.byref(cloned)) == 0 and release(c.byref(encoded)) == 0
    retained = P(labels.value)
    assert from_states(states, 0.01, 2, c.byref(retained)) == 2 and retained.value == labels.value
    assert from_states(states, 0, 0, c.byref(retained)) == 3 and retained.value == labels.value
    assert free(c.byref(labels)) == 0
assert states_free(c.byref(states)) == 0

name_wire = 'retained\0Unicodeλ'.encode()
encoded = owned(name_wire)
bits = [0x8000000000000000, 1, 0x7ff8000000000042, 0xfff0000000000000]
as_float = lambda bits: struct.unpack('<d', struct.pack('<Q', bits))[0]
inputs = (Input * 2)(Input(as_float(bits[0]), as_float(bits[1]), encoded.value),
                     Input(as_float(bits[2]), as_float(bits[3]), encoded.value))
assert labels_create(inputs, 2, c.byref(labels)) == 0
assert release(c.byref(encoded)) == 0
for index in range(2):
    times = Info()
    assert info(labels, index, c.byref(times)) == 0
    assert c.string_at(c.byref(times), c.sizeof(times)) == struct.pack('<QQ', *bits[index * 2:index * 2 + 2])
snapshot = P()
assert name(labels, 0, c.byref(snapshot)) == 0
retained = P(labels.value)
bad = owned(b'0 1 aa\n\nx 2 bb')
assert parse(bad, c.byref(retained)) == 3 and retained.value == labels.value
assert release(c.byref(bad)) == 0
times = Info(12, 13)
assert info(labels, 2, c.byref(times)) == 2 and [times.start, times.end] == [12, 13]
retained_bytes = P(snapshot.value)
assert name(labels, 2, c.byref(retained_bytes)) == 2 and retained_bytes.value == snapshot.value
assert free(c.byref(labels)) == 0 and copied(snapshot) == name_wire
valid_name, bad_name = owned(b'valid'), owned(b'bad\tname')
inputs = (Input * 2)(Input(0, 1, valid_name.value), Input(1, 2, bad_name.value))
assert labels_create(inputs, 2, c.byref(labels)) == 0
assert write(labels, c.byref(retained_bytes)) == 3 and retained_bytes.value == snapshot.value
retained = P(labels.value)
inputs[1].name = None
assert labels_create(inputs, 2, c.byref(retained)) == 1 and retained.value == labels.value
assert labels_create(None, N(-1).value, c.byref(retained)) == 2 and retained.value == labels.value
assert free(c.byref(labels)) == 0
assert release(c.byref(valid_name)) == 0 and release(c.byref(bad_name)) == 0
empty = P()
assert labels_create(None, 0, c.byref(empty)) == 0
assert write(empty, c.byref(encoded)) == 0 and copied(encoded) == b''
assert release(c.byref(encoded)) == 0 and free(c.byref(empty)) == 0
for filename, expected in [('dir.name/clip.features.f', 'dir.name/clip.features.txt'),
                           ('dir.name\\clip', 'dir.name\\clip.txt'), ('.f', '.txt'), ('clip', 'clip.txt')]:
    encoded, suffix, result = owned(filename.encode()), owned(b'.txt'), P()
    assert output_path(encoded, suffix, c.byref(result)) == 0 and copied(result) == expected.encode()
    assert release(c.byref(result)) == 0 and release(c.byref(encoded)) == 0 and release(c.byref(suffix)) == 0
assert free(None) == 1 and clone(None, c.byref(labels)) == 1 and info(None, 0, c.byref(times)) == 1
assert release(c.byref(snapshot)) == 0
assert symbols <= called
print('SHIRO labels ctypes: all11 exports, original Lua rows/states, complete binary64 bits/names, CRLF, paths and failures passed')
