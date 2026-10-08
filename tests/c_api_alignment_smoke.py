"""Actual ctypes full settings, original C paths and host metadata verification."""
import ctypes as c
import json
from pathlib import Path
import sys
import tempfile

lib = c.CDLL(sys.argv[1])
P, N, U, B, F = c.c_void_p, c.c_size_t, c.c_uint32, c.c_uint8, c.c_float
symbols = {'alignment_options_default', 'align_states', 'align_document'}
if len(sys.argv) > 2:
    assert sys.argv[2] == '--no-c-api'
    for name in symbols:
        assert not hasattr(lib, 'shiro_rs_' + name), name
    print('SHIRO alignment ctypes: all3 symbols absent without c-api')
    sys.exit(0)
class Options(c.Structure):
    _fields_ = [('duration_mode',U),('isolated',U),('hsmm_temperature',F),('duration_weight',F),('state_radius',F),('duration_extra',N),('duration_extra_factor',F),('geometric_temperature',F),('pruning_slope',F)]
called = set()
def function(name, args):
    fn = getattr(lib, 'shiro_rs_' + name)
    fn.argtypes, fn.restype = args, U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke
create = function('bytes_create', [c.POINTER(B),N,c.POINTER(P)])
length = function('bytes_length', [P,c.POINTER(N)])
copy = function('bytes_copy', [P,N,c.POINTER(B),N])
release_bytes = function('bytes_release', [c.POINTER(P)])
model_read = function('model_read_bytes', [P,N,c.POINTER(P)])
model_release = function('model_release', [c.POINTER(P)])
obs_read = function('observation_from_model_rawfloat', [P,P,N,c.POINTER(P)])
obs_release = function('observation_release', [c.POINTER(P)])
states_read = function('states_read_json', [P,c.POINTER(P)])
states_write = function('states_write_json', [P,c.POINTER(P)])
states_release = function('states_release', [c.POINTER(P)])
defaults = function('alignment_options_default', [c.POINTER(Options)])
align = function('align_states', [P,P,P,c.POINTER(Options),c.POINTER(P)])
align_document = function('align_document', [P,P,c.POINTER(Options),c.POINTER(P)])
def owned(data):
    output = P(); assert create((B*len(data)).from_buffer_copy(data),len(data),c.byref(output)) == 0; return output
def copied(owner):
    count = N(); assert length(owner,c.byref(count)) == 0
    data = (B*count.value)(); assert copy(owner,0,data,count.value) == 0; return bytes(data)
def states_json(owner):
    output = P(); assert states_write(owner,c.byref(output)) == 0
    result = json.loads(copied(output)); assert release_bytes(c.byref(output)) == 0; return result
root = Path(__file__).parent/'fixtures'
config = Options(); assert defaults(c.byref(config)) == 0
assert [config.duration_mode,config.isolated,config.hsmm_temperature,config.duration_weight,config.state_radius,config.duration_extra,config.duration_extra_factor,config.geometric_temperature,config.pruning_slope] == [0,0,1,1,5,30,1,1,c.c_float(0.3).value]
assert defaults(None) == 1
encoded = owned((root/'init-c-aligned.hsmm').read_bytes()); model = P()
assert model_read(encoded,16*1024*1024,c.byref(model)) == 0 and release_bytes(c.byref(encoded)) == 0
raw = (root/'init-input.bin').read_bytes(); encoded = owned(raw); observation = P()
assert obs_read(encoded,model,12,c.byref(observation)) == 0 and release_bytes(c.byref(encoded)) == 0
references = ['align-c-embedded-hsmm.json','align-c-embedded-hmm.json','align-c-embedded-hsmm-pruned.json','align-c-embedded-hmm-pruned.json','align-c-isolated-hsmm.json','align-c-isolated-hmm.json','align-c-isolated-hsmm-pruned.json','align-c-isolated-hmm-pruned.json','align-c-four-hmm.json']
with tempfile.TemporaryDirectory(prefix='shiro-c-api-alignment-') as directory:
    feature = Path(directory)/'input.f'; feature.write_bytes(raw)
    for index, reference in enumerate(references):
        document = json.loads((root/('align-c-four.json' if index==8 else 'align-c-isolated.json' if index>=4 else 'align-c-embedded.json')).read_text())
        original = document['file_list'][0]['states']
        expected = json.loads((root/reference).read_text())['file_list'][0]['states']
        config = Options(); assert defaults(c.byref(config)) == 0
        config.duration_mode = 1 if index==8 else index%2; config.isolated = int(index>=4 and index!=8)
        if index != 8: config.pruning_slope = 0.8
        if index<8 and index%4>=2: config.state_radius,config.duration_extra,config.pruning_slope = 2,8,0.5
        encoded = owned(json.dumps(original).encode()); states,aligned = P(),P()
        assert states_read(encoded,c.byref(states)) == 0 and release_bytes(c.byref(encoded)) == 0
        assert align(model,observation,states,c.byref(config),c.byref(aligned)) == 0
        assert states_json(aligned) == expected and states_json(states) == original
        assert states_release(c.byref(aligned)) == 0
        document['file_list'][0]['filename'] = str(feature)
        document['file_list'][0]['file_metadata'] = {'keep':True}; document['document_metadata'] = {'keep':[1,None]}
        source = json.dumps(document).encode(); encoded = owned(source); output = P()
        assert align_document(model,encoded,c.byref(config),c.byref(output)) == 0
        result = json.loads(copied(output)); expected_document = json.loads(source)
        expected_document['file_list'][0]['states'] = expected
        assert result == expected_document and copied(encoded) == source
        config.isolated = 2; retained = P(states.value); retained_bytes = P(output.value)
        assert align(model,observation,states,c.byref(config),c.byref(retained)) == 2 and retained.value == states.value
        assert align_document(model,encoded,c.byref(config),c.byref(retained_bytes)) == 2 and retained_bytes.value == output.value
        config.isolated = int(index>=4 and index!=8)
        assert align(model,observation,states,None,c.byref(retained)) == 1 and retained.value == states.value
        assert align(model,observation,states,c.byref(config),None) == 1
        assert align_document(model,encoded,None,c.byref(retained_bytes)) == 1
        assert align_document(model,encoded,c.byref(config),None) == 1
        assert release_bytes(c.byref(output)) == 0 and release_bytes(c.byref(encoded)) == 0
        assert states_release(c.byref(states)) == 0
assert obs_release(c.byref(observation)) == 0 and model_release(c.byref(model)) == 0
assert symbols <= called
print('SHIRO alignment ctypes: all3 exports, all9 original C cases, complete descriptor defaults and host metadata passed')
