"""Actual ctypes complete utterance stages, reports and decoded-wave workflows."""
import ctypes as c
import json
import math
from pathlib import Path
import struct
import sys
import wave

lib = c.CDLL(sys.argv[1])
P, N, U, B, F, D = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8, c.c_float, c.c_double
symbols = {'utterance_options_default', 'utterances_create', 'utterances_has_model', 'utterances_get_model',
    'utterances_split_features', 'utterances_split_wave', 'utterances_get_phonemap', 'utterances_get_definition',
    'utterances_get_phones', 'utterances_get_initial_segmentation', 'utterances_get_iterations', 'utterances_get_alignment',
    'utterances_get_labels', 'utterances_clone', 'utterances_release', 'segmented_wave_create', 'segmented_wave_get_audio',
    'segmented_wave_get_features', 'segmented_wave_get_utterances', 'segmented_wave_clone', 'segmented_wave_release'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO utterances ctypes: all21 symbols absent without c-api')
    sys.exit(0)

class Options(c.Structure):
    _fields_ = [('utterances', N), ('hop_seconds', D), ('minimum_silence_seconds', D), ('minimum_voicing_seconds', D), ('iterations', N)]

class Source(c.Structure):
    _fields_ = [('mode', U), ('model', P)]

class Input(c.Structure):
    _fields_ = [(name, P) for name in ['phonemap', 'definition', 'phones', 'initial_segmentation',
        'uninitialized_model', 'initialized_model', 'model', 'iterations', 'alignment', 'labels']]

class WaveInfo(c.Structure):
    _fields_ = [('sample_rate', U), ('bits_per_sample', c.c_uint16), ('channels', c.c_uint16), ('encoding', U)]

class WaveInput(c.Structure):
    _fields_ = [('header', WaveInfo), ('samples', P), ('filename', P), ('dimensions', N), ('kind', U)]

class Iteration(c.Structure):
    _fields_ = [('iteration', N), ('temperature', F), ('mean_log_likelihood', F)]

class Label(c.Structure):
    _fields_ = [('start', D), ('end', D)]

class FeatureInfo(c.Structure):
    _fields_ = [('frames', N), ('columns', N)]

Callback = c.CFUNCTYPE(U, P, c.POINTER(F))
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
array_create = function('array_f32_create', [c.POINTER(F), N, out])
array_length = function('array_f32_length', [P, c.POINTER(N)])
array_copy = function('array_f32_copy', [P, N, c.POINTER(F), N])
array_release = function('array_f32_release', [out])
model_read = function('model_read_bytes', [P, N, out])
model_write = function('model_write_bytes', [P, U, out])
model_release = function('model_release', [out])
features_create = function('features_create', [N, N, P, out])
features_info = function('features_get_info', [P, c.POINTER(FeatureInfo)])
features_values = function('features_get_values', [P, out])
features_release = function('features_release', [out])
defaults = function('utterance_options_default', [c.POINTER(Options)])
create = function('utterances_create', [c.POINTER(Input), out])
has_model = function('utterances_has_model', [P, U, c.POINTER(U)])
get_model = function('utterances_get_model', [P, U, out])
split_features = function('utterances_split_features', [P, P, c.POINTER(Options), c.POINTER(Source), out])
split_wave = function('utterances_split_wave', [c.POINTER(WaveInput), c.POINTER(Options), c.POINTER(Source), Callback, P, out])
get_map = function('utterances_get_phonemap', [P, out])
get_definition = function('utterances_get_definition', [P, out])
get_phones = function('utterances_get_phones', [P, out])
get_initial = function('utterances_get_initial_segmentation', [P, out])
get_reports = function('utterances_get_iterations', [P, out])
get_alignment = function('utterances_get_alignment', [P, out])
get_labels = function('utterances_get_labels', [P, out])
clone = function('utterances_clone', [P, out])
release = function('utterances_release', [out])
wave_create = function('segmented_wave_create', [P, P, P, out])
wave_audio = function('segmented_wave_get_audio', [P, out])
wave_features = function('segmented_wave_get_features', [P, out])
wave_utterances = function('segmented_wave_get_utterances', [P, out])
wave_clone = function('segmented_wave_clone', [P, out])
wave_release = function('segmented_wave_release', [out])
map_write = function('phone_map_write_json', [P, out])
map_release = function('phone_map_release', [out])
definition_write = function('definition_write_json', [P, out])
definition_release = function('definition_release', [out])
strings_length = function('strings_length', [P, c.POINTER(N)])
strings_get = function('strings_get', [P, N, out])
strings_release = function('strings_release', [out])
document_write = function('document_write_json', [P, out])
document_release = function('document_release', [out])
reports_length = function('iteration_reports_length', [P, c.POINTER(N)])
reports_get = function('iteration_reports_get', [P, N, out])
reports_release = function('iteration_reports_release', [out])
report_info = function('iteration_report_info', [P, c.POINTER(Iteration)])
report_count = function('iteration_report_file_count', [P, c.POINTER(N)])
report_file = function('iteration_report_get_file', [P, N, out])
report_release = function('iteration_report_release', [out])
labels_length = function('labels_length', [P, c.POINTER(N)])
labels_info = function('labels_get_info', [P, N, c.POINTER(Label)])
labels_name = function('labels_get_name', [P, N, out])
labels_release = function('labels_release', [out])
audio_rate = function('audio_sample_rate', [P, c.POINTER(U)])
audio_samples = function('audio_get_samples', [P, out])
audio_release = function('audio_release', [out])
rng_create = function('dither_linux_gnu', [out])
rng_next = function('dither_next_uniform', [P, c.POINTER(F)])
rng_release = function('dither_release', [out])

def owned(data):
    output = P()
    assert bytes_create((B * len(data)).from_buffer_copy(data), len(data), c.byref(output)) == 0
    return output

def copied(owner):
    count = N()
    assert bytes_length(owner, c.byref(count)) == 0
    output = (B * count.value)()
    assert bytes_copy(owner, 0, output, count.value) == 0
    return bytes(output)

def array(data):
    output = P()
    assert array_create(data, len(data), c.byref(output)) == 0
    return output

def array_bits(owner):
    count = N()
    assert array_length(owner, c.byref(count)) == 0
    output = (U * count.value)()
    assert array_copy(owner, 0, c.cast(output, c.POINTER(F)), count.value) == 0
    return list(output)

def json_value(owner, writer):
    output = P()
    assert writer(owner, c.byref(output)) == 0
    value = json.loads(copied(output))
    assert bytes_release(c.byref(output)) == 0
    return value

def model_bytes(owner):
    output = P()
    assert model_write(owner, 0, c.byref(output)) == 0
    value = copied(output)
    assert bytes_release(c.byref(output)) == 0
    return value

def bits(owner, field, integer):
    return integer.from_address(c.addressof(owner) + getattr(type(owner), field).offset).value

def snapshot(owner):
    result = {}
    for key, getter, writer, free in [
        ('phonemap', get_map, map_write, map_release), ('definition', get_definition, definition_write, definition_release),
        ('initial', get_initial, document_write, document_release), ('alignment', get_alignment, document_write, document_release)]:
        output = P()
        assert getter(owner, c.byref(output)) == 0
        result[key] = json_value(output, writer)
        assert free(c.byref(output)) == 0
    phones, count = P(), N()
    assert get_phones(owner, c.byref(phones)) == 0 and strings_length(phones, c.byref(count)) == 0
    result['phones'] = []
    for index in range(count.value):
        value = P()
        assert strings_get(phones, index, c.byref(value)) == 0
        result['phones'].append(copied(value))
        assert bytes_release(c.byref(value)) == 0
    assert strings_release(c.byref(phones)) == 0
    result['models'] = []
    for stage in range(3):
        present, model = U(99), P()
        assert has_model(owner, stage, c.byref(present)) == 0
        assert get_model(owner, stage, c.byref(model)) == (0 if present.value else 2)
        result['models'].append(model_bytes(model) if present.value else None)
        assert model_release(c.byref(model)) == 0
    reports = P()
    assert get_reports(owner, c.byref(reports)) == 0 and reports_length(reports, c.byref(count)) == 0
    result['reports'] = []
    for index in range(count.value):
        report, info, files = P(), Iteration(), N()
        assert reports_get(reports, index, c.byref(report)) == 0
        assert report_info(report, c.byref(info)) == 0 and report_count(report, c.byref(files)) == 0
        rows = []
        for row_index in range(files.value):
            row = P()
            assert report_file(report, row_index, c.byref(row)) == 0
            rows.append(array_bits(row))
            assert array_release(c.byref(row)) == 0
        result['reports'].append((info.iteration, bits(info, 'temperature', U), bits(info, 'mean_log_likelihood', U), rows))
        assert report_release(c.byref(report)) == 0
    assert reports_release(c.byref(reports)) == 0
    labels = P()
    assert get_labels(owner, c.byref(labels)) == 0 and labels_length(labels, c.byref(count)) == 0
    result['labels'] = []
    for index in range(count.value):
        info, name = Label(), P()
        assert labels_info(labels, index, c.byref(info)) == 0 and labels_name(labels, index, c.byref(name)) == 0
        result['labels'].append((bits(info, 'start', c.c_uint64), bits(info, 'end', c.c_uint64), copied(name)))
        assert bytes_release(c.byref(name)) == 0
    assert labels_release(c.byref(labels)) == 0
    return result

def reconstruct(owner):
    fields = {}
    for key, getter, free in [('phonemap', get_map, map_release), ('definition', get_definition, definition_release),
        ('phones', get_phones, strings_release), ('initial_segmentation', get_initial, document_release),
        ('iterations', get_reports, reports_release), ('alignment', get_alignment, document_release), ('labels', get_labels, labels_release)]:
        value = P()
        assert getter(owner, c.byref(value)) == 0
        fields[key] = (value, free)
    for stage, key in enumerate(['uninitialized_model', 'initialized_model', 'model']):
        value, present = P(), U()
        assert has_model(owner, stage, c.byref(present)) == 0
        assert get_model(owner, stage, c.byref(value)) == (0 if present.value else 2)
        fields[key] = (value, model_release)
    descriptor = Input(**{key: value.value for key, (value, _) in fields.items()})
    output = P()
    assert create(c.byref(descriptor), c.byref(output)) == 0
    invalid = Input(**{key: value.value for key, (value, _) in fields.items()})
    invalid.model = None
    retained = P(output.value)
    assert create(c.byref(invalid), c.byref(retained)) == 1 and retained.value == output.value
    for value, free in fields.values():
        assert free(c.byref(value)) == 0
    return output

root = Path(__file__).parent / 'fixtures'
raw = (root / 'utterances-c-features.bin').read_bytes()
values = (F * (len(raw) // 4)).from_buffer_copy(raw)
data = array(values)
features = P()
assert features_create(40, 13, data, c.byref(features)) == 0 and array_release(c.byref(data)) == 0
filename = owned(b'sample.param')
settings = Options()
assert defaults(c.byref(settings)) == 0
assert [getattr(settings, key) for key, _ in Options._fields_] == [1, 0.1, 0.3, 0.3, 15]
settings.utterances, settings.iterations = 2, 2
for mode, model_name in [(0, None), (1, 'utterances-c-flat.hsmm'), (2, 'utterances-c-trained.hsmm')]:
    model = P()
    if model_name:
        encoded = owned((root / model_name).read_bytes())
        assert model_read(encoded, 16 * 1024 * 1024, c.byref(model)) == 0 and bytes_release(c.byref(encoded)) == 0
    source = Source(mode, model.value if mode else 1)
    output = P()
    assert split_features(features, filename, c.byref(settings), c.byref(source), c.byref(output)) == 0
    if model_name:
        assert model_bytes(model) == (root / model_name).read_bytes()
    assert model_release(c.byref(model)) == 0
    expected = snapshot(output)
    assert expected['models'][2] == (root / 'utterances-c-trained.hsmm').read_bytes()
    assert expected['models'][:2] == ([(root / 'utterances-c-uninit.hsmm').read_bytes(), (root / 'utterances-c-flat.hsmm').read_bytes()] if mode == 0 else [None, None])
    assert expected['phones'] == [b'sil', b'utt', b'sil', b'utt', b'sil']
    for key, fixture in [('phonemap', 'utterances-c-phonemap.json'), ('initial', 'utterances-c-initial.json'), ('alignment', 'utterances-c-aligned.json')]:
        assert expected[key] == json.loads((root / fixture).read_bytes())
    definition = json.loads((root / 'utterances-c-definition.json').read_bytes())
    definition['dur_attr'].sort(key=lambda value: value['index'])
    assert expected['definition'] == definition
    assert len(expected['reports']) == (0 if mode == 2 else 2)
    for order, (index, temperature, mean, rows) in enumerate(expected['reports']):
        assert index == order and temperature == U.from_buffer_copy(struct.pack('<f', math.sqrt((index + 1) / 2))).value
        assert len(rows) == 1 and len(rows[0]) == 1
        row_value = struct.unpack('<f', struct.pack('<I', rows[0][0]))[0]
        temp_value = struct.unpack('<f', struct.pack('<I', temperature))[0]
        assert mean == U.from_buffer_copy(struct.pack('<f', row_value / temp_value)).value
    expected_labels = [line.split('\t') for line in (root / 'utterances-c-labels.txt').read_text().splitlines()]
    assert len(expected['labels']) == len(expected_labels)
    for (start, end, name), original in zip(expected['labels'], expected_labels):
        assert abs(struct.unpack('<d', struct.pack('<Q', start))[0] - float(original[0])) < 1e-14
        assert abs(struct.unpack('<d', struct.pack('<Q', end))[0] - float(original[1])) < 1e-14
        assert name == original[2].encode()
    rebuilt, cloned = reconstruct(output), P()
    assert clone(rebuilt, c.byref(cloned)) == 0 and release(c.byref(rebuilt)) == 0 and release(c.byref(output)) == 0
    assert snapshot(cloned) == expected
    retained = P(cloned.value)
    for config in [Options(N(-1).value, 0.1, 0.3, 0.3, 2), Options(2, 0, 0.3, 0.3, 2)]:
        assert split_features(features, filename, c.byref(config), c.byref(Source(0, None)), c.byref(retained)) == 3 and retained.value == cloned.value
    present = U(73)
    assert has_model(cloned, 3, c.byref(present)) == 2 and present.value == 73
    assert release(c.byref(cloned)) == 0
for config in [Options(2, 0.1, 0.3, 0.3, 0), Options(1, 0.125, 0.25, 0.375, 1)]:
    nondefault_name = owned(b'audio-\xf0\x9f\x8e\xb5\0.param')
    output = P()
    assert split_features(features, nondefault_name, c.byref(config), c.byref(Source(0, None)), c.byref(output)) == 0
    expected = snapshot(output)
    assert len(expected['phones']) == config.utterances * 2 + 1
    assert len(expected['reports']) == config.iterations
    assert expected['initial']['file_list'][0]['filename'].encode() == b'audio-\xf0\x9f\x8e\xb5\0.param'
    assert expected['alignment']['file_list'][0]['filename'].encode() == b'audio-\xf0\x9f\x8e\xb5\0.param'
    rebuilt = reconstruct(output)
    assert release(c.byref(output)) == 0 and snapshot(rebuilt) == expected
    assert release(c.byref(rebuilt)) == 0 and bytes_release(c.byref(nondefault_name)) == 0
assert features_release(c.byref(features)) == 0 and bytes_release(c.byref(filename)) == 0

with wave.open(str(root / 'utterances-input.wav'), 'rb') as audio_file:
    assert audio_file.getsampwidth() == 2 and audio_file.getnchannels() == 1
    sample_rate = audio_file.getframerate()
    raw = audio_file.readframes(audio_file.getnframes())
pcm = struct.unpack('<' + 'h' * (len(raw) // 2), raw)
decoded = (F * len(pcm))(*(value / 32768 for value in pcm))
samples, filename = array(decoded), owned(b'sample.param')
callback_errors, draws, fail_after = [], [0], [N(-1).value]
rng = P()
@Callback
def uniform(context, output):
    try:
        if context != 73:
            callback_errors.append('invalid context')
            return 7
        if draws[0] == fail_after[0]:
            return 7
        status = rng_next(rng, output)
        if status == 0:
            draws[0] += 1
        return status
    except BaseException as error:
        callback_errors.append(error)
        return 7

for kind in range(3):
    assert rng_create(c.byref(rng)) == 0
    draws[0] = 0
    input = WaveInput(WaveInfo(sample_rate, 16, 1, 0), samples.value, filename.value, 13, kind)
    source = Source(0, None)
    output = P()
    assert split_wave(c.byref(input), c.byref(settings), c.byref(source), uniform, 73, c.byref(output)) == 0
    assert not callback_errors and draws[0] > 0
    assert rng_release(c.byref(rng)) == 0
    audio, extracted, utterances = P(), P(), P()
    assert wave_audio(output, c.byref(audio)) == 0 and wave_features(output, c.byref(extracted)) == 0
    assert wave_utterances(output, c.byref(utterances)) == 0
    expected = snapshot(utterances)
    rebuilt, cloned = P(), P()
    assert wave_create(audio, extracted, utterances, c.byref(rebuilt)) == 0
    assert wave_clone(rebuilt, c.byref(cloned)) == 0 and wave_release(c.byref(rebuilt)) == 0 and wave_release(c.byref(output)) == 0
    rate, audio_data = U(), P()
    assert audio_rate(audio, c.byref(rate)) == 0 and rate.value == 16000
    assert audio_samples(audio, c.byref(audio_data)) == 0
    if kind == 0:
        original = (root / 'utterances-c-audio.bin').read_bytes()
        assert array_bits(audio_data) == list(struct.unpack('<' + 'I' * (len(original) // 4), original))
    assert array_release(c.byref(audio_data)) == 0
    info, extracted_values = FeatureInfo(), P()
    assert features_info(extracted, c.byref(info)) == 0 and (info.frames, info.columns) == (40, 13)
    assert features_values(extracted, c.byref(extracted_values)) == 0
    if kind == 0:
        actual = [struct.unpack('<f', struct.pack('<I', bits))[0] for bits in array_bits(extracted_values)]
        original = (root / 'utterances-c-features.bin').read_bytes()
        reference = struct.unpack('<' + 'f' * (len(original) // 4), original)
        assert len(actual) == len(reference) and max(abs(a - b) / max(abs(b), 1) for a, b in zip(actual, reference)) <= 2e-5
    assert array_release(c.byref(extracted_values)) == 0
    assert audio_release(c.byref(audio)) == 0 and features_release(c.byref(extracted)) == 0 and release(c.byref(utterances)) == 0
    assert wave_utterances(cloned, c.byref(utterances)) == 0 and wave_release(c.byref(cloned)) == 0
    assert snapshot(utterances) == expected and release(c.byref(utterances)) == 0
    assert rng_create(c.byref(rng)) == 0
    draws[0], fail_after[0] = 0, 3
    retained = P()
    assert split_wave(c.byref(input), c.byref(settings), c.byref(source), uniform, 73, None) == 1 and draws[0] == 0
    invalid = WaveInput(input.header, samples.value, filename.value, 1, kind)
    assert split_wave(c.byref(invalid), c.byref(settings), c.byref(source), uniform, 73, c.byref(retained)) == 3 and draws[0] == 0
    assert split_wave(c.byref(input), c.byref(settings), c.byref(source), uniform, 73, c.byref(retained)) == 3
    assert draws[0] == 3 and retained.value is None and not callback_errors
    assert rng_release(c.byref(rng)) == 0
    fail_after[0] = N(-1).value
for mode in [0, 1, 2]:
    model = P()
    model_name = 'utterances-c-flat.hsmm' if mode == 1 else 'utterances-c-trained.hsmm'
    encoded = owned((root / model_name).read_bytes())
    assert model_read(encoded, 16 * 1024 * 1024, c.byref(model)) == 0 and bytes_release(c.byref(encoded)) == 0
    for modified in [False, True]:
        assert rng_create(c.byref(rng)) == 0
        draws[0] = 0
        header = WaveInfo(sample_rate // 2 if modified else sample_rate, 0 if modified else 16, 65535 if modified else 1, int(modified))
        input = WaveInput(header, samples.value, filename.value, 13, 0)
        source = Source(mode, model.value)
        config = Options(2, 0.1, 0.3, 0.3, 0)
        output = P()
        assert split_wave(c.byref(input), c.byref(config), c.byref(source), uniform, 73, c.byref(output)) == 0
        assert not callback_errors and draws[0] > 0
        assert rng_release(c.byref(rng)) == 0
        assert model_bytes(model) == (root / model_name).read_bytes()
        result = P()
        assert wave_utterances(output, c.byref(result)) == 0 and wave_release(c.byref(output)) == 0
        expected = snapshot(result)
        assert expected['reports'] == []
        assert [value is not None for value in expected['models']] == ([True, True, True] if mode == 0 else [False, False, True])
        if mode != 0:
            assert expected['models'][2] == (root / model_name).read_bytes()
        rebuilt = reconstruct(result)
        assert release(c.byref(result)) == 0 and snapshot(rebuilt) == expected
        assert release(c.byref(rebuilt)) == 0
    assert model_release(c.byref(model)) == 0
assert array_release(c.byref(samples)) == 0 and bytes_release(c.byref(filename)) == 0

class Stream(c.Structure):
    _fields_ = [('states', N), ('dimensions', N), ('mixtures', N), ('weight', F)]

class Constraint(c.Structure):
    _fields_ = [('index', N), ('has_minimum', U), ('minimum', c.c_int32), ('has_maximum', U), ('maximum', c.c_int32)]

class DefinitionInfo(c.Structure):
    _fields_ = [('duration_states', N), ('streams', N), ('duration_constraints', N)]

class StateInput(c.Structure):
    _fields_ = [('time', D), ('has_duration', U), ('duration', N), ('outputs', P), ('jumps', P), ('metadata', P), ('attributes', P)]

class StateInfo(c.Structure):
    _fields_ = [('time', D), ('has_duration', U), ('duration', N), ('has_outputs', U), ('has_jumps', U)]

class LabelInput(c.Structure):
    _fields_ = [('start', D), ('end', D), ('name', P)]

map_read = function('phone_map_read_json', [P, out])
definition_create = function('definition_create', [N, c.POINTER(Stream), N, c.POINTER(Constraint), N, out])
definition_info = function('definition_get_info', [P, c.POINTER(DefinitionInfo)])
definition_stream = function('definition_get_stream', [P, N, c.POINTER(Stream)])
definition_constraint = function('definition_get_constraint', [P, N, c.POINTER(Constraint)])
strings_create = function('strings_create', [c.POINTER(P), N, out])
usize_create = function('array_usize_create', [c.POINTER(N), N, out])
usize_length = function('array_usize_length', [P, c.POINTER(N)])
usize_copy = function('array_usize_copy', [P, N, c.POINTER(N), N])
usize_release = function('array_usize_release', [out])
states_create = function('states_create', [c.POINTER(StateInput), N, out])
states_length = function('states_length', [P, c.POINTER(N)])
states_info = function('states_get_info', [P, N, c.POINTER(StateInfo)])
states_outputs = function('states_get_outputs', [P, N, out])
states_field = function('states_get_json_field', [P, N, U, out])
states_release = function('states_release', [out])
file_create = function('segmented_file_create', [P, P, P, out])
file_name = function('segmented_file_get_filename', [P, out])
file_states = function('segmented_file_get_states', [P, out])
file_attrs = function('segmented_file_get_attributes', [P, out])
file_release = function('segmented_file_release', [out])
document_create = function('document_create', [c.POINTER(P), N, P, out])
document_length = function('document_length', [P, c.POINTER(N)])
document_file = function('document_get_file', [P, N, out])
document_attrs = function('document_get_attributes', [P, out])
report_create = function('iteration_report_create', [c.POINTER(Iteration), c.POINTER(P), N, out])
reports_create = function('iteration_reports_create', [c.POINTER(P), N, out])
labels_create = function('labels_create', [c.POINTER(LabelInput), N, out])
audio_create = function('audio_create', [U, P, out])

def encoded(value):
    return owned(json.dumps(value).encode())

def set_bits(owner, field, integer, value):
    integer.from_address(c.addressof(owner) + getattr(type(owner), field).offset).value = value

maximum, time_bits, weight_bits = N(-1).value, 0x7ff8123456789abc, 0x7fc12345
patterns = [0x80000000, 0x7fc54321, 0x7f800000, 1]
map_value = {'phone_map': {}, 'nested': {'data': [None, True]}}
map_json, phonemap = encoded(map_value), P()
assert map_read(map_json, c.byref(phonemap)) == 0 and bytes_release(c.byref(map_json)) == 0
stream, constraint = Stream(maximum, 0, maximum, 0), Constraint(maximum, 1, -2147483648, 1, 0)
set_bits(stream, 'weight', U, weight_bits)
definition = P()
assert definition_create(maximum, c.byref(stream), 1, c.byref(constraint), 1, c.byref(definition)) == 0
name, empty_name, phones = owned(b'voice\0.param'), owned(b''), P()
assert strings_create((P * 3)(empty_name.value, name.value, empty_name.value), 3, c.byref(phones)) == 0
assert bytes_release(c.byref(empty_name)) == 0
metadata_value = [None, 'voice', 42, {'nested': [True]}]
state_attrs_value, jumps_value = {'time': 'shadow', 'dur': False}, [[0.25, {'name': 'voice'}]]
metadata, attributes, jumps = encoded(metadata_value), encoded(state_attrs_value), encoded(jumps_value)
integers, outputs = (N * 2)(0, maximum), P()
assert usize_create(integers, 2, c.byref(outputs)) == 0
state = StateInput(0, 1, maximum, outputs.value, jumps.value, metadata.value, attributes.value)
set_bits(state, 'time', c.c_uint64, time_bits)
states = P()
assert states_create(c.byref(state), 1, c.byref(states)) == 0
for value in [metadata, attributes, jumps]:
    assert bytes_release(c.byref(value)) == 0
assert usize_release(c.byref(outputs)) == 0
file_attrs_value = {'filename': 'shadow', 'states': False}
attrs, file = encoded(file_attrs_value), P()
assert file_create(name, states, attrs, c.byref(file)) == 0
assert states_release(c.byref(states)) == 0 and bytes_release(c.byref(attrs)) == 0
document_attrs_value = {'file_list': 'shadow', 'nested': [None, 7]}
attrs, document = encoded(document_attrs_value), P()
assert document_create((P * 2)(file.value, file.value), 2, attrs, c.byref(document)) == 0
assert file_release(c.byref(file)) == 0 and bytes_release(c.byref(attrs)) == 0
raw_values = (U * len(patterns))(*patterns)
row, empty_row = P(), P()
assert array_create(c.cast(raw_values, c.POINTER(F)), len(patterns), c.byref(row)) == 0
assert array_create(None, 0, c.byref(empty_row)) == 0
iteration = Iteration(maximum, 0, 0)
set_bits(iteration, 'temperature', U, weight_bits)
set_bits(iteration, 'mean_log_likelihood', U, 0xff800000)
report, reports = P(), P()
assert report_create(c.byref(iteration), (P * 3)(row.value, empty_row.value, row.value), 3, c.byref(report)) == 0
assert reports_create((P * 2)(report.value, report.value), 2, c.byref(reports)) == 0
assert report_release(c.byref(report)) == 0 and array_release(c.byref(row)) == 0 and array_release(c.byref(empty_row)) == 0
label, labels = LabelInput(0, 0, name.value), P()
set_bits(label, 'start', c.c_uint64, time_bits)
set_bits(label, 'end', c.c_uint64, 0xfff0000000000000)
assert labels_create(c.byref(label), 1, c.byref(labels)) == 0 and bytes_release(c.byref(name)) == 0
model = P()
model_raw = (root / 'utterances-c-trained.hsmm').read_bytes()
encoded_model = owned(model_raw)
assert model_read(encoded_model, 16 * 1024 * 1024, c.byref(model)) == 0 and bytes_release(c.byref(encoded_model)) == 0
constructed = []
for uninitialized, initialized in [(False, False), (True, False), (False, True), (True, True)]:
    input = Input(phonemap.value, definition.value, phones.value, document.value,
        model.value if uninitialized else None, model.value if initialized else None, model.value,
        reports.value, document.value, labels.value)
    output, cloned = P(), P()
    assert create(c.byref(input), c.byref(output)) == 0
    assert clone(output, c.byref(cloned)) == 0 and release(c.byref(output)) == 0
    constructed.append((cloned, uninitialized, initialized))
for owner, free in [(phonemap, map_release), (definition, definition_release), (phones, strings_release),
    (document, document_release), (reports, reports_release), (labels, labels_release), (model, model_release)]:
    assert free(c.byref(owner)) == 0

for owner, uninitialized, initialized in constructed:
    phonemap, definition, phones = P(), P(), P()
    assert get_map(owner, c.byref(phonemap)) == 0 and json_value(phonemap, map_write) == map_value
    assert map_release(c.byref(phonemap)) == 0
    assert get_definition(owner, c.byref(definition)) == 0
    info, actual, bound = DefinitionInfo(), Stream(), Constraint()
    assert definition_info(definition, c.byref(info)) == 0
    assert (info.duration_states, info.streams, info.duration_constraints) == (maximum, 1, 1)
    assert definition_stream(definition, 0, c.byref(actual)) == 0
    assert (actual.states, actual.dimensions, actual.mixtures, bits(actual, 'weight', U)) == (maximum, 0, maximum, weight_bits)
    assert definition_constraint(definition, 0, c.byref(bound)) == 0
    assert tuple(getattr(bound, field) for field, _ in Constraint._fields_) == (maximum, 1, -2147483648, 1, 0)
    assert definition_release(c.byref(definition)) == 0
    assert get_phones(owner, c.byref(phones)) == 0
    count = N()
    assert strings_length(phones, c.byref(count)) == 0 and count.value == 3
    for index, expected in enumerate([b'', b'voice\0.param', b'']):
        name = P()
        assert strings_get(phones, index, c.byref(name)) == 0 and copied(name) == expected
        assert bytes_release(c.byref(name)) == 0
    assert strings_release(c.byref(phones)) == 0
    for getter in [get_initial, get_alignment]:
        document = P()
        assert getter(owner, c.byref(document)) == 0
        assert document_length(document, c.byref(count)) == 0 and count.value == 2
        assert json_value(document, document_attrs) == document_attrs_value
        for index in range(2):
            file, name, states = P(), P(), P()
            assert document_file(document, index, c.byref(file)) == 0
            assert file_name(file, c.byref(name)) == 0 and copied(name) == b'voice\0.param'
            assert bytes_release(c.byref(name)) == 0
            assert json_value(file, file_attrs) == file_attrs_value
            assert file_states(file, c.byref(states)) == 0 and file_release(c.byref(file)) == 0
            assert states_length(states, c.byref(count)) == 0 and count.value == 1
            info = StateInfo()
            assert states_info(states, 0, c.byref(info)) == 0
            assert (bits(info, 'time', c.c_uint64), info.has_duration, info.duration, info.has_outputs, info.has_jumps) == (time_bits, 1, maximum, 1, 1)
            outputs = P()
            assert states_outputs(states, 0, c.byref(outputs)) == 0
            assert usize_length(outputs, c.byref(count)) == 0 and count.value == 2
            actual = (N * 2)()
            assert usize_copy(outputs, 0, actual, 2) == 0 and list(actual) == [0, maximum]
            assert usize_release(c.byref(outputs)) == 0
            for field, expected in enumerate([jumps_value, metadata_value, state_attrs_value]):
                value = P()
                assert states_field(states, 0, field, c.byref(value)) == 0
                assert json.loads(copied(value)) == expected and bytes_release(c.byref(value)) == 0
            assert states_release(c.byref(states)) == 0
        retained = encoded({'retained': True})
        sentinel = P(retained.value)
        assert document_write(document, c.byref(sentinel)) == 3 and sentinel.value == retained.value
        assert bytes_release(c.byref(retained)) == 0 and document_release(c.byref(document)) == 0
    for stage, present in enumerate([uninitialized, initialized, True]):
        flag, model = U(99), P()
        assert has_model(owner, stage, c.byref(flag)) == 0 and flag.value == int(present)
        assert get_model(owner, stage, c.byref(model)) == (0 if present else 2)
        if present:
            assert model_bytes(model) == model_raw
        assert model_release(c.byref(model)) == 0
    reports = P()
    assert get_reports(owner, c.byref(reports)) == 0
    assert reports_length(reports, c.byref(count)) == 0 and count.value == 2
    for index in range(2):
        report, info = P(), Iteration()
        assert reports_get(reports, index, c.byref(report)) == 0 and report_info(report, c.byref(info)) == 0
        assert (info.iteration, bits(info, 'temperature', U), bits(info, 'mean_log_likelihood', U)) == (maximum, weight_bits, 0xff800000)
        assert report_count(report, c.byref(count)) == 0 and count.value == 3
        for index in range(3):
            row = P()
            assert report_file(report, index, c.byref(row)) == 0
            assert array_bits(row) == ([] if index == 1 else patterns)
            assert array_release(c.byref(row)) == 0
        assert report_release(c.byref(report)) == 0
    assert reports_release(c.byref(reports)) == 0
    labels, info, name = P(), Label(), P()
    assert get_labels(owner, c.byref(labels)) == 0
    assert labels_length(labels, c.byref(count)) == 0 and count.value == 1
    assert labels_info(labels, 0, c.byref(info)) == 0
    assert (bits(info, 'start', c.c_uint64), bits(info, 'end', c.c_uint64)) == (time_bits, 0xfff0000000000000)
    assert labels_name(labels, 0, c.byref(name)) == 0 and copied(name) == b'voice\0.param'
    assert labels_release(c.byref(labels)) == 0 and bytes_release(c.byref(name)) == 0
    samples, audio, features, wave_owner = P(), P(), P(), P()
    assert array_create(c.cast(raw_values, c.POINTER(F)), len(patterns), c.byref(samples)) == 0
    assert audio_create(0, samples, c.byref(audio)) == 0
    assert features_create(maximum, 0, samples, c.byref(features)) == 0
    assert wave_create(audio, features, owner, c.byref(wave_owner)) == 0
    assert array_release(c.byref(samples)) == 0 and audio_release(c.byref(audio)) == 0
    assert features_release(c.byref(features)) == 0 and release(c.byref(owner)) == 0
    restored = P()
    assert wave_audio(wave_owner, c.byref(audio)) == 0 and wave_features(wave_owner, c.byref(features)) == 0
    assert wave_utterances(wave_owner, c.byref(restored)) == 0 and wave_release(c.byref(wave_owner)) == 0
    rate, feature_info = U(99), FeatureInfo()
    assert audio_rate(audio, c.byref(rate)) == 0 and rate.value == 0
    assert audio_samples(audio, c.byref(samples)) == 0 and array_bits(samples) == patterns
    assert array_release(c.byref(samples)) == 0 and audio_release(c.byref(audio)) == 0
    assert features_info(features, c.byref(feature_info)) == 0 and (feature_info.frames, feature_info.columns) == (maximum, 0)
    assert features_values(features, c.byref(samples)) == 0 and array_bits(samples) == patterns
    assert array_release(c.byref(samples)) == 0 and features_release(c.byref(features)) == 0
    assert get_definition(restored, c.byref(definition)) == 0 and definition_stream(definition, 0, c.byref(actual := Stream())) == 0
    assert bits(actual, 'weight', U) == weight_bits
    assert definition_release(c.byref(definition)) == 0 and release(c.byref(restored)) == 0
assert symbols <= called, symbols - called
print('SHIRO utterances ctypes: all21 exports, original stages/full reports/all feature kinds/ownership/RNG passed')
