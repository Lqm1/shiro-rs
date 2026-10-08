"""Actual ctypes paired dataset/native initializer and original C model checks."""
import ctypes as c
import json
from pathlib import Path
import sys
import tempfile

lib=c.CDLL(sys.argv[1]);P,N,U,B,F=c.c_void_p,c.c_size_t,c.c_uint32,c.c_uint8,c.c_float
symbols={'dataset_create','dataset_read_document','dataset_length','dataset_get_observation','dataset_get_segmentation_bytes','dataset_clone','dataset_release','initialization_options_default','initialize'}
if len(sys.argv)>2:
    assert sys.argv[2]=='--no-c-api'
    for name in symbols: assert not hasattr(lib,'shiro_rs_'+name),name
    print('SHIRO initialization ctypes: all9 symbols absent without c-api');sys.exit(0)
class Options(c.Structure):
    _fields_=[('flat_start',U),('globally_tied',U),('variance_floor_ratio',F)]
called=set()
def function(name,args):
    fn=getattr(lib,'shiro_rs_'+name);fn.argtypes,fn.restype=args,U
    def invoke(*values): called.add(name);return fn(*values)
    return invoke
create=function('bytes_create',[c.POINTER(B),N,c.POINTER(P)])
length=function('bytes_length',[P,c.POINTER(N)])
copy=function('bytes_copy',[P,N,c.POINTER(B),N])
release_bytes=function('bytes_release',[c.POINTER(P)])
model_create=function('model_from_definition',[P,c.POINTER(P)])
model_write=function('model_write_bytes',[P,U,c.POINTER(P)])
model_release=function('model_release',[c.POINTER(P)])
obs_read=function('observation_from_model_rawfloat',[P,P,N,c.POINTER(P)])
obs_write=function('observation_write_bytes',[P,c.POINTER(P)])
obs_release=function('observation_release',[c.POINTER(P)])
states_read=function('states_read_json',[P,c.POINTER(P)])
states_seg=function('states_segmentation_bytes',[P,P,c.POINTER(P)])
states_release=function('states_release',[c.POINTER(P)])
data_create=function('dataset_create',[P,c.POINTER(P),c.POINTER(P),N,c.POINTER(P)])
data_read=function('dataset_read_document',[P,P,N,c.POINTER(P)])
data_length=function('dataset_length',[P,c.POINTER(N)])
data_observation=function('dataset_get_observation',[P,N,c.POINTER(P)])
data_segmentation=function('dataset_get_segmentation_bytes',[P,N,c.POINTER(P)])
data_clone=function('dataset_clone',[P,c.POINTER(P)])
data_release=function('dataset_release',[c.POINTER(P)])
defaults=function('initialization_options_default',[c.POINTER(Options)])
initialize=function('initialize',[P,P,c.POINTER(Options),c.POINTER(P)])
def owned(data):
    output=P();assert create((B*len(data)).from_buffer_copy(data),len(data),c.byref(output))==0;return output
def copied(owner):
    count=N();assert length(owner,c.byref(count))==0
    data=(B*count.value)();assert copy(owner,0,data,count.value)==0;return bytes(data)
root=Path(__file__).parent/'fixtures'
definition=owned((root/'init-definition.json').read_bytes());model=P()
assert model_create(definition,c.byref(model))==0 and release_bytes(c.byref(definition))==0
raw=(root/'init-input.bin').read_bytes();encoded=owned(raw);observation=P()
assert obs_read(encoded,model,12,c.byref(observation))==0 and release_bytes(c.byref(encoded))==0
document=json.loads((root/'init-segmentation.json').read_text());encoded=owned(json.dumps(document['file_list'][0]['states']).encode());states=P()
assert states_read(encoded,c.byref(states))==0 and release_bytes(c.byref(encoded))==0
data,cloned=P(),P()
assert data_create(model,(P*1)(observation.value),(P*1)(states.value),1,c.byref(data))==0
assert data_clone(data,c.byref(cloned))==0 and data_release(c.byref(data))==0
config=Options();assert defaults(c.byref(config))==0
assert [config.flat_start,config.globally_tied,config.variance_floor_ratio]==[0,0,F(0.1).value]
for i,fixture in enumerate(['init-c-aligned.hsmm','init-c-flat.hsmm','init-c-tied.hsmm','init-c-flat-tied.hsmm']):
    config.flat_start,config.globally_tied,config.variance_floor_ratio=i%2,i//2,0.4 if i==3 else 0.1
    initialized,wire=P(),P();assert initialize(model,cloned,c.byref(config),c.byref(initialized))==0
    assert model_write(initialized,0,c.byref(wire))==0 and copied(wire)==(root/fixture).read_bytes()
    assert release_bytes(c.byref(wire))==0 and model_release(c.byref(initialized))==0
count=N();assert data_length(cloned,c.byref(count))==0 and count.value==1
snapshot,seg,expected_seg=P(),P(),P()
assert data_observation(cloned,0,c.byref(snapshot))==0 and data_segmentation(cloned,0,c.byref(seg))==0
assert states_seg(states,model,c.byref(expected_seg))==0 and copied(seg)==copied(expected_seg)
assert release_bytes(c.byref(seg))==0 and release_bytes(c.byref(expected_seg))==0
assert data_release(c.byref(cloned))==0 and obs_release(c.byref(observation))==0 and states_release(c.byref(states))==0
wire=P();assert obs_write(snapshot,c.byref(wire))==0
assert release_bytes(c.byref(wire))==0 and obs_release(c.byref(snapshot))==0
with tempfile.TemporaryDirectory(prefix='shiro-c-api-initialization-') as directory:
    path=Path(directory)/'input.f';path.write_bytes(raw);document['file_list'][0]['filename']=str(path)
    original=json.dumps(document).encode();encoded=owned(original)
    assert data_read(model,encoded,12,c.byref(data))==0 and copied(encoded)==original
    retained=P(data.value);assert data_read(model,encoded,11,c.byref(retained))==3 and retained.value==data.value
    config.flat_start=2;retained_model=P(model.value)
    assert initialize(model,data,c.byref(config),c.byref(retained_model))==2 and retained_model.value==model.value
    assert data_create(model,None,None,N(-1).value,c.byref(retained))==2 and retained.value==data.value
    assert data_observation(data,1,c.byref(snapshot))==2 and snapshot.value is None
    assert data_segmentation(data,1,c.byref(seg))==2 and seg.value is None
    assert defaults(None)==1 and data_release(None)==1 and data_clone(data,None)==1 and data_length(data,None)==1
    assert initialize(model,data,None,c.byref(retained_model))==1 and retained_model.value==model.value
    assert data_release(c.byref(data))==0 and data_release(c.byref(data))==0 and release_bytes(c.byref(encoded))==0
assert model_release(c.byref(model))==0
assert symbols<=called
print('SHIRO initialization ctypes: all9 exports, four exact original C models, paired snapshots, host loading and errors passed')
