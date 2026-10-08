"""Actual ctypes complete definitions and ordered iteration reports."""
import ctypes as c
import json
from pathlib import Path
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, I, F, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_int32, c.c_float, c.c_uint8
symbols = {
    'definition_create', 'definition_get_info', 'definition_get_stream',
    'definition_get_constraint', 'definition_read_json', 'definition_write_json',
    'definition_build', 'definition_clone', 'definition_release',
    'iteration_report_create', 'iteration_reports_create', 'iteration_reports_length',
    'iteration_reports_get', 'iteration_reports_clone', 'iteration_reports_release',
}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO definitions/reports ctypes: all15 symbols absent without c-api')
    sys.exit(0)

class Stream(c.Structure):
    _fields_ = [('states', N), ('dimensions', N), ('mixtures', N), ('weight', F)]

class Constraint(c.Structure):
    _fields_ = [('index', N), ('has_minimum', U), ('minimum', I), ('has_maximum', U), ('maximum', I)]

class Info(c.Structure):
    _fields_ = [('duration_states', N), ('streams', N), ('duration_constraints', N)]

class Iteration(c.Structure):
    _fields_ = [('iteration', N), ('temperature', F), ('mean_log_likelihood', F)]

called = set()
def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke

out = c.POINTER(P)
create_bytes = function('bytes_create', [c.POINTER(B), N, out])
length_bytes = function('bytes_length', [P, c.POINTER(N)])
copy_bytes = function('bytes_copy', [P, N, c.POINTER(B), N])
release_bytes = function('bytes_release', [out])
create = function('definition_create', [N, c.POINTER(Stream), N, c.POINTER(Constraint), N, out])
get_info = function('definition_get_info', [P, c.POINTER(Info)])
get_stream = function('definition_get_stream', [P, N, c.POINTER(Stream)])
get_constraint = function('definition_get_constraint', [P, N, c.POINTER(Constraint)])
read = function('definition_read_json', [P, out])
write = function('definition_write_json', [P, out])
build = function('definition_build', [P, out])
clone = function('definition_clone', [P, out])
release = function('definition_release', [out])
model_write = function('model_write_bytes', [P, U, out])
model_release = function('model_release', [out])
array_create = function('array_f32_create', [c.POINTER(F), N, out])
array_length = function('array_f32_length', [P, c.POINTER(N)])
array_copy = function('array_f32_copy', [P, N, c.POINTER(F), N])
array_release = function('array_f32_release', [out])
report_create = function('iteration_report_create', [c.POINTER(Iteration), c.POINTER(P), N, out])
reports_create = function('iteration_reports_create', [c.POINTER(P), N, out])
reports_length = function('iteration_reports_length', [P, c.POINTER(N)])
reports_get = function('iteration_reports_get', [P, N, out])
reports_clone = function('iteration_reports_clone', [P, out])
reports_release = function('iteration_reports_release', [out])
report_info = function('iteration_report_info', [P, c.POINTER(Iteration)])
report_count = function('iteration_report_file_count', [P, c.POINTER(N)])
report_file = function('iteration_report_get_file', [P, N, out])
report_release = function('iteration_report_release', [out])

def owned(data):
    value = P()
    assert create_bytes((B * len(data)).from_buffer_copy(data), len(data), c.byref(value)) == 0
    return value

def copied(value):
    length = N()
    assert length_bytes(value, c.byref(length)) == 0
    data = (B * length.value)()
    assert copy_bytes(value, 0, data, length.value) == 0
    return bytes(data)

def bits(value, field):
    return U.from_address(c.addressof(value) + getattr(type(value), field).offset).value

root = Path(__file__).parent / 'fixtures'
for name, original in [('modeldef.json', 'empty-c.hsmm'),
                       ('utterances-c-definition.json', 'utterances-c-uninit.hsmm'),
                       ('init-definition.json', None)]:
    native = json.loads((root / name).read_bytes())
    source, definition, snapshot = owned((root / name).read_bytes()), P(), P()
    assert read(source, c.byref(definition)) == 0 and release_bytes(c.byref(source)) == 0
    assert clone(definition, c.byref(snapshot)) == 0 and release(c.byref(definition)) == 0
    info = Info()
    assert get_info(snapshot, c.byref(info)) == 0
    assert (info.duration_states, info.streams, info.duration_constraints) == (
        native['ndurstate'], len(native['streamdef']), len(native.get('dur_attr', [])))
    for index, expected in enumerate(native['streamdef']):
        stream = Stream()
        assert get_stream(snapshot, index, c.byref(stream)) == 0
        assert (stream.states, stream.dimensions, stream.mixtures, stream.weight) == (
            expected['nstate'], expected['ndim'], expected.get('nmix', 1), expected.get('weight', 1))
    for index, expected in enumerate(native.get('dur_attr', [])):
        constraint = Constraint()
        assert get_constraint(snapshot, index, c.byref(constraint)) == 0
        assert (constraint.index, constraint.has_minimum, constraint.minimum, constraint.has_maximum, constraint.maximum) == (
            expected['index'], int('floor' in expected), expected.get('floor', 0), int('ceil' in expected), expected.get('ceil', 0))
    encoded, reread, model, second, wire, other = P(), P(), P(), P(), P(), P()
    assert write(snapshot, c.byref(encoded)) == 0 and read(encoded, c.byref(reread)) == 0
    assert build(snapshot, c.byref(model)) == 0 and build(reread, c.byref(second)) == 0
    assert model_write(model, 0, c.byref(wire)) == 0 and model_write(second, 0, c.byref(other)) == 0
    assert copied(wire) == copied(other)
    if original:
        assert copied(wire) == (root / original).read_bytes()
    for value in [encoded, wire, other]:
        assert release_bytes(c.byref(value)) == 0
    for value in [model, second]:
        assert model_release(c.byref(value)) == 0
    for value in [snapshot, reread]:
        assert release(c.byref(value)) == 0

