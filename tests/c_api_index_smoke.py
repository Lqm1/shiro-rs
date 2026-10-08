"""Actual ctypes complete index fields, native paths and phoneme padding."""
import ctypes as c
import json
from pathlib import Path
import struct
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
symbols = {'strings_create', 'strings_length', 'strings_get', 'strings_clone',
           'strings_release', 'index_entries_create', 'index_entries_length',
           'index_entries_get_stem', 'index_entries_get_phonemes',
           'index_entries_clone', 'index_entries_release', 'index_read_bytes'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO index ctypes: all12 symbols absent without c-api')
    sys.exit(0)
called = set()

def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke

class Entry(c.Structure):
    _fields_ = [('stem', P), ('phonemes', P)]

create_bytes = function('bytes_create', [c.POINTER(B), N, c.POINTER(P)])
byte_length = function('bytes_length', [P, c.POINTER(N)])
copy = function('bytes_copy', [P, N, c.POINTER(B), N])
release_bytes = function('bytes_release', [c.POINTER(P)])
create_strings = function('strings_create', [c.POINTER(P), N, c.POINTER(P)])
string_length = function('strings_length', [P, c.POINTER(N)])
get_string = function('strings_get', [P, N, c.POINTER(P)])
clone_strings = function('strings_clone', [P, c.POINTER(P)])
release_strings = function('strings_release', [c.POINTER(P)])
create_entries = function('index_entries_create', [c.POINTER(Entry), N, c.POINTER(P)])
entry_length = function('index_entries_length', [P, c.POINTER(N)])
get_stem = function('index_entries_get_stem', [P, N, c.POINTER(P)])
get_phones = function('index_entries_get_phonemes', [P, N, c.POINTER(P)])
clone_entries = function('index_entries_clone', [P, c.POINTER(P)])
release_entries = function('index_entries_release', [c.POINTER(P)])
read = function('index_read_bytes', [P, P, P, P, c.POINTER(P)])
utf8 = function('path_from_utf8', [P, c.POINTER(P)])
from_native = function('path_from_native_bytes', [P, c.POINTER(P)])
native_bytes = function('path_native_bytes', [P, c.POINTER(P)])
release_path = function('path_release', [c.POINTER(P)])
encoding = function('path_native_encoding', [])()
assert encoding in (1, 2)

def native(text):
    return text.encode('utf-16le' if encoding == 2 else 'utf-8')

def owned(data):
    result = P()
    assert create_bytes((B * len(data)).from_buffer_copy(data), len(data), c.byref(result)) == 0
    return result

def copied(owner):
    count = N()
    assert byte_length(owner, c.byref(count)) == 0
    data = (B * count.value)()
    assert copy(owner, 0, data, count.value) == 0
    return bytes(data)

def strings(texts):
    owners = [owned(text.encode()) for text in texts]
    pointers = (P * len(owners))(*(owner.value for owner in owners))
    result = P()
    assert create_strings(pointers, len(owners), c.byref(result)) == 0
    for owner in owners:
        assert release_bytes(c.byref(owner)) == 0
    return result

def phones(owner):
    count, result = N(), []
    assert string_length(owner, c.byref(count)) == 0
    for i in range(count.value):
        snapshot = P()
        assert get_string(owner, i, c.byref(snapshot)) == 0
        result.append(copied(snapshot).decode())
        assert release_bytes(c.byref(snapshot)) == 0
    return result

def wire(path):
    result = P()
    assert native_bytes(path, c.byref(result)) == 0
    value = copied(result)
    assert release_bytes(c.byref(result)) == 0
    return value

directory, source = P(), owned(b'data')
assert utf8(source, c.byref(directory)) == 0 and release_bytes(c.byref(source)) == 0
left, right = strings(['left']), strings(['right'])
fixtures = Path(__file__).parent / 'fixtures'
source = owned((fixtures / 'index-original.txt').read_bytes())
entries, count = P(), N()
assert read(source, directory, left, right, c.byref(entries)) == 0
assert entry_length(entries, c.byref(count)) == 0 and count.value == 3
for i, expected in enumerate(json.loads((fixtures / 'index-original.json').read_text())):
    stem, values = P(), P()
    assert get_stem(entries, i, c.byref(stem)) == 0
    assert get_phones(entries, i, c.byref(values)) == 0
    expected_path = 'data' + ('\\' if encoding == 2 else '/') + expected['path'][5:]
    assert wire(stem) == native(expected_path)
    assert phones(values) == expected['phonemes']
    assert release_path(c.byref(stem)) == 0 and release_strings(c.byref(values)) == 0
assert release_entries(c.byref(entries)) == 0 and release_bytes(c.byref(source)) == 0
assert release_strings(c.byref(left)) == 0 and release_strings(c.byref(right)) == 0
left, right = strings(['', 'pad\0\U0001f642']), strings([' right '])
source = owned(b'\r\nclip, aa  bb \r\n\nsilent,\n')
assert read(source, directory, left, right, c.byref(entries)) == 0
for i, expected in enumerate([['', 'pad\0\U0001f642', '', 'aa', '', 'bb', '', ' right '], ['', 'pad\0\U0001f642', ' right ']]):
    values = P()
    assert get_phones(entries, i, c.byref(values)) == 0 and phones(values) == expected
    assert release_strings(c.byref(values)) == 0
assert release_entries(c.byref(entries)) == 0 and release_bytes(c.byref(source)) == 0

for data in [b'\n\r\nbad\n', b'ok,a\n\n,b\n', b'ok,a\n\nclip,a,b\n', b'clip,\xff\n']:
    source = owned(data)
    retained = P(directory.value)
    assert read(source, directory, left, right, c.byref(retained)) == 3 and retained.value == directory.value
    assert release_bytes(c.byref(source)) == 0
assert release_path(c.byref(directory)) == 0
assert release_strings(c.byref(left)) == 0 and release_strings(c.byref(right)) == 0

expected = struct.pack('<4H', 0x66, 0xd800, 0, 0xdc00) if encoding == 2 else b'f\xff\0\x80'
source = owned(expected)
assert from_native(source, c.byref(directory)) == 0 and release_bytes(c.byref(source)) == 0
source, empty = owned(b'repeat\0 '), owned(b'')
values = (P * 3)(source.value, empty.value, source.value)
strings_owner, strings_clone = P(), P()
assert create_strings(values, 3, c.byref(strings_owner)) == 0
assert clone_strings(strings_owner, c.byref(strings_clone)) == 0
assert release_bytes(c.byref(source)) == 0 and release_bytes(c.byref(empty)) == 0
descriptors = (Entry * 2)(Entry(directory, strings_owner), Entry(directory, strings_owner))
assert create_entries(descriptors, 2, c.byref(entries)) == 0
assert release_strings(c.byref(strings_owner)) == 0 and release_path(c.byref(directory)) == 0
cloned = P()
assert clone_entries(entries, c.byref(cloned)) == 0 and release_entries(c.byref(entries)) == 0
stem, values_owner = P(), P()
assert get_stem(cloned, 1, c.byref(stem)) == 0
assert get_phones(cloned, 1, c.byref(values_owner)) == 0
assert entry_length(cloned, c.byref(count)) == 0 and count.value == 2
assert get_stem(cloned, 2, c.byref(stem)) == 2 and get_phones(cloned, 2, c.byref(values_owner)) == 2
assert release_entries(c.byref(cloned)) == 0
assert wire(stem) == expected and phones(values_owner) == phones(strings_clone) == ['repeat\0 ', '', 'repeat\0 ']
source, padding = owned(b'child,\n'), strings([])
assert read(source, stem, padding, padding, c.byref(entries)) == 0
joined = P()
assert get_stem(entries, 0, c.byref(joined)) == 0
assert wire(joined) == expected + native('\\child' if encoding == 2 else '/child')
assert release_path(c.byref(joined)) == 0 and release_entries(c.byref(entries)) == 0
assert release_bytes(c.byref(source)) == 0 and release_strings(c.byref(padding)) == 0
valid, invalid = owned(b'valid'), owned(b'\xff')
retained = P(strings_clone.value)
assert create_strings((P * 2)(valid.value, invalid.value), 2, c.byref(retained)) == 3 and retained.value == strings_clone.value
assert create_strings(None, N(-1).value, c.byref(retained)) == 2
assert create_strings((P * 1)(None), 1, c.byref(retained)) == 1
assert get_string(strings_clone, N(-1).value, c.byref(valid)) == 2 and copied(valid) == b'valid'
assert create_entries(None, 0, c.byref(entries)) == 0
assert entry_length(entries, c.byref(count)) == 0 and count.value == 0
assert release_entries(c.byref(entries)) == 0 and release_entries(c.byref(entries)) == 0
assert release_entries(None) == 1 and release_strings(None) == 1
for owner in [valid, invalid]:
    assert release_bytes(c.byref(owner)) == 0
for owner in [values_owner, strings_clone]:
    assert release_strings(c.byref(owner)) == 0
assert release_path(c.byref(stem)) == 0
assert symbols <= called, symbols - called
print('SHIRO index ctypes: all12 exports, original index/full fields/native units/padding/ownership/failures passed')
