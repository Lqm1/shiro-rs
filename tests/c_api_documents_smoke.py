"""Actual ctypes complete typed states, files and segmentation documents."""
import ctypes as c
import json
from pathlib import Path
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B, D = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8, c.c_double
symbols = {'states_create', 'states_length', 'states_get_info', 'states_get_outputs', 'states_get_json_field',
    'segmented_file_create', 'segmented_file_get_filename', 'segmented_file_get_states', 'segmented_file_get_attributes',
    'segmented_file_clone', 'segmented_file_release', 'document_create', 'document_length', 'document_get_file',
    'document_get_attributes', 'document_read_json', 'document_write_json', 'document_clone', 'document_release'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO documents ctypes: all19 symbols absent without c-api')
    sys.exit(0)

class StateInput(c.Structure):
    _fields_ = [('time', D), ('has_duration', U), ('duration', N), ('outputs', P), ('jumps', P), ('metadata', P), ('attributes', P)]

class StateInfo(c.Structure):
    _fields_ = [('time', D), ('has_duration', U), ('duration', N), ('has_outputs', U), ('has_jumps', U)]

called = set()
def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke

out = c.POINTER(P)
bytes_create = function('bytes_create', [c.POINTER(B), N, out])
bytes_length = function('bytes_length', [P, c.POINTER(N)])
bytes_copy = function('bytes_copy', [P, N, c.POINTER(B), N])
bytes_release = function('bytes_release', [out])
array_create = function('array_usize_create', [c.POINTER(N), N, out])
array_length = function('array_usize_length', [P, c.POINTER(N)])
array_copy = function('array_usize_copy', [P, N, c.POINTER(N), N])
array_release = function('array_usize_release', [out])
states_create = function('states_create', [c.POINTER(StateInput), N, out])
states_length = function('states_length', [P, c.POINTER(N)])
states_info = function('states_get_info', [P, N, c.POINTER(StateInfo)])
states_outputs = function('states_get_outputs', [P, N, out])
states_field = function('states_get_json_field', [P, N, U, out])
states_write = function('states_write_json', [P, out])
states_release = function('states_release', [out])
file_create = function('segmented_file_create', [P, P, P, out])
file_name = function('segmented_file_get_filename', [P, out])
file_states = function('segmented_file_get_states', [P, out])
file_attributes = function('segmented_file_get_attributes', [P, out])
file_clone = function('segmented_file_clone', [P, out])
file_release = function('segmented_file_release', [out])
document_create = function('document_create', [c.POINTER(P), N, P, out])
document_length = function('document_length', [P, c.POINTER(N)])
document_file = function('document_get_file', [P, N, out])
document_attributes = function('document_get_attributes', [P, out])
document_read = function('document_read_json', [P, out])
document_write = function('document_write_json', [P, out])
document_clone = function('document_clone', [P, out])
document_release = function('document_release', [out])

def owned(data):
    owner = P()
    assert bytes_create((B * len(data)).from_buffer_copy(data), len(data), c.byref(owner)) == 0
    return owner

def encoded(value):
    return owned(json.dumps(value).encode())

def copied(owner):
    count = N()
    assert bytes_length(owner, c.byref(count)) == 0
    output = (B * count.value)()
    assert bytes_copy(owner, 0, output, count.value) == 0
    return bytes(output)

def snapshot_json(fn, owner):
    output = P()
    assert fn(owner, c.byref(output)) == 0
    result = json.loads(copied(output))
    assert bytes_release(c.byref(output)) == 0
    return result

def typed_states(values):
    inputs = (StateInput * len(values))()
    byte_owners, arrays = [], []
    for descriptor, value in zip(inputs, values):
        descriptor.time = value.get('time', 0)
        descriptor.has_duration = int(value.get('dur') is not None)
        descriptor.duration = value.get('dur') or 0
        if value.get('out') is not None:
            data = value['out']
            owner = P()
            assert array_create((N * len(data))(*data), len(data), c.byref(owner)) == 0
            arrays.append(owner)
            descriptor.outputs = owner.value
        if value.get('jmp') is not None:
            owner = encoded(value['jmp'])
            byte_owners.append(owner)
            descriptor.jumps = owner.value
        metadata = encoded(value.get('ext', []))
        attributes = encoded({key: item for key, item in value.items() if key not in {'time', 'dur', 'out', 'jmp', 'ext'}})
        byte_owners.extend([metadata, attributes])
        descriptor.metadata, descriptor.attributes = metadata.value, attributes.value
    output = P()
    assert states_create(inputs, len(inputs), c.byref(output)) == 0
    for owner in byte_owners:
        assert bytes_release(c.byref(owner)) == 0
    for owner in arrays:
        assert array_release(c.byref(owner)) == 0
    return output

root = Path(__file__).parent / 'fixtures'
for name in ['utterances-c-initial.json', 'utterances-c-aligned.json', 'align-c-isolated.json', 'init-segmentation.json']:
    data = (root / name).read_bytes()
    native = json.loads(data)
    for file in native['file_list']:
        for state in file['states']:
            state.setdefault('time', 0)
            state.setdefault('ext', [])
            for key in ['dur', 'out', 'jmp']:
                if state.get(key) is None:
                    state.pop(key, None)
    source, parsed = owned(data), P()
    assert document_read(source, c.byref(parsed)) == 0 and bytes_release(c.byref(source)) == 0
    count = N()
    assert document_length(parsed, c.byref(count)) == 0 and count.value == len(native['file_list'])
    assert snapshot_json(document_attributes, parsed) == {key: value for key, value in native.items() if key != 'file_list'}
    files = []
    for index, expected in enumerate(native['file_list']):
        file, snapshot = P(), P()
        assert document_file(parsed, index, c.byref(file)) == 0
        assert file_clone(file, c.byref(snapshot)) == 0 and file_release(c.byref(file)) == 0
        name, attrs, actual = P(), P(), P()
        assert file_name(snapshot, c.byref(name)) == 0 and copied(name) == expected['filename'].encode()
        assert file_attributes(snapshot, c.byref(attrs)) == 0
        assert json.loads(copied(attrs)) == {key: value for key, value in expected.items() if key not in {'filename', 'states'}}
        assert file_states(snapshot, c.byref(actual)) == 0 and file_release(c.byref(snapshot)) == 0
        assert snapshot_json(states_write, actual) == expected['states']
        typed = typed_states(expected['states'])
        built = P()
        assert file_create(name, typed, attrs, c.byref(built)) == 0
        files.append(built)
        assert states_release(c.byref(actual)) == 0 and states_release(c.byref(typed)) == 0
        assert bytes_release(c.byref(name)) == 0 and bytes_release(c.byref(attrs)) == 0
    assert document_release(c.byref(parsed)) == 0
    attributes = encoded({key: value for key, value in native.items() if key != 'file_list'})
    rebuilt, snapshot = P(), P()
    assert document_create((P * len(files))(*(file.value for file in files)), len(files), attributes, c.byref(rebuilt)) == 0
    for file in files:
        assert file_release(c.byref(file)) == 0
    assert bytes_release(c.byref(attributes)) == 0
    assert document_clone(rebuilt, c.byref(snapshot)) == 0 and document_release(c.byref(rebuilt)) == 0
    assert snapshot_json(document_write, snapshot) == native
    assert document_release(c.byref(snapshot)) == 0

maximum = N(-1).value
patterns = [0x8000000000000000, 0x7ff8123456789abc, 0x7ff0000000000000, 0xfff0000000000000, 1]
metadata = encoded(['phone', 7, {'nested': [None, True, '\0']}])
attributes = encoded({'time': 'shadow', 'dur': False, 'out': [], 'ext': 42, 'jmp': None})
jumps = encoded([])
outputs, empty = P(), P()
assert array_create((N * 3)(0, maximum, maximum - 1), 3, c.byref(outputs)) == 0
assert array_create(None, 0, c.byref(empty)) == 0
inputs = (StateInput * len(patterns))()
for index, (descriptor, bits) in enumerate(zip(inputs, patterns)):
    c.c_uint64.from_address(c.addressof(descriptor) + StateInput.time.offset).value = bits
    descriptor.has_duration, descriptor.duration = int(index != 0), (0 if index == 1 else maximum)
    descriptor.outputs = None if index == 0 else (empty.value if index == 1 else outputs.value)
    descriptor.jumps = None if index == 0 else jumps.value
    descriptor.metadata, descriptor.attributes = metadata.value, attributes.value
typed = P()
assert states_create(inputs, len(inputs), c.byref(typed)) == 0
name = owned(b'audio-\xf0\x9f\x8e\xb5\0file.wav')
file_attrs = encoded({'filename': 'shadow', 'states': 0, 'nested': [None, True]})
top_attrs = encoded({'file_list': 'shadow', 'nested': {'ok': True}})
file, document = P(), P()
assert file_create(name, typed, file_attrs, c.byref(file)) == 0
assert document_create((P * 2)(file.value, file.value), 2, top_attrs, c.byref(document)) == 0
assert file_release(c.byref(file)) == 0 and states_release(c.byref(typed)) == 0
for owner in [name, metadata, attributes, jumps]:
    assert bytes_release(c.byref(owner)) == 0
assert array_release(c.byref(outputs)) == 0 and array_release(c.byref(empty)) == 0
snapshot = P()
assert document_clone(document, c.byref(snapshot)) == 0 and document_release(c.byref(document)) == 0
count = N()
assert document_length(snapshot, c.byref(count)) == 0 and count.value == 2
assert snapshot_json(document_attributes, snapshot) == json.loads(copied(top_attrs))
saved = []
for index in range(2):
    file = P()
    assert document_file(snapshot, index, c.byref(file)) == 0
    saved.append(file)
retained = P(saved[0].value)
assert document_file(snapshot, 2, c.byref(retained)) == 2 and retained.value == saved[0].value
retained_bytes = P(file_attrs.value)
assert document_write(snapshot, c.byref(retained_bytes)) == 3 and retained_bytes.value == file_attrs.value
retained_document = P(snapshot.value)
assert document_create(None, 1, top_attrs, c.byref(retained_document)) == 1 and retained_document.value == snapshot.value
assert document_release(c.byref(snapshot)) == 0
for file in saved:
    name, typed = P(), P()
    assert file_name(file, c.byref(name)) == 0 and copied(name) == b'audio-\xf0\x9f\x8e\xb5\0file.wav'
    assert file_states(file, c.byref(typed)) == 0
    assert snapshot_json(file_attributes, file) == json.loads(copied(file_attrs))
    assert file_release(c.byref(file)) == 0
    assert bytes_release(c.byref(name)) == 0
    assert states_length(typed, c.byref(count)) == 0 and count.value == len(patterns)
    for index, bits in enumerate(patterns):
        info = StateInfo()
        assert states_info(typed, index, c.byref(info)) == 0
        assert c.c_uint64.from_address(c.addressof(info) + StateInfo.time.offset).value == bits
        assert (info.has_duration, info.duration, info.has_outputs, info.has_jumps) == (
            int(index != 0), 0 if index <= 1 else maximum, int(index != 0), int(index != 0))
        row = P()
        assert states_outputs(typed, index, c.byref(row)) == (2 if index == 0 else 0)
        if index != 0:
            length = N()
            assert array_length(row, c.byref(length)) == 0 and length.value == (0 if index == 1 else 3)
            output = (N * length.value)()
            assert array_copy(row, 0, output, length.value) == 0
            assert list(output) == ([] if index == 1 else [0, maximum, maximum - 1])
            assert array_release(c.byref(row)) == 0
        for field in range(3):
            output = P()
            status = states_field(typed, index, field, c.byref(output))
            if index == 0 and field == 0:
                assert status == 2 and output.value is None
                continue
            assert status == 0
            assert json.loads(copied(output)) == [[], ['phone', 7, {'nested': [None, True, '\0']}],
                {'time': 'shadow', 'dur': False, 'out': [], 'ext': 42, 'jmp': None}][field]
            assert bytes_release(c.byref(output)) == 0
    retained_bytes = P(file_attrs.value)
    assert states_write(typed, c.byref(retained_bytes)) == 3 and retained_bytes.value == file_attrs.value
    assert states_release(c.byref(typed)) == 0
assert document_create(None, 0, top_attrs, c.byref(document)) == 0
assert document_length(document, c.byref(count)) == 0 and count.value == 0
assert document_release(c.byref(document)) == 0 and document_release(c.byref(document)) == 0
assert bytes_release(c.byref(file_attrs)) == 0 and bytes_release(c.byref(top_attrs)) == 0
assert symbols <= called, symbols - called
print('SHIRO documents ctypes: all19 exports, original full typed reconstruction/all fields/bits/ownership passed')
