"""Actual ctypes training, original C wire models and complete progress reports."""
import ctypes as c
import json
import math
from pathlib import Path
import sys
import tempfile

lib = c.CDLL(sys.argv[1])
P, N, U, F, B = c.c_void_p, c.c_size_t, c.c_uint32, c.c_float, c.c_uint8
symbols = {
    'training_files_create', 'training_files_read_document', 'training_files_length',
    'training_files_get_dataset', 'training_files_clone', 'training_files_release',
    'training_options_default', 'train', 'train_with_progress',
    'training_result_length', 'training_result_get_model', 'training_result_get_report',
    'training_result_clone', 'training_result_release', 'training_result_likelihood_csv_bytes', 'training_result_write_likelihood_csv_stream', 'iteration_report_info',
    'iteration_report_file_count', 'iteration_report_get_file',
    'iteration_report_clone', 'iteration_report_release', 'iteration_report_create',
    'iteration_report_replace', 'training_result_create', 'training_result_replace',
}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO training ctypes: all25 symbols absent without c-api')
    sys.exit(0)

class Options(c.Structure):
    _fields_ = [
        ('iterations', N), ('duration_mode', U), ('hsmm_temperature', F),
        ('duration_weight', F), ('state_radius', F), ('duration_extra', N),
        ('duration_extra_factor', F), ('geometric_temperature', F),
        ('pruning_slope', F), ('termination_threshold', F),
        ('deterministic_annealing', U), ('mean_frame_likelihood', U), ('workers', N),
    ]

class Info(c.Structure):
    _fields_ = [('iteration', N), ('temperature', F), ('mean_log_likelihood', F)]

Callback = c.CFUNCTYPE(None, P, P)
Write = c.CFUNCTYPE(U, P, c.POINTER(B), N, c.POINTER(N))
Flush = c.CFUNCTYPE(U, P)
class Writer(c.Structure):
    _fields_ = [('context', P), ('write', Write), ('flush', Flush)]

class CsvChannel:
    def __init__(self, chunk, fail=None):
        self.chunk, self.fail = chunk, fail
        self.output, self.calls, self.flushes = bytearray(), 0, 0
        self.stream = Writer(None, Write(self.write), Flush(self.flush))
    def write(self, context, buffer, capacity, count):
        self.calls += 1
        if self.calls == 1:
            return 1
        if self.fail is not None and len(self.output) >= self.fail:
            return 2
        size = min(capacity, self.chunk)
        self.output.extend(c.string_at(buffer, size))
        count[0] = size
        return 0
    def flush(self, context):
        self.flushes += 1
        return 0
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
model_read = function('model_read_bytes', [P, N, c.POINTER(P)])
model_write = function('model_write_bytes', [P, U, c.POINTER(P)])
model_release = function('model_release', [c.POINTER(P)])
files_create = function('training_files_create', [c.POINTER(P), N, c.POINTER(P)])
files_read = function('training_files_read_document', [P, P, N, U, c.POINTER(P)])
files_length = function('training_files_length', [P, c.POINTER(N)])
files_dataset = function('training_files_get_dataset', [P, N, c.POINTER(P)])
files_clone = function('training_files_clone', [P, c.POINTER(P)])
files_release = function('training_files_release', [c.POINTER(P)])
dataset_length = function('dataset_length', [P, c.POINTER(N)])
dataset_release = function('dataset_release', [c.POINTER(P)])
defaults = function('training_options_default', [c.POINTER(Options)])
train = function('train', [P, P, c.POINTER(Options), c.POINTER(P)])
train_progress = function('train_with_progress', [P, P, c.POINTER(Options), Callback, P, c.POINTER(P)])
result_length = function('training_result_length', [P, c.POINTER(N)])
result_model = function('training_result_get_model', [P, c.POINTER(P)])
result_report = function('training_result_get_report', [P, N, c.POINTER(P)])
result_clone = function('training_result_clone', [P, c.POINTER(P)])
result_csv = function('training_result_likelihood_csv_bytes', [P, c.POINTER(P)])
result_csv_stream = function('training_result_write_likelihood_csv_stream', [P, c.POINTER(Writer)])
result_release = function('training_result_release', [c.POINTER(P)])
result_create = function('training_result_create', [P, c.POINTER(P), N, c.POINTER(P)])
result_replace = function('training_result_replace', [P, P, c.POINTER(P), N])
report_create = function('iteration_report_create', [c.POINTER(Info), c.POINTER(P), N, c.POINTER(P)])
report_replace = function('iteration_report_replace', [P, c.POINTER(Info), c.POINTER(P), N])
report_info = function('iteration_report_info', [P, c.POINTER(Info)])
report_count = function('iteration_report_file_count', [P, c.POINTER(N)])
report_file = function('iteration_report_get_file', [P, N, c.POINTER(P)])
report_clone = function('iteration_report_clone', [P, c.POINTER(P)])
report_release = function('iteration_report_release', [c.POINTER(P)])
array_length = function('array_f32_length', [P, c.POINTER(N)])
array_copy = function('array_f32_copy', [P, N, c.POINTER(F), N])
array_release = function('array_f32_release', [c.POINTER(P)])
array_create = function('array_f32_create', [c.POINTER(F), N, c.POINTER(P)])

