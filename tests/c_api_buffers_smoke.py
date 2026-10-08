"""Actual ctypes ownership and original rawfloat wire/bit verification."""
import ctypes as c
import struct
import sys

lib=c.CDLL(sys.argv[1])
P,N,U,B,F=c.c_void_p,c.c_size_t,c.c_uint32,c.c_uint8,c.c_float
called=set()
if len(sys.argv)>2:
    assert sys.argv[2]=='--no-c-api'
    symbols={'abi_version','rawfloat_read_bytes','rawfloat_write_bytes'}|{prefix+'_'+method for prefix in ['bytes','array_f32'] for method in ['create','length','copy','clone','release']}
    for name in symbols:
        assert not hasattr(lib,'shiro_rs_'+name),name
    print('SHIRO buffers ctypes: all13 C ABI symbols absent without c-api')
    sys.exit(0)

def function(name,args):
    fn=getattr(lib,'shiro_rs_'+name)
    fn.argtypes,fn.restype=args,U
    def invoke(*values):
        called.add(name)
        return fn(*values)
    return invoke

version=function('abi_version',[])
assert version()==1
for prefix,scalar in [('bytes',B),('array_f32',F)]:
    create=function(prefix+'_create',[c.POINTER(scalar),N,c.POINTER(P)])
    length=function(prefix+'_length',[P,c.POINTER(N)])
    copy=function(prefix+'_copy',[P,N,c.POINTER(scalar),N])
    clone=function(prefix+'_clone',[P,c.POINTER(P)])
    release=function(prefix+'_release',[c.POINTER(P)])
    original=(scalar*4)(1,2,3,4)
    owner,cloned=P(),P()
    assert create(original,4,c.byref(owner))==0
    original[0]=99
    assert clone(owner,c.byref(cloned))==0
    assert release(c.byref(owner))==0 and owner.value is None
    output=(scalar*4)()
    count=N()
    assert length(cloned,c.byref(count))==0 and count.value==4
    assert copy(cloned,0,output,4)==0 and list(output)==[1,2,3,4]
    output[0]=71
    assert copy(cloned,3,output,2)==2 and output[0]==71
    assert copy(cloned,N(-1).value,output,2)==2 and output[0]==71
    assert copy(cloned,4,None,0)==0 and copy(cloned,5,None,0)==2
    assert copy(cloned,0,None,1)==1
    sentinel=P(cloned.value)
    assert create(None,1,c.byref(sentinel))==1
    assert create(None,N(-1).value,c.byref(sentinel))==2
    assert sentinel.value==cloned.value
    assert length(cloned,None)==1 and clone(cloned,None)==1
    assert release(c.byref(cloned))==0 and release(c.byref(cloned))==0
    assert release(None)==1
    assert create(None,0,c.byref(owner))==0
    assert length(owner,c.byref(count))==0 and count.value==0
    assert release(c.byref(owner))==0

read=function('rawfloat_read_bytes',[c.POINTER(B),N,N,c.POINTER(P)])
write=function('rawfloat_write_bytes',[P,c.POINTER(P)])
array_create=function('array_f32_create',[c.POINTER(F),N,c.POINTER(P)])
array_copy=function('array_f32_copy',[P,N,c.POINTER(F),N])
array_release=function('array_f32_release',[c.POINTER(P)])
byte_copy=function('bytes_copy',[P,N,c.POINTER(B),N])
byte_release=function('bytes_release',[c.POINTER(P)])
patterns=[0,0x80000000,1,0x80000001,0x3f800001,0x7f800000,0xff800000,0x7fc01234]
wire=b''.join(struct.pack('<I',value) for value in patterns)
data=(F*8).from_buffer_copy(wire)
owner,encoded,decoded=P(),P(),P()
assert array_create(data,8,c.byref(owner))==0
assert write(owner,c.byref(encoded))==0
assert array_release(c.byref(owner))==0
actual=(B*len(wire))()
assert byte_copy(encoded,0,actual,len(wire))==0 and bytes(actual)==wire
assert read(actual,len(wire),8,c.byref(decoded))==0
values=(F*8)()
assert array_copy(decoded,0,values,8)==0 and c.string_at(c.addressof(values),len(wire))==wire
sentinel=P(decoded.value)
assert read(actual,len(wire)-1,8,c.byref(sentinel))==3
assert read(actual,len(wire),7,c.byref(sentinel))==3
assert sentinel.value==decoded.value
assert read(None,1,8,c.byref(sentinel))==1
assert read(None,N(-1).value,8,c.byref(sentinel))==2
assert write(decoded,None)==1
assert array_release(c.byref(decoded))==0 and byte_release(c.byref(encoded))==0
expected={'abi_version','rawfloat_read_bytes','rawfloat_write_bytes'}|{prefix+'_'+method for prefix in ['bytes','array_f32'] for method in ['create','length','copy','clone','release']}
assert called==expected and len(called)==13
print('SHIRO buffers ctypes: all13 exports, binary32 bits, rawfloat wire and owner/error transactions passed')
