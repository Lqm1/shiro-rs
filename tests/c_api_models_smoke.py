"""Actual ctypes calls for complete SHIRO-owned model interchange."""
import ctypes as c
from pathlib import Path
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
model_symbols = {'model_from_definition', 'model_read_bytes', 'model_write_bytes', 'model_clone', 'model_release'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in model_symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO models ctypes: all5 model symbols absent without c-api')
    sys.exit(0)
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
release_bytes = function('bytes_release', [c.POINTER(P)])
definition = function('model_from_definition', [P, c.POINTER(P)])
read = function('model_read_bytes', [P, N, c.POINTER(P)])
write = function('model_write_bytes', [P, U, c.POINTER(P)])
clone = function('model_clone', [P, c.POINTER(P)])
release = function('model_release', [c.POINTER(P)])

def owned(data):
    result = P()
    buffer = (B * len(data)).from_buffer_copy(data)
    assert create(buffer, len(data), c.byref(result)) == 0
    return result

def copied(owner):
    count = N()
    assert length(owner, c.byref(count)) == 0
    buffer = (B * count.value)()
    assert copy(owner, 0, buffer, count.value) == 0
    return bytes(buffer)

root = Path(__file__).parent / 'fixtures'
source = owned((root / 'modeldef.json').read_bytes())
model, cloned, wire = P(), P(), P()
assert definition(source, c.byref(model)) == 0
assert release_bytes(c.byref(source)) == 0
assert clone(model, c.byref(cloned)) == 0
assert release(c.byref(model)) == 0 and model.value is None
assert release(c.byref(model)) == 0
assert write(cloned, 0, c.byref(wire)) == 0
assert release(c.byref(cloned)) == 0
assert copied(wire) == (root / 'empty-c.hsmm').read_bytes()
assert release_bytes(c.byref(wire)) == 0
for filename, encoding in [('empty-c.hsmm', 0), ('cmu-arctic-all-speakers.hsmm', 1), ('init-c-multi.hsmm', 0), ('untie-c-weighted-input.hsmm', 0), ('rest-c-isolated-daem.hsmm', 0), ('utterances-c-trained.hsmm', 0)]:
    original = (root / filename).read_bytes()
    source = owned(original)
    assert read(source, 16 * 1024 * 1024, c.byref(model)) == 0
    sentinel = P(model.value)
    assert read(source, 0, c.byref(sentinel)) == 3 and sentinel.value == model.value
    assert clone(None, c.byref(sentinel)) == 1 and sentinel.value == model.value
    assert definition(None, c.byref(sentinel)) == 1 and sentinel.value == model.value
    assert clone(model, None) == 1 and read(source, N(-1).value, None) == 1
    assert write(model, encoding, c.byref(wire)) == 0
    assert copied(wire) == original
    retained = P(wire.value)
    assert write(model, 2, c.byref(retained)) == 2 and retained.value == wire.value
    assert write(model, 0, None) == 1
    assert release_bytes(c.byref(source)) == 0
    assert release(c.byref(model)) == 0
    assert read(wire, 16 * 1024 * 1024, c.byref(model)) == 0
    assert release_bytes(c.byref(wire)) == 0
    assert release(c.byref(model)) == 0
assert release(None) == 1
assert model_symbols <= called
print('SHIRO models ctypes: all5 exports, original definition/model bytes, both schemas and owner/error transactions passed')
