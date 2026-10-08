"""Actual ctypes standalone groups, complete positions, samples and metadata."""
import ctypes as c
import copy
import json
from pathlib import Path
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
symbols = {'isolated_groups_create', 'isolated_groups', 'isolated_groups_length',
           'isolated_groups_get_info', 'isolated_groups_get_observation',
           'isolated_groups_get_states', 'isolated_groups_clone', 'isolated_groups_release'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO isolation ctypes: all8 symbols absent without c-api')
    sys.exit(0)

class GroupInput(c.Structure):
    _fields_ = [('first_state', N), ('first_frame', N), ('observation', P), ('states', P)]

class GroupInfo(c.Structure):
    _fields_ = [('first_state', N), ('first_frame', N)]

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
copy_bytes = function('bytes_copy', [P, N, c.POINTER(B), N])
release = function('bytes_release', [c.POINTER(P)])
model_read = function('model_read_bytes', [P, N, c.POINTER(P)])
model_release = function('model_release', [c.POINTER(P)])
observation_read = function('observation_from_model_rawfloat', [P, P, N, c.POINTER(P)])
observation_write = function('observation_write_bytes', [P, c.POINTER(P)])
observation_release = function('observation_release', [c.POINTER(P)])
states_read = function('states_read_json', [P, c.POINTER(P)])
states_write = function('states_write_json', [P, c.POINTER(P)])
states_release = function('states_release', [c.POINTER(P)])
create = function('isolated_groups_create', [c.POINTER(GroupInput), N, c.POINTER(P)])
isolate = function('isolated_groups', [P, P, P, c.POINTER(P)])
count = function('isolated_groups_length', [P, c.POINTER(N)])
info = function('isolated_groups_get_info', [P, N, c.POINTER(GroupInfo)])
observation = function('isolated_groups_get_observation', [P, N, c.POINTER(P)])
states = function('isolated_groups_get_states', [P, N, c.POINTER(P)])
clone = function('isolated_groups_clone', [P, c.POINTER(P)])
free = function('isolated_groups_release', [c.POINTER(P)])

def owned(data):
    result = P()
    assert create_bytes((B * len(data)).from_buffer_copy(data), len(data), c.byref(result)) == 0
    return result

def copied(owner):
    size = N()
    assert length(owner, c.byref(size)) == 0
    result = (B * size.value)()
    assert copy_bytes(owner, 0, result, size.value) == 0
    return bytes(result)

def read_states(value):
    wire, result = owned(json.dumps(value).encode()), P()
    assert states_read(wire, c.byref(result)) == 0 and release(c.byref(wire)) == 0
    return result

def serialized(writer, owner):
    wire = P()
    assert writer(owner, c.byref(wire)) == 0
    data = copied(wire)
    assert release(c.byref(wire)) == 0
    return data

root = Path(__file__).parent / 'fixtures'
wire, model = owned((root / 'init-c-aligned.hsmm').read_bytes()), P()
assert model_read(wire, 16 * 1024 * 1024, c.byref(model)) == 0 and release(c.byref(wire)) == 0
raw = (root / 'init-input.bin').read_bytes()
wire, source = owned(raw), P()
assert observation_read(wire, model, 12, c.byref(source)) == 0 and release(c.byref(wire)) == 0
original = json.loads((root / 'align-c-isolated.json').read_text())['file_list'][0]['states']
for enhanced in [False, True]:
    value = copy.deepcopy(original)
    if enhanced:
        value[0]['jmp'] = [{'d': 1, 'p': 0.9}, {'d': 4, 'p': 0.1}]
        value[0]['extra'] = {'nested': [1, None, 'retained']}
        value[3]['ext'].append({'retained': True})
        value[5]['time'] = 40
    state_owner, groups, cloned = read_states(value), P(), P()
    assert isolate(model, source, state_owner, c.byref(groups)) == 0
    assert clone(groups, c.byref(cloned)) == 0 and free(c.byref(groups)) == 0
    size = N()
    assert count(cloned, c.byref(size)) == 0 and size.value == 2
    snapshots = []
    for index in range(2):
        positions, sample, local = GroupInfo(), P(), P()
        assert info(cloned, index, c.byref(positions)) == 0
        assert [positions.first_state, positions.first_frame] == [index * 3, index * 6]
        assert observation(cloned, index, c.byref(sample)) == 0
        assert states(cloned, index, c.byref(local)) == 0
        snapshots.append((sample, local))
    assert free(c.byref(cloned)) == 0
    # Independent original rawfloat frame slice, not Rust-generated expected bytes.
    width = len(raw) // 12
    for index, (sample, local) in enumerate(snapshots):
        expected_raw = owned(raw[index * 6 * width:(index + 1) * 6 * width])
        expected_sample = P()
        assert observation_read(expected_raw, model, 6, c.byref(expected_sample)) == 0
        assert serialized(observation_write, sample) == serialized(observation_write, expected_sample)
        expected_states = copy.deepcopy(value[index * 3:(index + 1) * 3])
        for state in expected_states:
            state['time'] -= index * 6
        if enhanced and index == 0:
            expected_states[0]['jmp'] = [{'d': 1, 'p': 0.9}]
        assert json.loads(serialized(states_write, local)) == expected_states
        for owner in [sample, expected_sample]:
            assert observation_release(c.byref(owner)) == 0
        assert release(c.byref(expected_raw)) == 0 and states_release(c.byref(local)) == 0
    assert json.loads(serialized(states_write, state_owner)) == value
    assert states_release(c.byref(state_owner)) == 0

state_owner = read_states(original)
inputs = (GroupInput * 2)(*[GroupInput(N(-1).value, N(-2).value, source, state_owner)] * 2)
groups = P()
assert create(inputs, 2, c.byref(groups)) == 0
positions = GroupInfo()
assert info(groups, 1, c.byref(positions)) == 0
assert [positions.first_state, positions.first_frame] == [N(-1).value, N(-2).value]
assert info(groups, 2, c.byref(positions)) == 2 and positions.first_state == N(-1).value
retained = P(groups.value)
inputs[1].states = None
assert create(inputs, 2, c.byref(retained)) == 1 and retained.value == groups.value
assert create(None, N(-1).value, c.byref(retained)) == 2 and retained.value == groups.value
for bad in [[], [dict(state, time=6) if i >= 3 else state for i, state in enumerate(original)]]:
    bad_owner = read_states(bad)
    assert isolate(model, source, bad_owner, c.byref(retained)) == 3 and retained.value == groups.value
    assert states_release(c.byref(bad_owner)) == 0
sample, local = P(), P()
assert observation(groups, 0, c.byref(sample)) == 0 and states(groups, 0, c.byref(local)) == 0
expected_observation = serialized(observation_write, source)
assert free(c.byref(groups)) == 0 and observation_release(c.byref(source)) == 0
assert states_release(c.byref(state_owner)) == 0 and model_release(c.byref(model)) == 0
assert serialized(observation_write, sample) == expected_observation
assert json.loads(serialized(states_write, local)) == original
assert observation_release(c.byref(sample)) == 0 and states_release(c.byref(local)) == 0
assert create(None, 0, c.byref(groups)) == 0
size = N(99)
assert count(groups, c.byref(size)) == 0 and size.value == 0
assert clone(groups, None) == 1 and free(None) == 1 and free(c.byref(groups)) == 0
assert symbols <= called, symbols - called
print('SHIRO isolation ctypes: all8 exports, complete samples/states/positions and ownership passed')