maximum = N(-1).value
patterns = [0x80000000, 0x7fc12345, 0x7f800000, 0xff800000, 1]
streams = (Stream * len(patterns))()
for value, pattern in zip(streams, patterns):
    value.states, value.dimensions, value.mixtures = maximum, 0, maximum - 1
    U.from_address(c.addressof(value) + Stream.weight.offset).value = pattern
constraints = (Constraint * 3)(Constraint(maximum, 0, 99, 1, 0),
    Constraint(0, 1, -2147483648, 0, 99), Constraint(1, 1, 0, 1, 2147483647))
definition, snapshot = P(), P()
assert create(maximum, streams, len(streams), constraints, len(constraints), c.byref(definition)) == 0
assert clone(definition, c.byref(snapshot)) == 0 and release(c.byref(definition)) == 0
for index, pattern in enumerate(patterns):
    actual = Stream()
    assert get_stream(snapshot, index, c.byref(actual)) == 0
    assert (actual.states, actual.dimensions, actual.mixtures, bits(actual, 'weight')) == (maximum, 0, maximum - 1, pattern)
for index, expected in enumerate(constraints):
    actual = Constraint()
    assert get_constraint(snapshot, index, c.byref(actual)) == 0
    assert (actual.index, actual.has_minimum, actual.minimum, actual.has_maximum, actual.maximum) == (
        expected.index, expected.has_minimum, expected.minimum if expected.has_minimum else 0,
        expected.has_maximum, expected.maximum if expected.has_maximum else 0)
sentinel = owned(b'retained')
retained = P(sentinel.value)
assert write(snapshot, c.byref(retained)) == 3 and retained.value == sentinel.value
assert copied(sentinel) == b'retained'
invalid = Constraint(0, 2, 0, 0, 0)
retained = P(snapshot.value)
assert create(0, None, 0, c.byref(invalid), 1, c.byref(retained)) == 2 and retained.value == snapshot.value
assert create(0, None, 1, None, 0, c.byref(retained)) == 1 and retained.value == snapshot.value
model = P()
assert build(snapshot, c.byref(model)) == 3 and model.value is None
assert release_bytes(c.byref(sentinel)) == 0 and release(c.byref(snapshot)) == 0

raw = (U * len(patterns))(*patterns)
row, empty = P(), P()
assert array_create(c.cast(raw, c.POINTER(F)), len(patterns), c.byref(row)) == 0
assert array_create(None, 0, c.byref(empty)) == 0
info = Iteration(maximum, 0, 0)
U.from_address(c.addressof(info) + Iteration.temperature.offset).value = 0x7fc54321
U.from_address(c.addressof(info) + Iteration.mean_log_likelihood.offset).value = 0xff800000
report, blank = P(), P()
assert report_create(c.byref(info), (P * 3)(row.value, empty.value, row.value), 3, c.byref(report)) == 0
assert report_create(c.byref(info), None, 0, c.byref(blank)) == 0
assert array_release(c.byref(row)) == 0 and array_release(c.byref(empty)) == 0
collection, snapshot = P(), P()
assert reports_create((P * 3)(report.value, blank.value, report.value), 3, c.byref(collection)) == 0
assert report_release(c.byref(report)) == 0 and report_release(c.byref(blank)) == 0
assert reports_clone(collection, c.byref(snapshot)) == 0 and reports_release(c.byref(collection)) == 0
length = N()
assert reports_length(snapshot, c.byref(length)) == 0 and length.value == 3
saved = []
for index in range(length.value):
    report = P()
    assert reports_get(snapshot, index, c.byref(report)) == 0
    saved.append(report)
retained = P(saved[0].value)
assert reports_get(snapshot, 3, c.byref(retained)) == 2 and retained.value == saved[0].value
assert reports_release(c.byref(snapshot)) == 0
for index, report in enumerate(saved):
    actual = Iteration()
    assert report_info(report, c.byref(actual)) == 0
    assert (actual.iteration, bits(actual, 'temperature'), bits(actual, 'mean_log_likelihood')) == (maximum, 0x7fc54321, 0xff800000)
    count = N()
    assert report_count(report, c.byref(count)) == 0 and count.value == (0 if index == 1 else 3)
    for row_index in range(count.value):
        row = P()
        assert report_file(report, row_index, c.byref(row)) == 0
        size = N()
        assert array_length(row, c.byref(size)) == 0 and size.value == (0 if row_index == 1 else len(patterns))
        output = (U * size.value)()
        assert array_copy(row, 0, c.cast(output, c.POINTER(F)), size.value) == 0
        assert list(output) == ([] if row_index == 1 else patterns)
        assert array_release(c.byref(row)) == 0
    assert report_release(c.byref(report)) == 0
assert reports_create(None, 0, c.byref(collection)) == 0
assert reports_length(collection, c.byref(length)) == 0 and length.value == 0
assert reports_release(c.byref(collection)) == 0 and reports_release(c.byref(collection)) == 0
assert symbols <= called, symbols - called
print('SHIRO definitions/reports ctypes: all15 exports, original models, full fields/bits and ownership passed')
