"""Independent ctypes rawfloat/JSON/legacy segmentation wire oracle."""
import ctypes as c
import json
import struct
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
symbols = {'observation_read_rawfloat', 'observation_from_model_rawfloat', 'observation_write_bytes', 'observation_clone', 'observation_release', 'states_read_json', 'states_write_json', 'states_clone', 'states_release', 'states_segmentation_bytes'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO samples ctypes: all10 symbols absent without c-api')
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
model_create = function('model_from_definition', [P, c.POINTER(P)])
model_release = function('model_release', [c.POINTER(P)])
read = function('observation_read_rawfloat', [P, c.POINTER(N), N, N, c.POINTER(P)])
read_model = function('observation_from_model_rawfloat', [P, P, N, c.POINTER(P)])
write = function('observation_write_bytes', [P, c.POINTER(P)])
clone = function('observation_clone', [P, c.POINTER(P)])
release = function('observation_release', [c.POINTER(P)])
states_read = function('states_read_json', [P, c.POINTER(P)])
states_write = function('states_write_json', [P, c.POINTER(P)])
states_clone = function('states_clone', [P, c.POINTER(P)])
states_release = function('states_release', [c.POINTER(P)])
segmentation = function('states_segmentation_bytes', [P, P, c.POINTER(P)])
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
def scalar_bits(bits):
    return b'\xca' + struct.pack('>I', bits)
def scalar(value):
    return b'\xca' + struct.pack('>f', value)
patterns = [0, 0x80000000, 1, 0x80000001, 0x3f800001, 0x3f000000]
raw = b''.join(struct.pack('<I', bits) for bits in patterns)
expected = b'\x93\x02\x92\x02\x01\x92\x94' + b''.join(scalar_bits(patterns[i]) for i in [0,1,3,4]) + b'\x92' + scalar_bits(patterns[2]) + scalar_bits(patterns[5])
dims = (N * 2)(2, 1)
source = owned(raw)
observation, cloned, wire = P(), P(), P()
assert read(source, dims, 2, 2, c.byref(observation)) == 0
assert clone(observation, c.byref(cloned)) == 0
assert release(c.byref(observation)) == 0
assert write(cloned, c.byref(wire)) == 0 and copied(wire) == expected
assert release_bytes(c.byref(wire)) == 0
retained = P(cloned.value)
assert read(source, dims, 2, 1, c.byref(retained)) == 3 and retained.value == cloned.value
assert read(source, None, 1, 2, c.byref(retained)) == 1 and retained.value == cloned.value
assert read(source, None, N(-1).value, 2, c.byref(retained)) == 2 and retained.value == cloned.value
assert clone(cloned, None) == 1 and write(cloned, None) == 1
assert release(c.byref(cloned)) == 0
configuration = owned(b'{"ndurstate":4,"streamdef":[{"nstate":3,"ndim":2},{"nstate":3,"ndim":1}]}')
model = P()
assert model_create(configuration, c.byref(model)) == 0
assert release_bytes(c.byref(configuration)) == 0
assert read_model(source, model, 2, c.byref(observation)) == 0
assert release_bytes(c.byref(source)) == 0
assert write(observation, c.byref(wire)) == 0 and copied(wire) == expected
assert release_bytes(c.byref(wire)) == 0 and release(c.byref(observation)) == 0
states_value = [{"time":1.75,"dur":0,"out":[0,0],"jmp":[{"d":0,"p":0.123456789,"edge":"keep"},{"d":1,"p":0.9}],"ext":["aa",0,{"nested":[True,None,5]}],"custom":{"value":"keep"}}, {"time":4.125,"dur":1,"out":[1,1],"jmp":[],"ext":["aa",1,"tail"]}]
source = owned(json.dumps(states_value).encode())
states, cloned = P(), P()
assert states_read(source, c.byref(states)) == 0
assert release_bytes(c.byref(source)) == 0
assert states_clone(states, c.byref(cloned)) == 0
assert states_release(c.byref(states)) == 0
assert states_write(cloned, c.byref(wire)) == 0
assert json.loads(copied(wire)) == states_value
assert release_bytes(c.byref(wire)) == 0
expected_seg = bytes([0x93,0x92,1,4,0x92,0,1,0x92,0x92,0,1,0x92,0,1,0x92,0x92,0]) + scalar(0.123456789) + b'\x01' + scalar(1.0 - 0.123456789) + b'\x91\x01' + scalar(1.0)
assert segmentation(cloned, model, c.byref(wire)) == 0 and copied(wire) == expected_seg
assert release_bytes(c.byref(wire)) == 0
retained = P(cloned.value)
source = owned(b'[] []')
assert states_read(source, c.byref(retained)) == 3 and retained.value == cloned.value
assert states_clone(cloned, None) == 1 and states_write(cloned, None) == 1
assert segmentation(cloned, None, c.byref(wire)) == 1 and wire.value is None
assert release_bytes(c.byref(source)) == 0
assert states_release(c.byref(cloned)) == 0 and states_release(c.byref(cloned)) == 0
assert model_release(c.byref(model)) == 0
assert states_release(None) == 1 and release(None) == 1
assert symbols <= called
print('SHIRO samples ctypes: all10 exports, complete deinterleaving bits, metadata and independent legacy segmentation wire passed')