def owned(data):
    output = P()
    assert create((B * len(data)).from_buffer_copy(data), len(data), c.byref(output)) == 0
    return output

def copied(owner):
    count = N()
    assert length(owner, c.byref(count)) == 0
    data = (B * count.value)()
    assert copy(owner, 0, data, count.value) == 0
    return bytes(data)

def wire(model):
    data = P()
    assert model_write(model, 0, c.byref(data)) == 0
    output = copied(data)
    assert release_bytes(c.byref(data)) == 0
    return output

def report(owner):
    info, count = Info(), N()
    assert report_info(owner, c.byref(info)) == 0
    assert report_count(owner, c.byref(count)) == 0
    rows = []
    for index in range(count.value):
        row, size = P(), N()
        assert report_file(owner, index, c.byref(row)) == 0
        assert array_length(row, c.byref(size)) == 0
        values = (F * size.value)()
        assert array_copy(row, 0, values, size.value) == 0
        rows.append(list(values))
        assert array_release(c.byref(row)) == 0
    invalid = P()
    assert report_file(owner, count.value, c.byref(invalid)) == 2 and invalid.value is None
    return info.iteration, info.temperature, info.mean_log_likelihood, rows

events, callback_errors = [], []
@Callback
def progress(context, borrowed):
    # Catch Python exceptions here; never silently lose a callback failure.
    try:
        assert context == 73
        output = P()
        assert report_clone(borrowed, c.byref(output)) == 0
        events.append(output)
    except BaseException as error:
        callback_errors.append(error)

