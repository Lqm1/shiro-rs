"""Actual ctypes complete phone maps and original Lua transformation fixtures."""
import ctypes as c
import json
from pathlib import Path
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
symbols = {'phone_options_default', 'phone_map_create', 'phone_map_read_json',
           'phone_map_write_json', 'phone_map_clone', 'phone_map_release',
           'phone_map_to_definition', 'segmentation_initial'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO phones ctypes: all8 symbols absent without c-api')
    sys.exit(0)

class Options(c.Structure):
    _fields_ = [('states_per_phone', N), ('streams', N), ('weak_skips', U)]

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
defaults = function('phone_options_default', [c.POINTER(Options)])
map_create = function('phone_map_create', [P, c.POINTER(Options), P, c.POINTER(P)])
map_read = function('phone_map_read_json', [P, c.POINTER(P)])
map_write = function('phone_map_write_json', [P, c.POINTER(P)])
map_clone = function('phone_map_clone', [P, c.POINTER(P)])
map_release = function('phone_map_release', [c.POINTER(P)])
definition = function('phone_map_to_definition', [P, N, c.c_double, c.POINTER(P)])
initial = function('segmentation_initial', [P, P, N, c.POINTER(P)])
states_write = function('states_write_json', [P, c.POINTER(P)])
states_release = function('states_release', [c.POINTER(P)])
model_create = function('model_from_definition', [P, c.POINTER(P)])
model_release = function('model_release', [c.POINTER(P)])

def owned(data):
    output = P()
    assert create((B * len(data)).from_buffer_copy(data), len(data), c.byref(output)) == 0
    return output

def decoded(owner):
    count = N()
    assert length(owner, c.byref(count)) == 0
    values = (B * count.value)()
    assert copy(owner, 0, values, count.value) == 0
    return json.loads(bytes(values))

def compare(actual, expected):
    if isinstance(actual, dict):
        assert actual.keys() == expected.keys()
        for key in actual:
            compare(actual[key], expected[key])
    elif isinstance(actual, list):
        assert len(actual) == len(expected)
        for a, b in zip(actual, expected):
            compare(a, b)
    elif isinstance(actual, (float, int)) and not isinstance(actual, bool):
        assert abs(actual - expected) <= 1e-14
    else:
        assert actual == expected

root = Path(__file__).parent / 'fixtures'
text = owned((root / 'phones-input.txt').read_bytes())
names = owned(b'["bb","aa","bb","cc","aa"]')
config = Options()
assert defaults(c.byref(config)) == 0
assert [config.states_per_phone, config.streams, config.weak_skips] == [3, 3, 0]
cases = json.loads((root / 'phones-original.json').read_text())
for case in cases:
    config.states_per_phone, config.streams, config.weak_skips = case['count'], 2, 1
    topology = owned(case['topology'].encode())
    original, cloned = P(), P()
    assert map_create(text, c.byref(config), topology, c.byref(original)) == 0
    assert release(c.byref(topology)) == 0
    assert map_clone(original, c.byref(cloned)) == 0
    assert map_release(c.byref(original)) == 0
    encoded = P()
    assert map_write(cloned, c.byref(encoded)) == 0
    compare(decoded(encoded), case['map'])
    assert map_read(encoded, c.byref(original)) == 0
    assert release(c.byref(encoded)) == 0 and map_release(c.byref(cloned)) == 0
    states = P()
    assert initial(names, original, 41, c.byref(states)) == 0
    assert states_write(states, c.byref(encoded)) == 0
    compare(decoded(encoded), case['segmentation']['file_list'][0]['states'])
    assert release(c.byref(encoded)) == 0 and states_release(c.byref(states)) == 0
    if case.get('definition') is not None:
        assert definition(original, 12, 0.01, c.byref(encoded)) == 0
        actual, expected = decoded(encoded), case['definition']
        # Original Lua constraint order is nondeterministic; compare by index.
        for value in [actual, expected]:
            value['dur_attr'].sort(key=lambda row: row['index'])
        compare(actual, expected)
        model = P()
        assert model_create(encoded, c.byref(model)) == 0
        assert model_release(c.byref(model)) == 0 and release(c.byref(encoded)) == 0
    retained = P(original.value)
    config.weak_skips = 2
    assert map_create(text, c.byref(config), None, c.byref(retained)) == 2
    assert retained.value == original.value
    assert definition(original, 0, 0.01, c.byref(encoded)) == 3 and encoded.value is None
    assert map_release(c.byref(original)) == 0

metadata = {'phone_map': {'aa': {'states': [{'dur': 0, 'out': [0], 'extra': {'x': [1, None]}}],
                                 'extra': True}}, 'extra': {'nested': 'retained'}}
encoded = owned(json.dumps(metadata).encode())
original = P()
assert map_read(encoded, c.byref(original)) == 0 and release(c.byref(encoded)) == 0
assert map_write(original, c.byref(encoded)) == 0 and decoded(encoded) == metadata
assert release(c.byref(encoded)) == 0
assert defaults(None) == 1 and map_release(None) == 1 and map_clone(original, None) == 1
assert map_release(c.byref(original)) == 0 and map_release(c.byref(original)) == 0
assert release(c.byref(names)) == 0 and release(c.byref(text)) == 0
assert symbols <= called
print('SHIRO phones ctypes: all8 exports, original Lua maps/definitions/states, metadata, ownership and failures passed')
