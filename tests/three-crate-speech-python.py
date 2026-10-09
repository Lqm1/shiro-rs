"""Connect three independent C ABI libraries through real-speech bytes/arrays.

Owner pointers never cross library boundaries. Original C feature/state corpora
are independent checks; newly learned model roundtrips are interoperability
checks, not a new independent C training oracle.
"""
import argparse
import ctypes as c
import json
import math
from pathlib import Path
import struct

if not __debug__:
    raise RuntimeError('Run verification without Python -O; assertions are required.')

parser = argparse.ArgumentParser(description=__doc__)
for name in ('ciglet', 'liblrhsmm', 'shiro', 'fixtures', 'output'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()
P, N, U, B, F, H = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8, c.c_float, c.c_uint16
slot = c.POINTER(P)

class Api:
    def __init__(self, path, prefix):
        self.lib, self.prefix, self.cache = c.CDLL(str(path.resolve())), prefix, {}
    def call(self, name, types, *values):
        if name not in self.cache:
            fn = getattr(self.lib, self.prefix + name)
            fn.argtypes, fn.restype = types, U
            self.cache[name] = fn
        status = self.cache[name](*values)
        assert status == 0, (self.prefix, name, status)
    def result(self, name, types, *values):
        output = P()
        self.call(name, types + [slot], *values, c.byref(output))
        assert output.value, name
        return output
    def release(self, name, owner):
        self.call(name, [slot], c.byref(owner))
        assert owner.value is None, name
    def bytes(self, data):
        return self.result('bytes_create', [c.POINTER(B), N], (B * len(data)).from_buffer_copy(data), len(data))
    def copy_bytes(self, owner):
        size = N()
        self.call('bytes_length', [P, c.POINTER(N)], owner, c.byref(size))
        output = (B * size.value)()
        self.call('bytes_copy', [P, N, c.POINTER(B), N], owner, 0, output, size.value)
        return bytes(output)

C = Api(args.ciglet, 'ciglet_rs_')
L = Api(args.liblrhsmm, 'liblrhsmm_rs_')
S = Api(args.shiro, 'shiro_rs_')

class WaveInfo(c.Structure):
    _fields_ = [('sample_rate', U), ('bits_per_sample', H), ('channels', H), ('encoding', U), ('sample_count', N)]
class Features(c.Structure):
    _fields_ = [('kind', U), ('order', N), ('channels', N), ('frame_length', N), ('hop', F),
                ('sample_rate_hz', F), ('minimum_bandwidth_hz', F), ('warp', F),
                ('include_dc', U), ('energy', U), ('delta', U), ('acceleration', U)]
class FeatureInfo(c.Structure):
    _fields_ = [('frames', N), ('columns', N)]
class Alignment(c.Structure):
    _fields_ = [('duration_mode', U), ('isolated', U), ('hsmm_temperature', F), ('duration_weight', F),
                ('state_radius', F), ('duration_extra', N), ('duration_extra_factor', F),
                ('geometric_temperature', F), ('pruning_slope', F)]
class Initialization(c.Structure):
    _fields_ = [('flat_start', U), ('globally_tied', U), ('variance_floor_ratio', F)]
class Training(c.Structure):
    _fields_ = [('iterations', N), ('duration_mode', U), ('hsmm_temperature', F), ('duration_weight', F),
                ('state_radius', F), ('duration_extra', N), ('duration_extra_factor', F),
                ('geometric_temperature', F), ('pruning_slope', F), ('termination_threshold', F),
                ('deterministic_annealing', U), ('mean_frame_likelihood', U), ('workers', N)]
class Report(c.Structure):
    _fields_ = [('iteration', N), ('temperature', F), ('mean_log_likelihood', F)]
class Limits(c.Structure):
    _fields_ = [('max_array_entries', N)]

def read_model(data):
    wire = S.bytes(data)
    result = S.result('model_read_bytes', [P, N], wire, 16 * 1024 * 1024)
    S.release('bytes_release', wire)
    return result

def wire_model(model, encoding=0):
    owner = S.result('model_write_bytes', [P, U], model, encoding)
    data = S.copy_bytes(owner)
    S.release('bytes_release', owner)
    return data

def foreign_roundtrip(family, data, encoding=None):
    raw = (B * len(data)).from_buffer_copy(data)
    limits = Limits(16 * 1024 * 1024)
    foreign = L.result(f'{family}_read_bytes_f32', [c.POINTER(B), N, c.POINTER(Limits)], raw, len(data), c.byref(limits))
    if encoding is None:
        wire = L.result(f'{family}_write_bytes_f32', [P], foreign)
    else:
        wire = L.result(f'{family}_write_bytes_f32', [P, U], foreign, encoding)
    assert L.copy_bytes(wire) == data, ('independent library full codec', family)
    L.release('bytes_release', wire)
    L.release(f'{family}_release_f32', foreign)

def read_states(states):
    encoded = S.bytes(json.dumps(states).encode())
    owner = S.result('states_read_json', [P], encoded)
    S.release('bytes_release', encoded)
    return owner

def states_json(states):
    encoded = S.result('states_write_json', [P], states)
    data = json.loads(S.copy_bytes(encoded))
    S.release('bytes_release', encoded)
    return data

def alignment(model, observation, states, options):
    return S.result('align_states', [P, P, P, c.POINTER(Alignment)], model, observation, states, c.byref(options))

fixtures = args.fixtures
model = read_model((fixtures / 'cmu-arctic-all-speakers.hsmm').read_bytes())
legacy = wire_model(model, 1)
assert legacy == (fixtures / 'cmu-arctic-all-speakers.hsmm').read_bytes()
foreign_roundtrip('model', legacy, 1)
references = [json.loads((fixtures / f'cmu-slt-c-{mode}.json').read_text())['file_list'] for mode in ('hmm', 'hsmm')]
settings = Features()
S.call('feature_options_default', [c.POINTER(Features)], c.byref(settings))
settings.delta = settings.acceleration = 1
settings.frame_length, settings.hop, settings.sample_rate_hz = 512, 80, 16000
samples, datasets = [], []
maximum_error, value_count, state_count = 0.0, 0, 0
for index, row in enumerate((fixtures / 'cmu-slt-index.csv').read_text().strip().splitlines()):
    stem = row.split(',', 1)[0]
    print('C ABI speech extraction:', stem, flush=True)
    data = (fixtures / f'cmu-slt-{stem}.wav').read_bytes()
    wave = C.result('wave_decode_f32', [c.POINTER(B), N, c.c_uint64, N],
                    (B * len(data)).from_buffer_copy(data), len(data), 0, 100000)
    info = WaveInfo()
    C.call('wave_info_f32', [P, c.POINTER(WaveInfo)], wave, c.byref(info))
    values = (F * info.sample_count)()
    C.call('wave_copy_samples_f32', [P, c.POINTER(F), N], wave, values, len(values))
    C.release('wave_release_f32', wave)
    assert info.sample_rate == 16000
    array = S.result('array_f32_create', [c.POINTER(F), N], values, len(values))
    features = S.result('features_extract', [P, c.POINTER(Features)], array, c.byref(settings))
    S.release('array_f32_release', array)
    feature_info = FeatureInfo()
    S.call('features_get_info', [P, c.POINTER(FeatureInfo)], features, c.byref(feature_info))
    assert feature_info.columns == 36
    array = S.result('features_get_values', [P], features)
    count = N()
    S.call('array_f32_length', [P, c.POINTER(N)], array, c.byref(count))
    values = (F * count.value)()
    S.call('array_f32_copy', [P, N, c.POINTER(F), N], array, 0, values, count.value)
    S.release('array_f32_release', array)
    S.release('features_release', features)
    expected = (fixtures / f'cmu-slt-{stem}.param').read_bytes()
    assert len(expected) == count.value * 4
    for value, (original,) in zip(values, struct.iter_unpack('<f', expected)):
        error = abs(value - original) / max(1, abs(original))
        assert error <= 5e-5, (stem, error)
        maximum_error = max(maximum_error, error)
    value_count += count.value
    raw = S.bytes(struct.pack('<' + 'f' * len(values), *values))
    observation = S.result('observation_from_model_rawfloat', [P, P, N], raw, model, 100000)
    S.release('bytes_release', raw)
    encoded = S.result('observation_write_bytes', [P], observation)
    foreign_roundtrip('observation', S.copy_bytes(encoded))
    S.release('bytes_release', encoded)
    # Use the complete independent C HMM state sequence as the HSMM input,
    # matching the native real-audio verifier's second inference stage.
    states = read_states(references[0][index]['states'])
    config = Alignment()
    S.call('alignment_options_default', [c.POINTER(Alignment)], c.byref(config))
    config.state_radius, config.duration_extra = 10, 50
    aligned = alignment(model, observation, states, config)
    assert states_json(aligned) == references[1][index]['states'], 'all original C HSMM states'
    state_count += len(states_json(aligned))
    S.release('states_release', aligned)
    encoded = S.result('states_segmentation_bytes', [P, P], states, model)
    foreign_roundtrip('segmentation', S.copy_bytes(encoded))
    S.release('bytes_release', encoded)
    dataset = S.result('dataset_create', [P, c.POINTER(P), c.POINTER(P), N],
                       model, (P * 1)(observation.value), (P * 1)(states.value), 1)
    datasets.append(dataset)
    samples.append((observation, states))

files = S.result('training_files_create', [c.POINTER(P), N], (P * len(datasets))(*(value.value for value in datasets)), len(datasets))
for value in datasets:
    S.release('dataset_release', value)
# Initialize the full corpus as one paired dataset; preserve each file as a
# separate training dataset for original per-file reduction/reporting.
dataset = S.result('dataset_create', [P, c.POINTER(P), c.POINTER(P), N], model,
                   (P * len(samples))(*(value.value for value, _ in samples)),
                   (P * len(samples))(*(value.value for _, value in samples)), len(samples))
initialization = Initialization(1, 1, 1)
initialized = S.result('initialize', [P, P, c.POINTER(Initialization)], model, dataset, c.byref(initialization))
S.release('dataset_release', dataset)
reports = []
for mode in (0, 1):
    print('C ABI real-speech training mode:', mode, flush=True)
    options = Training()
    S.call('training_options_default', [c.POINTER(Training)], c.byref(options))
    options.iterations, options.duration_mode, options.deterministic_annealing = 2, mode, 1
    options.termination_threshold, options.pruning_slope = 0, .8
    result = S.result('train', [P, P, c.POINTER(Training)], initialized, files, c.byref(options))
    length = N()
    S.call('training_result_length', [P, c.POINTER(N)], result, c.byref(length))
    assert length.value == 2
    for index in range(length.value):
        report = S.result('training_result_get_report', [P, N], result, index)
        info = Report()
        S.call('iteration_report_info', [P, c.POINTER(Report)], report, c.byref(info))
        assert info.iteration == index and math.isfinite(info.mean_log_likelihood)
        reports.append(info.mean_log_likelihood)
        S.release('iteration_report_release', report)
    csv = S.result('training_result_likelihood_csv_bytes', [P], result)
    assert len(S.copy_bytes(csv).decode().strip().splitlines()) == 6
    S.release('bytes_release', csv)
    learned = S.result('training_result_get_model', [P], result)
    wire = wire_model(learned)
    foreign_roundtrip('model', wire, 0)
    recovered = read_model(wire)
    assert wire_model(recovered) == wire
    for observation, states in samples:
        before, after = alignment(learned, observation, states, config), alignment(recovered, observation, states, config)
        assert states_json(before) == states_json(after), 'complete learned inference after reload'
        S.release('states_release', before)
        S.release('states_release', after)
    S.release('model_release', learned)
    S.release('model_release', recovered)
    S.release('training_result_release', result)
S.release('model_release', initialized)
S.release('model_release', model)
S.release('training_files_release', files)
for observation, states in samples:
    S.release('observation_release', observation)
    S.release('states_release', states)
result = dict(files=len(samples), feature_values=value_count, maximum_feature_error=maximum_error,
              original_c_hsmm_states=state_count, training_modes=2, iterations=len(reports),
              mean_log_likelihoods=reports, independent_libraries=3,
              cross_library_owner_pointers=False)
args.output.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
print(json.dumps(result))
