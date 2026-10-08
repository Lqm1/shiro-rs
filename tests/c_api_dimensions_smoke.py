"""Actual ctypes model dimensions and lossless target-width owned arrays."""
import ctypes as c
from pathlib import Path
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
symbols = {'array_usize_create', 'array_usize_length', 'array_usize_copy',
           'array_usize_clone', 'array_usize_release', 'model_dimensions'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO dimensions ctypes: all6 symbols absent without c-api')
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
release_bytes = function('bytes_release', [c.POINTER(P)])
model_read = function('model_read_bytes', [P, N, c.POINTER(P)])
model_build = function('model_from_definition', [P, c.POINTER(P)])
model_release = function('model_release', [c.POINTER(P)])
create = function('array_usize_create', [c.POINTER(N), N, c.POINTER(P)])
length = function('array_usize_length', [P, c.POINTER(N)])
copy = function('array_usize_copy', [P, N, c.POINTER(N), N])
clone = function('array_usize_clone', [P, c.POINTER(P)])
release = function('array_usize_release', [c.POINTER(P)])
dimensions = function('model_dimensions', [P, c.POINTER(P)])

def owned(data):
    output = P()
    assert create_bytes((B * len(data)).from_buffer_copy(data), len(data), c.byref(output)) == 0
    return output

def copied(values):
    count = N()
    assert length(values, c.byref(count)) == 0
    output = (N * count.value)()
    assert copy(values, 0, output, count.value) == 0
    return list(output)

root = Path(__file__).parent / 'fixtures'
for name, expected in [('empty-c.hsmm', [2, 1]), ('init-c-multi.hsmm', [2, 1])]:
    wire, model, values, snapshot = owned((root / name).read_bytes()), P(), P(), P()
    assert model_read(wire, 16 * 1024 * 1024, c.byref(model)) == 0
    assert release_bytes(c.byref(wire)) == 0
    assert dimensions(model, c.byref(values)) == 0 and clone(values, c.byref(snapshot)) == 0
    assert release(c.byref(values)) == 0
    assert dimensions(model, c.byref(values)) == 0 and model_release(c.byref(model)) == 0
    assert copied(values) == copied(snapshot) == expected
    assert release(c.byref(values)) == 0 and release(c.byref(snapshot)) == 0

definition = owned(b'{"ndurstate":1,"streamdef":[{"nstate":1,"ndim":7},{"nstate":2,"ndim":1},{"nstate":1,"ndim":19}]}')
model, values = P(), P()
assert model_build(definition, c.byref(model)) == 0 and release_bytes(c.byref(definition)) == 0
assert dimensions(model, c.byref(values)) == 0 and dimensions(model, None) == 1
assert model_release(c.byref(model)) == 0 and copied(values) == [7, 1, 19]
assert release(c.byref(values)) == 0

maximum = N(-1).value
source = (N * 4)(0, maximum, maximum - 1, 1)
assert create(source, 4, c.byref(values)) == 0
source[1] = 42
assert copied(values) == [0, maximum, maximum - 1, 1]
output = (N * 4)(73, 73, 73, 73)
assert copy(values, 1, output, 2) == 0 and list(output) == [maximum, maximum - 1, 73, 73]
for offset, count in [(3, 2), (maximum, 1), (5, 0)]:
    output = (N * 4)(73, 73, 73, 73)
    assert copy(values, offset, output, count) == 2 and list(output) == [73] * 4
assert copy(values, 4, None, 0) == 0 and copy(values, 0, None, 1) == 1
retained = P(values.value)
assert create(None, 1, c.byref(retained)) == 1 and retained.value == values.value
assert create(None, maximum, c.byref(retained)) == 2 and retained.value == values.value
misaligned = c.cast(P(1), c.POINTER(N))
assert create(misaligned, 1, c.byref(retained)) == 1 and retained.value == values.value
assert dimensions(None, c.byref(retained)) == 1 and retained.value == values.value
assert clone(values, None) == 1 and length(values, None) == 1 and release(None) == 1
snapshot = P()
assert clone(values, c.byref(snapshot)) == 0 and release(c.byref(values)) == 0
assert copied(snapshot) == [0, maximum, maximum - 1, 1] and release(c.byref(snapshot)) == 0
assert create(None, 0, c.byref(values)) == 0 and copied(values) == []
assert release(c.byref(values)) == 0 and release(c.byref(values)) == 0
assert symbols <= called, symbols - called
print('SHIRO dimensions ctypes: all6 exports, complete stream order/full-width values and ownership passed')