root = Path(__file__).parent / 'fixtures'
initial_wire = (root / 'init-c-aligned.hsmm').read_bytes()
encoded, model = owned(initial_wire), P()
assert model_read(encoded, 16 * 1024 * 1024, c.byref(model)) == 0
assert release_bytes(c.byref(encoded)) == 0
config = Options()
assert defaults(c.byref(config)) == 0
assert [getattr(config, name) for name, _ in Options._fields_] == [
    1, 0, 1.0, 1.0, 5.0, 30, 1.0, 1.0, F(0.3).value, 1.0, 0, 0, 1,
]
cases = [
    ('hsmm-one', 1, 0, 0, 0, 0, 'hsmm-one'),
    ('hsmm-two', 2, 0, 0, 0, 0, 'hsmm-two'),
    ('daem', 3, 0, 1, 0, 0, 'daem'),
    ('hmm', 2, 1, 0, 0, 0, 'hmm'),
    ('hsmm-one', 1, 0, 0, 1, 0, 'mean'),
    ('isolated', 2, 0, 0, 0, 1, 'isolated'),
    ('isolated-hmm', 2, 1, 0, 0, 1, 'isolated-hmm'),
    ('isolated-daem', 3, 0, 1, 0, 1, 'isolated-daem'),
    ('isolated-mean', 2, 0, 0, 1, 1, 'isolated-mean'),
]
with tempfile.TemporaryDirectory(prefix='shiro-c-api-training-') as directory:
    path = Path(directory) / 'input.f'
    path.write_bytes((root / 'init-input.bin').read_bytes())
    document = json.loads((root / 'align-c-isolated.json').read_text())
    document['file_list'][0]['filename'] = str(path)
    encoded = owned(json.dumps(document).encode())
    for fixture, iterations, mode, anneal, mean, isolated, likelihood in cases:
        files, cloned = P(), P()
        assert files_read(model, encoded, 12, isolated, c.byref(files)) == 0
        assert files_clone(files, c.byref(cloned)) == 0
        assert files_release(c.byref(files)) == 0
        count, snapshot = N(), P()
        assert files_length(cloned, c.byref(count)) == 0 and count.value == 1
        assert files_dataset(cloned, 0, c.byref(snapshot)) == 0
        assert dataset_length(snapshot, c.byref(count)) == 0 and count.value == (2 if isolated else 1)
        assert files_create((P * 1)(snapshot.value), 1, c.byref(files)) == 0
        assert dataset_release(c.byref(snapshot)) == 0
        assert files_release(c.byref(cloned)) == 0
        assert defaults(c.byref(config)) == 0
        config.iterations, config.duration_mode = iterations, mode
        config.deterministic_annealing, config.mean_frame_likelihood = anneal, mean
        config.termination_threshold = 0
        config.pruning_slope = 0.8
        result, cloned = P(), P()
        assert train_progress(model, files, c.byref(config), progress, P(73), c.byref(result)) == 0
        assert not callback_errors
        assert result_length(result, c.byref(count)) == 0 and count.value == iterations
        assert len(events) == iterations
        assert result_clone(result, c.byref(cloned)) == 0
        assert result_release(c.byref(result)) == 0
        trained = P()
        assert result_model(cloned, c.byref(trained)) == 0
        expected_wire = (root / ('rest-c-' + fixture + '.hsmm')).read_bytes()
        assert wire(trained) == expected_wire
        expected_rows = [[float(x) for x in line.split(',')]
                         for line in (root / ('rest-c-' + likelihood + '.likelihood')).read_text().splitlines()]
        csv = P()
        assert result_csv(cloned, c.byref(csv)) == 0
        encoded_csv = copied(csv)
        for chunk in [1, 13, 16384]:
            channel = CsvChannel(chunk)
            assert result_csv_stream(cloned, c.byref(channel.stream)) == 0
            assert bytes(channel.output) == encoded_csv and channel.flushes == 0
        channel = CsvChannel(1, fail=7)
        assert result_csv_stream(cloned, c.byref(channel.stream)) == 3
        assert bytes(channel.output) == encoded_csv[:7] and channel.flushes == 0
        channel = CsvChannel(0)
        assert result_csv_stream(cloned, c.byref(channel.stream)) == 3 and not channel.output and channel.flushes == 0
        channel = CsvChannel(1)
        assert result_csv_stream(None, c.byref(channel.stream)) == 1 and channel.calls == 0
        assert result_csv_stream(cloned, None) == 1
        missing = Writer(None, Write(), Flush())
        assert result_csv_stream(cloned, c.byref(missing)) == 3
        @Write
        def excessive(context, buffer, capacity, count):
            count[0] = capacity + 1
            return 0
        invalid = Writer(None, excessive, Flush())
        assert result_csv_stream(cloned, c.byref(invalid)) == 3
        retained_csv = P(csv.value)
        assert result_csv(None, c.byref(retained_csv)) == 1 and retained_csv.value == csv.value
        assert encoded_csv.endswith(b'\n')
        csv_rows = [[float(x) for x in line.split(',')] for line in encoded_csv.decode().splitlines()]
        assert len(csv_rows) == len(expected_rows)
        for actual_row, expected_row in zip(csv_rows, expected_rows):
            assert len(actual_row) == len(expected_row)
            assert all(abs(a - b) <= 1e-5 for a, b in zip(actual_row, expected_row))
        assert release_bytes(c.byref(csv)) == 0
        for index, expected in enumerate(expected_rows):
            value = P()
            assert result_report(cloned, index, c.byref(value)) == 0
            actual = report(value)
            assert actual == report(events[index])
            assert actual[0] == index and len(actual[3]) == 1
            temperature = F(math.sqrt(F((index + 1) / iterations).value)).value if anneal else 1.0
            assert actual[1] == temperature and math.isfinite(actual[2])
            assert len(actual[3][0]) == len(expected)
            for a, b in zip(actual[3][0], expected):
                assert abs(a - b) <= 1e-5
            # Native mean divides each group before summing in binary32.
            total = F(0)
            for value_in_row in actual[3][0]:
                total = F(total.value + F(value_in_row / len(expected)).value)
            assert actual[2] == F(total.value / temperature).value
            assert report_release(c.byref(value)) == 0
        assert result_release(c.byref(cloned)) == 0
        for value in events:
            assert math.isfinite(report(value)[2])
            assert report_release(c.byref(value)) == 0
        events.clear()
        # Save/reload a complete trained model.
        saved, reloaded = owned(wire(trained)), P()
        assert model_read(saved, 16 * 1024 * 1024, c.byref(reloaded)) == 0
        assert wire(reloaded) == expected_wire
        assert release_bytes(c.byref(saved)) == 0
        assert model_release(c.byref(reloaded)) == 0
        assert model_release(c.byref(trained)) == 0
        assert files_release(c.byref(files)) == 0
    # Both duration modes stop after the second notification, on two ordered files.
    document['file_list'].append(document['file_list'][0].copy())
    assert release_bytes(c.byref(encoded)) == 0
    encoded = owned(json.dumps(document).encode())
    files = P()
    assert files_read(model, encoded, 12, 1, c.byref(files)) == 0
    for mode in range(2):
        assert defaults(c.byref(config)) == 0
        config.iterations, config.duration_mode, config.workers = 5, mode, 2
        config.termination_threshold = F(3.4028234663852886e38).value
        result = P()
        assert train_progress(model, files, c.byref(config), progress, P(73), c.byref(result)) == 0
        assert result_length(result, c.byref(count)) == 0 and count.value == 2
        assert len(events) == 2 and not callback_errors
        assert result_release(c.byref(result)) == 0
        for value in events:
            assert [len(row) for row in report(value)[3]] == [2, 2]
            assert report_release(c.byref(value)) == 0
        events.clear()
    retained = P(files.value)
    assert files_read(model, encoded, 11, 1, c.byref(retained)) == 3 and retained.value == files.value
    assert files_read(model, encoded, 12, 2, c.byref(retained)) == 2 and retained.value == files.value
    assert files_dataset(files, 2, c.byref(snapshot)) == 2 and snapshot.value is None
    assert files_release(c.byref(files)) == 0
    assert release_bytes(c.byref(encoded)) == 0

