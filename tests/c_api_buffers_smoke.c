#ifdef NDEBUG
#undef NDEBUG
#endif
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "shiro_rs.h"

int main(void) {
    assert(shiro_rs_abi_version()==1);
    uint8_t input[4]={1,2,3,4},result[4]={0};
    ShiroRsBytes *bytes=NULL,*copy=NULL;
    assert(shiro_rs_bytes_create(input,4,&bytes)==0);
    input[0]=99;
    uintptr_t length=99;
    assert(shiro_rs_bytes_length(bytes,&length)==0 && length==4);
    assert(shiro_rs_bytes_clone(bytes,&copy)==0);
    assert(shiro_rs_bytes_release(&bytes)==0 && bytes==NULL);
    assert(shiro_rs_bytes_copy(copy,0,result,4)==0 && result[0]==1);
    result[0]=71;
    assert(shiro_rs_bytes_copy(copy,3,result,2)==2 && result[0]==71);
    assert(shiro_rs_bytes_copy(copy,UINTPTR_MAX,result,2)==2 && result[0]==71);
    assert(shiro_rs_bytes_copy(copy,4,NULL,0)==0 && shiro_rs_bytes_copy(copy,5,NULL,0)==2);
    ShiroRsBytes *sentinel=copy;
    assert(shiro_rs_bytes_create(NULL,1,&sentinel)==1 && sentinel==copy);
    assert(shiro_rs_bytes_create(NULL,UINTPTR_MAX,&sentinel)==2 && sentinel==copy);
    assert(shiro_rs_bytes_length(copy,NULL)==1 && shiro_rs_bytes_clone(copy,NULL)==1);
    assert(shiro_rs_bytes_release(&copy)==0 && shiro_rs_bytes_release(&copy)==0);
    assert(shiro_rs_bytes_release(NULL)==1);
    const uint32_t patterns[8]={0,0x80000000,1,0x80000001,0x3f800001,0x7f800000,0xff800000,0x7fc01234};
    float values[8],actual[8];uint8_t expected[32];
    memcpy(values,patterns,sizeof values);
    for(size_t i=0;i<8;i++) for(size_t byte=0;byte<4;byte++) expected[4*i+byte]=(uint8_t)(patterns[i]>>(8*byte));
    ShiroRsArrayF32 *array=NULL,*cloned=NULL,*decoded=NULL;
    assert(shiro_rs_array_f32_create(values,8,&array)==0);
    memset(values,0,sizeof values);
    assert(shiro_rs_array_f32_length(array,&length)==0 && length==8);
    assert(shiro_rs_array_f32_clone(array,&cloned)==0);
    assert(shiro_rs_array_f32_release(&array)==0 && array==NULL);
    assert(shiro_rs_array_f32_copy(cloned,0,actual,8)==0 && !memcmp(actual,patterns,sizeof actual));
    actual[0]=71;
    assert(shiro_rs_array_f32_copy(cloned,7,actual,2)==2 && actual[0]==71);
    assert(shiro_rs_array_f32_copy(cloned,8,NULL,0)==0);
    assert(shiro_rs_array_f32_copy(cloned,9,NULL,0)==2);
    assert(shiro_rs_array_f32_copy(cloned,0,NULL,1)==1);
    assert(shiro_rs_rawfloat_write_bytes(cloned,&bytes)==0);
    assert(shiro_rs_array_f32_release(&cloned)==0);
    uint8_t wire[32];
    assert(shiro_rs_bytes_copy(bytes,0,wire,32)==0 && !memcmp(wire,expected,32));
    assert(shiro_rs_rawfloat_read_bytes(wire,32,8,&decoded)==0);
    assert(shiro_rs_array_f32_copy(decoded,0,actual,8)==0 && !memcmp(actual,patterns,sizeof actual));
    ShiroRsArrayF32 *array_sentinel=decoded;
    assert(shiro_rs_rawfloat_read_bytes(wire,31,8,&array_sentinel)==3 && array_sentinel==decoded);
    assert(shiro_rs_rawfloat_read_bytes(wire,32,7,&array_sentinel)==3 && array_sentinel==decoded);
    assert(shiro_rs_rawfloat_read_bytes(NULL,1,8,&array_sentinel)==1);
    assert(shiro_rs_rawfloat_read_bytes(NULL,UINTPTR_MAX,8,&array_sentinel)==2);
    assert(shiro_rs_rawfloat_write_bytes(decoded,NULL)==1);
    assert(shiro_rs_array_f32_length(decoded,NULL)==1 && shiro_rs_array_f32_clone(decoded,NULL)==1);
    assert(shiro_rs_array_f32_release(&decoded)==0 && shiro_rs_array_f32_release(&decoded)==0);
    assert(shiro_rs_array_f32_release(NULL)==1);
    assert(shiro_rs_bytes_release(&bytes)==0);
    assert(shiro_rs_array_f32_create(NULL,0,&array)==0);
    assert(shiro_rs_array_f32_length(array,&length)==0 && length==0);
    assert(shiro_rs_array_f32_release(&array)==0);
    assert(shiro_rs_rawfloat_read_bytes(NULL,0,0,&array)==0);
    assert(shiro_rs_array_f32_length(array,&length)==0 && length==0);
    assert(shiro_rs_array_f32_release(&array)==0);
    puts("SHIRO buffers C: all13 exports, binary32 bits, original rawfloat wire and owner/error transactions passed");
    return 0;
}
