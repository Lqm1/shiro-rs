"""Actual ctypes direct native read/write/buffered callbacks and IO failures."""
import ctypes as c
from pathlib import Path
import struct
import sys

lib = c.CDLL(sys.argv[1])
P, N, U, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8
symbols = {'rawfloat_read_stream', 'rawfloat_write_stream', 'observation_read_stream',
           'labels_write_stream', 'untied_model_write_summary_stream',
           'index_read_stream', 'write_stream_flush'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO streams ctypes: all7 symbols absent without c-api')
    sys.exit(0)
called = set()

def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke

Read = c.CFUNCTYPE(U, P, c.POINTER(B), N, c.POINTER(N))
Write = c.CFUNCTYPE(U, P, c.POINTER(B), N, c.POINTER(N))
Flush = c.CFUNCTYPE(U, P)
Fill = c.CFUNCTYPE(U, P, c.POINTER(c.POINTER(B)), c.POINTER(N))
Consume = c.CFUNCTYPE(None, P, N)

class Reader(c.Structure):
    _fields_ = [('context', P), ('read', Read)]

class Writer(c.Structure):
    _fields_ = [('context', P), ('write', Write), ('flush', Flush)]

class Buffered(c.Structure):
    _fields_ = [('context', P), ('fill', Fill), ('consume', Consume)]

create_bytes = function('bytes_create', [c.POINTER(B), N, c.POINTER(P)])
byte_length = function('bytes_length', [P, c.POINTER(N)])
copy = function('bytes_copy', [P, N, c.POINTER(B), N])
release_bytes = function('bytes_release', [c.POINTER(P)])
read_raw = function('rawfloat_read_stream', [c.POINTER(Reader), N, c.POINTER(P)])
write_raw = function('rawfloat_write_stream', [P, c.POINTER(Writer)])
read_obs = function('observation_read_stream', [c.POINTER(Reader), c.POINTER(N), N, N, c.POINTER(P)])
write_labels = function('labels_write_stream', [P, c.POINTER(Writer)])
write_summary = function('untied_model_write_summary_stream', [P, c.POINTER(Writer)])
read_index = function('index_read_stream', [c.POINTER(Buffered), P, P, P, c.POINTER(P)])
flush_stream = function('write_stream_flush', [c.POINTER(Writer)])
array_wire = function('rawfloat_write_bytes', [P, c.POINTER(P)])
release_array = function('array_f32_release', [c.POINTER(P)])
obs_wire = function('observation_write_bytes', [P, c.POINTER(P)])
byte_obs = function('observation_read_rawfloat', [P, c.POINTER(N), N, N, c.POINTER(P)])
release_obs = function('observation_release', [c.POINTER(P)])
utf8 = function('path_from_utf8', [P, c.POINTER(P)])
release_path = function('path_release', [c.POINTER(P)])
strings = function('strings_create', [c.POINTER(P), N, c.POINTER(P)])
release_strings = function('strings_release', [c.POINTER(P)])
entry_length = function('index_entries_length', [P, c.POINTER(N)])
entry_stem = function('index_entries_get_stem', [P, N, c.POINTER(P)])
entry_phones = function('index_entries_get_phonemes', [P, N, c.POINTER(P)])
string_length = function('strings_length', [P, c.POINTER(N)])
get_string = function('strings_get', [P, N, c.POINTER(P)])
native_bytes = function('path_native_bytes', [P, c.POINTER(P)])
native_kind = function('path_native_encoding', [])()
release_entries = function('index_entries_release', [c.POINTER(P)])
parse_labels = function('labels_parse', [P, c.POINTER(P)])
labels_bytes = function('labels_write_bytes', [P, c.POINTER(P)])
release_labels = function('labels_release', [c.POINTER(P)])
read_model = function('model_read_bytes', [P, N, c.POINTER(P)])
release_model = function('model_release', [c.POINTER(P)])
untie = function('untie', [P, P, c.POINTER(P)])
release_untied = function('untied_model_release', [c.POINTER(P)])

def owned(data):
    value = P()
    assert create_bytes((B * len(data)).from_buffer_copy(data), len(data), c.byref(value)) == 0
    return value

def copied(owner):
    count = N()
    assert byte_length(owner, c.byref(count)) == 0
    data = (B * count.value)()
    assert copy(owner, 0, data, count.value) == 0
    return bytes(data)

class Channel:
    def __init__(self, data=b'', chunk=1, fail=None):
        self.data, self.chunk, self.fail = data, chunk, fail
        self.position, self.calls, self.flushes = 0, 0, 0
        self.output, self.trace, self.errors = bytearray(), [], []
        self.storage = (B * len(data)).from_buffer_copy(data)
        self.reader = Reader(None, Read(self.read))
        self.writer = Writer(None, Write(self.write), Flush(self.flush))
        self.buffered = Buffered(None, Fill(self.fill), Consume(self.consume))

    def step(self, kind, count):
        self.trace.append((kind, count))
        self.calls += 1
        if self.calls == 1:
            return 1
        return 2 if self.fail is not None and self.position >= self.fail else 0

    def read(self, context, buffer, capacity, count):
        result = self.step('r', capacity)
        if result:
            return result
        n = min(capacity, self.chunk, len(self.data) - self.position)
        c.memmove(buffer, self.data[self.position:self.position + n], n)
        self.position += n
        count[0] = n
        return 0

    def write(self, context, buffer, capacity, count):
        result = self.step('w', capacity)
        if result:
            return result
        n = min(capacity, self.chunk)
        self.output.extend(c.string_at(buffer, n))
        self.position += n
        count[0] = n
        return 0

    def flush(self, context):
        self.flushes += 1
        return self.step('s', 0)

    def fill(self, context, buffer, count):
        result = self.step('f', 0)
        if result:
            return result
        n = min(self.chunk, len(self.data) - self.position)
        buffer[0] = c.cast(c.byref(self.storage, self.position), c.POINTER(B))
        count[0] = n
        return 0

    def consume(self, context, count):
        if count > min(self.chunk, len(self.data) - self.position):
            self.errors.append('consume exceeds borrowed buffer')
            return
        self.trace.append(('c', count))
        self.position += count

wire = struct.pack('<6I', 0, 0x80000000, 1, 0x7fc12345, 0x7f800000, 0xff800000)
for chunk in [1, 7, 16384]:
    channel, array = Channel(wire, chunk), P()
    assert read_raw(c.byref(channel.reader), 6, c.byref(array)) == 0
    snapshot = P()
    assert array_wire(array, c.byref(snapshot)) == 0 and copied(snapshot) == wire
    assert release_bytes(c.byref(snapshot)) == 0
    output = Channel(chunk=chunk)
    assert write_raw(array, c.byref(output.writer)) == 0 and bytes(output.output) == wire and output.flushes == 0
    assert flush_stream(c.byref(output.writer)) == 0 and output.flushes == 1
    for data, budget, fail in [(wire[:-1], 6, None), (wire, 1, None), (wire, 6, 5)]:
        failing = Channel(data, 1, fail)
        retained = P(array.value)
        assert read_raw(c.byref(failing.reader), budget, c.byref(retained)) == 3 and retained.value == array.value
    failing = Channel(chunk=1, fail=5)
    assert write_raw(array, c.byref(failing.writer)) == 3 and bytes(failing.output) == wire[:5]
    zero = Channel(chunk=0)
    assert write_raw(array, c.byref(zero.writer)) == 3 and not zero.output
    assert release_array(c.byref(array)) == 0

fixtures = Path(__file__).parent / 'fixtures'
raw = (fixtures / 'init-input.bin').read_bytes()
dimensions = (N * 2)(2, 1)
source, reference = owned(raw), P()
assert byte_obs(source, dimensions, 2, 1000, c.byref(reference)) == 0
reference_wire = P()
assert obs_wire(reference, c.byref(reference_wire)) == 0
for chunk in [1, 13, 16384]:
    channel, observation, snapshot = Channel(raw, chunk), P(), P()
    assert read_obs(c.byref(channel.reader), dimensions, 2, 1000, c.byref(observation)) == 0
    assert obs_wire(observation, c.byref(snapshot)) == 0 and copied(snapshot) == copied(reference_wire)
    assert release_bytes(c.byref(snapshot)) == 0 and release_obs(c.byref(observation)) == 0
assert release_bytes(c.byref(source)) == 0 and release_bytes(c.byref(reference_wire)) == 0 and release_obs(c.byref(reference)) == 0

source, directory, padding = owned(b'data'), P(), P()
assert utf8(source, c.byref(directory)) == 0 and release_bytes(c.byref(source)) == 0
assert strings(None, 0, c.byref(padding)) == 0
for chunk in [1, 7, 999]:
    data = (fixtures / 'index-original.txt').read_bytes()
    channel, entries, count = Channel(data, chunk), P(), N()
    assert read_index(c.byref(channel.buffered), directory, padding, padding, c.byref(entries)) == 0
    assert channel.position == len(data) and not channel.errors
    assert entry_length(entries, c.byref(count)) == 0 and count.value == 3
    for i, (name, expected) in enumerate([('clip one', ['aa', 'bb']), ('sub/clip.two', ['cc', '', 'dd']), ('silent', [])]):
        stem, phones, snapshot = P(), P(), P()
        assert entry_stem(entries, i, c.byref(stem)) == 0 and native_bytes(stem, c.byref(snapshot)) == 0
        text = 'data' + ('\\' if native_kind == 2 else '/') + name
        assert copied(snapshot) == text.encode('utf-16le' if native_kind == 2 else 'utf-8')
        assert release_bytes(c.byref(snapshot)) == 0 and release_path(c.byref(stem)) == 0
        assert entry_phones(entries, i, c.byref(phones)) == 0 and string_length(phones, c.byref(count)) == 0 and count.value == len(expected)
        for j, phone in enumerate(expected):
            assert get_string(phones, j, c.byref(snapshot)) == 0 and copied(snapshot) == phone.encode()
            assert release_bytes(c.byref(snapshot)) == 0
        assert release_strings(c.byref(phones)) == 0
    for data in [b'\n\r\nbad\n', b'clip,\xff\n']:
        channel, retained = Channel(data, chunk), P(entries.value)
        assert read_index(c.byref(channel.buffered), directory, padding, padding, c.byref(retained)) == 3 and retained.value == entries.value
        assert not channel.errors
    assert release_entries(c.byref(entries)) == 0
assert release_path(c.byref(directory)) == 0 and release_strings(c.byref(padding)) == 0

source, labels, label_wire = owned((fixtures / 'labels-input.txt').read_bytes()), P(), P()
assert parse_labels(source, c.byref(labels)) == 0 and labels_bytes(labels, c.byref(label_wire)) == 0
assert release_bytes(c.byref(source)) == 0
source, model = owned((fixtures / 'init-c-aligned.hsmm').read_bytes()), P()
assert read_model(source, 16 * 1024 * 1024, c.byref(model)) == 0 and release_bytes(c.byref(source)) == 0
source, untied = owned((fixtures / 'align-c-isolated.json').read_bytes()), P()
assert untie(model, source, c.byref(untied)) == 0 and release_bytes(c.byref(source)) == 0
for operation, owner, expected in [(write_labels, labels, copied(label_wire)), (write_summary, untied, (fixtures / 'untie-c-summary.txt').read_bytes())]:
    for chunk in [1, 13, 16384]:
        channel = Channel(chunk=chunk)
        assert operation(owner, c.byref(channel.writer)) == 0 and bytes(channel.output) == expected and channel.flushes == 0
    channel = Channel(chunk=1, fail=7)
    assert operation(owner, c.byref(channel.writer)) == 3 and bytes(channel.output) == expected[:7] and channel.flushes == 0
assert release_bytes(c.byref(label_wire)) == 0 and release_labels(c.byref(labels)) == 0
assert release_untied(c.byref(untied)) == 0 and release_model(c.byref(model)) == 0
untouched, result = Channel(wire), P()
assert read_raw(c.byref(untouched.reader), 6, None) == 1 and untouched.calls == 0
assert read_raw(None, 6, c.byref(result)) == 1
assert flush_stream(None) == 1
assert symbols <= called, symbols - called
print('SHIRO streams ctypes: all7 exports, direct partial IO/full fields/original fixtures/callbacks/failures passed')