empty, result = P(), P()
assert files_create(None, 0, c.byref(empty)) == 0
assert defaults(c.byref(config)) == 0
config.iterations = 0
assert train(model, empty, c.byref(config), c.byref(result)) == 0
assert result_length(result, c.byref(count)) == 0 and count.value == 0
channel = CsvChannel(1)
assert result_csv_stream(result, c.byref(channel.stream)) == 0
assert not channel.output and channel.calls == 0 and channel.flushes == 0
retained = P(result.value)
config.duration_mode = 2
assert train(model, empty, c.byref(config), c.byref(retained)) == 2 and retained.value == result.value
config.duration_mode, config.iterations = 0, 1
assert train(model, empty, c.byref(config), c.byref(retained)) == 3 and retained.value == result.value
assert train(model, empty, None, c.byref(retained)) == 1 and retained.value == result.value
assert result_report(result, 0, c.byref(snapshot)) == 2 and snapshot.value is None
assert defaults(None) == 1 and files_release(None) == 1 and result_release(None) == 1 and report_release(None) == 1
assert result_release(c.byref(result)) == 0 and result_release(c.byref(result)) == 0
assert files_release(c.byref(empty)) == 0
bits = (U * 5)(0x80000000, 0x7f800000, 0xff800000, 0x7fc01234, 1)
row, empty_row, arbitrary, assembled, clone = P(), P(), P(), P(), P()
assert array_create(c.cast(bits, c.POINTER(F)), 5, c.byref(row)) == 0
assert array_create(None, 0, c.byref(empty_row)) == 0
rows = (P * 3)(row, empty_row, row)
info = Info(N(-1).value, -0.0, float('inf'))
assert report_create(c.byref(info), rows, 3, c.byref(arbitrary)) == 0
reports = (P * 2)(arbitrary, arbitrary)
assert result_create(model, reports, 2, c.byref(assembled)) == 0
assert result_clone(assembled, c.byref(clone)) == 0
invalid = (P * 2)(arbitrary, None)
retained = P(assembled.value)
assert result_create(model, invalid, 2, c.byref(retained)) == 1 and retained.value == assembled.value
assert result_create(model, None, 0, None) == 1
assert result_replace(assembled, model, invalid, 2) == 1
assert result_replace(assembled, model, None, N(-1).value) == 2
replacement = Info(7, 2.0, 3.0)
invalid_rows = (P * 2)(row, None)
assert report_replace(arbitrary, c.byref(replacement), invalid_rows, 2) == 1
assert report(arbitrary)[0] == info.iteration
assert report_replace(arbitrary, c.byref(replacement), None, N(-1).value) == 2
assert report_replace(arbitrary, c.byref(replacement), None, 0) == 0
assert report(arbitrary) == (7, 2.0, 3.0, [])
assert result_replace(assembled, model, reports, 1) == 0
snapshot = P()
assert result_report(assembled, 0, c.byref(snapshot)) == 0
assert report(snapshot) == (7, 2.0, 3.0, [])
assert report_release(c.byref(snapshot)) == 0
replacement_wire = (root / 'rest-c-hsmm-one.hsmm').read_bytes()
encoded, replacement_model = owned(replacement_wire), P()
assert model_read(encoded, 16 * 1024 * 1024, c.byref(replacement_model)) == 0
assert release_bytes(c.byref(encoded)) == 0
assert result_replace(assembled, replacement_model, None, 0) == 0
assert model_release(c.byref(replacement_model)) == 0
assert result_model(assembled, c.byref(snapshot)) == 0 and wire(snapshot) == replacement_wire
assert model_release(c.byref(snapshot)) == 0
assert result_length(assembled, c.byref(count)) == 0 and count.value == 0
assert result_release(c.byref(assembled)) == 0
assert report_release(c.byref(arbitrary)) == 0
assert array_release(c.byref(row)) == 0 and array_release(c.byref(empty_row)) == 0
assert result_length(clone, c.byref(count)) == 0 and count.value == 2
for index in range(2):
    assert result_report(clone, index, c.byref(snapshot)) == 0
    observed = Info()
    assert report_info(snapshot, c.byref(observed)) == 0
    assert bytes(observed) == bytes(info)
    assert report_count(snapshot, c.byref(count)) == 0 and count.value == 3
    for file_index in range(3):
        copied_row = P()
        assert report_file(snapshot, file_index, c.byref(copied_row)) == 0
        size = N()
        assert array_length(copied_row, c.byref(size)) == 0
        expected = b'' if file_index == 1 else bytes(bits)
        actual = (F * size.value)()
        assert array_copy(copied_row, 0, actual, size.value) == 0
        assert bytes(actual) == expected
        assert array_release(c.byref(copied_row)) == 0
    assert report_release(c.byref(snapshot)) == 0
assert result_model(clone, c.byref(snapshot)) == 0 and wire(snapshot) == initial_wire
assert model_release(c.byref(snapshot)) == 0
assert wire(model) == initial_wire and model_release(c.byref(model)) == 0
assert result_report(clone, 0, c.byref(snapshot)) == 0
assert report(snapshot)[0] == info.iteration
assert report_release(c.byref(snapshot)) == 0 and result_release(c.byref(clone)) == 0
assert symbols <= called
print('SHIRO training ctypes: all25 exports, nine original C models/likelihoods/CSV, direct partial CSV streams, arbitrary result/report fields, atomic replacement, independent ownership, callbacks and failures passed')
