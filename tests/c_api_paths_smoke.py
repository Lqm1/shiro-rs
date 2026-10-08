"""Actual ctypes complete native paths, lossless units and suffix ownership."""
import ctypes as c
import struct
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
symbols = {'path_native_encoding', 'path_from_native_bytes', 'path_from_utf8',
           'path_native_bytes', 'path_append_suffix', 'path_clone', 'path_release'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO paths ctypes: all7 symbols absent without c-api')
    sys.exit(0)
called = set()
def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke
create_bytes = function('bytes_create', [c.POINTER(B), N, c.POINTER(P)])
length = function('bytes_length', [P, c.POINTER(N)])
copy = function('bytes_copy', [P, N, c.POINTER(B), N])
release_bytes = function('bytes_release', [c.POINTER(P)])
encoding = function('path_native_encoding', [])
create = function('path_from_native_bytes', [P, c.POINTER(P)])
utf8 = function('path_from_utf8', [P, c.POINTER(P)])
native_bytes = function('path_native_bytes', [P, c.POINTER(P)])
append = function('path_append_suffix', [P, P, c.POINTER(P)])
clone = function('path_clone', [P, c.POINTER(P)])
release = function('path_release', [c.POINTER(P)])
def owned(data):
    output = P()
    assert create_bytes((B * len(data)).from_buffer_copy(data), len(data), c.byref(output)) == 0
    return output
def copied(owner):
    count = N()
    assert length(owner, c.byref(count)) == 0
    output = (B * count.value)()
    assert copy(owner, 0, output, count.value) == 0
    return bytes(output)
def wire(path):
    output = P()
    assert native_bytes(path, c.byref(output)) == 0
    result = copied(output)
    assert release_bytes(c.byref(output)) == 0
    return result
kind = encoding()
assert kind in [1, 2]
def native(text):
    return text.encode('utf-16le' if kind == 2 else 'utf-8')
expected = struct.pack('<6H', 0x66, 0xd800, 0, 0xdc00, 0x2f, 0x6f) if kind == 2 else bytes([0x66, 0xff, 0x80, 0, 0xfe, 0x2f, 0x6f])
source, path, cloned = owned(expected), P(), P()
assert create(source, c.byref(path)) == 0 and release_bytes(c.byref(source)) == 0
assert clone(path, c.byref(cloned)) == 0 and release(c.byref(path)) == 0
suffix = '.param\0\U0001f642'
source = owned(suffix.encode())
assert append(cloned, source, c.byref(path)) == 0 and release_bytes(c.byref(source)) == 0
original, appended = P(), P()
assert native_bytes(cloned, c.byref(original)) == 0 and native_bytes(path, c.byref(appended)) == 0
assert release(c.byref(cloned)) == 0 and release(c.byref(path)) == 0
assert copied(original) == expected and copied(appended) == expected + native(suffix)
assert release_bytes(c.byref(original)) == 0 and release_bytes(c.byref(appended)) == 0
for text in ['', 'data/sub/voice\0\U0001f642', 'C:\\folder\\clip.wav']:
    source, path = owned(text.encode()), P()
    assert utf8(source, c.byref(path)) == 0 and release_bytes(c.byref(source)) == 0
    assert wire(path) == native(text)
    invalid, retained = owned(b'\xff'), P(path.value)
    assert utf8(invalid, c.byref(retained)) == 3 and retained.value == path.value
    assert append(path, invalid, c.byref(retained)) == 3 and retained.value == path.value
    if kind == 2:
        assert create(invalid, c.byref(retained)) == 3 and retained.value == path.value
    assert create(None, c.byref(retained)) == 1 and retained.value == path.value
    assert utf8(None, c.byref(retained)) == 1 and retained.value == path.value
    assert clone(path, None) == 1 and native_bytes(path, None) == 1 and release(None) == 1
    assert release_bytes(c.byref(invalid)) == 0 and release(c.byref(path)) == 0 and release(c.byref(path)) == 0
assert symbols <= called, symbols - called
print('SHIRO paths ctypes: all7 exports, complete native units/Unicode/empty/suffixes and independent ownership passed')
