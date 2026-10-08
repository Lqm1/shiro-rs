"""Actual ctypes complete untying result and original C model/document/summary."""
import ctypes as c
import json
from pathlib import Path
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
symbols = {'untied_model_create', 'untie', 'untied_model_get_model', 'untied_model_get_document',
           'untied_model_length', 'untied_model_get_assignment', 'untied_model_summary_bytes',
           'untied_model_clone', 'untied_model_release'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO untying ctypes: all9 symbols absent without c-api')
    sys.exit(0)

class Assignment(c.Structure):
    _fields_ = [('state', N), ('file', N), ('segment', N)]

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
release = function('bytes_release', [c.POINTER(P)])
model_read = function('model_read_bytes', [P, N, c.POINTER(P)])
model_write = function('model_write_bytes', [P, U, c.POINTER(P)])
model_release = function('model_release', [c.POINTER(P)])
create = function('untied_model_create', [P, P, c.POINTER(Assignment), N, c.POINTER(P)])
untie = function('untie', [P, P, c.POINTER(P)])
get_model = function('untied_model_get_model', [P, c.POINTER(P)])
get_document = function('untied_model_get_document', [P, c.POINTER(P)])
count = function('untied_model_length', [P, c.POINTER(N)])
assignment = function('untied_model_get_assignment', [P, N, c.POINTER(Assignment)])
summary = function('untied_model_summary_bytes', [P, c.POINTER(P)])
clone = function('untied_model_clone', [P, c.POINTER(P)])
free = function('untied_model_release', [c.POINTER(P)])

def owned(data):
    output = P()
    assert create_bytes((B * len(data)).from_buffer_copy(data), len(data), c.byref(output)) == 0
    return output

def copied(owner):
    size = N()
    assert length(owner, c.byref(size)) == 0
    data = (B * size.value)()
    assert copy(owner, 0, data, size.value) == 0
    return bytes(data)

root = Path(__file__).parent / 'fixtures'
initial_wire = (root / 'init-c-aligned.hsmm').read_bytes()
encoded, model = owned(initial_wire), P()
assert model_read(encoded, 16 * 1024 * 1024, c.byref(model)) == 0 and release(c.byref(encoded)) == 0
document = (root / 'align-c-isolated.json').read_bytes()
encoded, result, cloned = owned(document), P(), P()
assert untie(model, encoded, c.byref(result)) == 0 and copied(encoded) == document
assert clone(result, c.byref(cloned)) == 0 and free(c.byref(result)) == 0
size = N()
assert count(cloned, c.byref(size)) == 0 and size.value == 6
for index in range(6):
    actual = Assignment()
    assert assignment(cloned, index, c.byref(actual)) == 0
    assert [actual.state, actual.file, actual.segment] == [index, 0, index]
snapshot, doc, report = P(), P(), P()
assert get_model(cloned, c.byref(snapshot)) == 0 and get_document(cloned, c.byref(doc)) == 0
assert summary(cloned, c.byref(report)) == 0 and free(c.byref(cloned)) == 0
assert json.loads(copied(doc)) == json.loads((root / 'untie-c.json').read_text())
assert copied(report) == (root / 'untie-c-summary.txt').read_bytes()
wire = P()
assert model_write(snapshot, 0, c.byref(wire)) == 0 and copied(wire) == (root / 'untie-c.hsmm').read_bytes()
assert release(c.byref(wire)) == 0 and model_release(c.byref(snapshot)) == 0
assert release(c.byref(doc)) == 0 and release(c.byref(report)) == 0

# Constructor accepts every native public assignment field without truncation.
entries = (Assignment * 1)(Assignment(N(-1).value, 0, 2))
assert create(model, encoded, entries, 1, c.byref(result)) == 0
actual = Assignment()
assert assignment(result, 0, c.byref(actual)) == 0 and actual.state == N(-1).value
assert summary(result, c.byref(report)) == 0
assert copied(report) == f'{N(-1).value} 0 2 a 2\n'.encode()
assert assignment(result, 1, c.byref(actual)) == 2 and actual.state == N(-1).value
retained = P(result.value)
assert create(model, encoded, None, N(-1).value, c.byref(retained)) == 2 and retained.value == result.value
invalid_document = json.loads(document)
invalid_document['file_list'][0]['states'][1]['dur'] = 999
bad = owned(json.dumps(invalid_document).encode())
assert untie(model, bad, c.byref(retained)) == 3 and retained.value == result.value
assert release(c.byref(bad)) == 0
entries = (Assignment * 2)(Assignment(0, 0, 0), Assignment(4, 99, 0))
assert create(model, encoded, entries, 2, c.byref(cloned)) == 0
retained_report = P(report.value)
assert summary(cloned, c.byref(retained_report)) == 3 and retained_report.value == report.value
assert free(c.byref(cloned)) == 0
assert create(model, encoded, None, 0, c.byref(cloned)) == 0
empty = P()
assert summary(cloned, c.byref(empty)) == 0 and copied(empty) == b''
assert release(c.byref(empty)) == 0 and free(c.byref(cloned)) == 0
assert free(None) == 1 and clone(result, None) == 1 and count(result, None) == 1
assert free(c.byref(result)) == 0 and free(c.byref(result)) == 0
assert release(c.byref(report)) == 0 and release(c.byref(encoded)) == 0
assert model_write(model, 0, c.byref(wire)) == 0 and copied(wire) == initial_wire
assert release(c.byref(wire)) == 0 and model_release(c.byref(model)) == 0
assert symbols <= called
print('SHIRO untying ctypes: all9 exports, original C model/JSON/summary, complete assignments and failure transactions passed')
