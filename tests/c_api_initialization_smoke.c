/* Actual C complete initialization/dataset ABI and original model verification. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <process.h>
#define TASK_PID _getpid
#define TASK_OS "windows"
#else
#include <unistd.h>
#define TASK_PID getpid
#define TASK_OS "linux"
#endif

static char *load(const char *path, size_t *size) {
    FILE *file = fopen(path, "rb"); assert(file && fseek(file, 0, SEEK_END) == 0);
    long count = ftell(file); assert(count >= 0 && fseek(file, 0, SEEK_SET) == 0);
    char *result = malloc((size_t)count + 1); assert(result);
    assert(fread(result, 1, (size_t)count, file) == (size_t)count); assert(fclose(file) == 0);
    result[count] = 0; *size = (size_t)count; return result;
}
static ShiroRsBytes *owned(const void *data, size_t size) {
    ShiroRsBytes *result = NULL; assert(shiro_rs_bytes_create(data, size, &result) == 0); return result;
}
static char *copied(ShiroRsBytes *owner) {
    size_t size = 0; assert(shiro_rs_bytes_length(owner, &size) == 0);
    char *result = malloc(size + 1); assert(result);
    assert(shiro_rs_bytes_copy(owner, 0, (unsigned char *)result, size) == 0); result[size] = 0; return result;
}
/* Lexically extract the complete states array; quoted brackets/escapes are data. */
static char *states_array(const char *document) {
    const char *start = strstr(document, "\"states\""); assert(start);
    start = strchr(start, '['); assert(start);
    int depth = 0, string = 0, escape = 0; const char *end = start;
    for (; *end; ++end) {
        if (string) { if (escape) escape = 0; else if (*end == '\\') escape = 1; else if (*end == '"') string = 0; }
        else if (*end == '"') string = 1;
        else if (*end == '[') ++depth;
        else if (*end == ']' && --depth == 0) { ++end; break; }
    }
    assert(depth == 0 && !string); size_t size = (size_t)(end - start);
    char *result = malloc(size + 1); assert(result); memcpy(result, start, size); result[size] = 0; return result;
}
static ShiroRsStates *read_states(const char *json) {
    ShiroRsBytes *bytes = owned(json, strlen(json)); ShiroRsStates *result = NULL;
    assert(shiro_rs_states_read_json(bytes, &result) == 0); assert(shiro_rs_bytes_release(&bytes) == 0); return result;
}
static char *states_json(ShiroRsStates *states) {
    ShiroRsBytes *bytes = NULL; assert(shiro_rs_states_write_json(states, &bytes) == 0);
    char *result = copied(bytes); assert(shiro_rs_bytes_release(&bytes) == 0); return result;
}

static void model_equal(ShiroRsModel *model, const char *path) {
    size_t expected_size; char *expected=load(path,&expected_size);
    ShiroRsBytes *wire=NULL; assert(shiro_rs_model_write_bytes(model,0,&wire)==0);
    size_t size=0; assert(shiro_rs_bytes_length(wire,&size)==0 && size==expected_size);
    char *actual=copied(wire); assert(memcmp(actual,expected,size)==0);
    free(actual);free(expected);assert(shiro_rs_bytes_release(&wire)==0);
}
int main(void) {
    size_t size; char *source=load("tests/fixtures/init-definition.json",&size);
    ShiroRsBytes *bytes=owned(source,size);free(source);ShiroRsModel *model=NULL;
    assert(shiro_rs_model_from_definition(bytes,&model)==0);assert(shiro_rs_bytes_release(&bytes)==0);
    source=load("tests/fixtures/init-input.bin",&size);
    char path[128];snprintf(path,sizeof(path),".shiro-c-api-init-%s-%ld.f",TASK_OS,(long)TASK_PID());
    FILE *file=fopen(path,"wb");assert(file&&fwrite(source,1,size,file)==size&&fclose(file)==0);
    bytes=owned(source,size);free(source);ShiroRsObservation *observation=NULL;
    assert(shiro_rs_observation_from_model_rawfloat(bytes,model,12,&observation)==0);assert(shiro_rs_bytes_release(&bytes)==0);
    source=load("tests/fixtures/init-segmentation.json",&size);char *array=states_array(source);free(source);
    ShiroRsStates *states=read_states(array);ShiroRsDataset *data=NULL,*clone=NULL;
    const ShiroRsObservation *observations[]={observation};const ShiroRsStates *sequences[]={states};
    assert(shiro_rs_dataset_create(model,observations,sequences,1,&data)==0);
    assert(shiro_rs_dataset_clone(data,&clone)==0);assert(shiro_rs_dataset_release(&data)==0&&!data);
    _Alignas(8) ShiroRsInitializationOptions config;assert(shiro_rs_initialization_options_default(&config)==0);
    assert(config.flat_start==0&&config.globally_tied==0&&config.variance_floor_ratio==0.1f);
    const char *fixtures[]={"init-c-aligned.hsmm","init-c-flat.hsmm","init-c-tied.hsmm","init-c-flat-tied.hsmm"};
    for(size_t i=0;i<4;++i){
        config.flat_start=(uint32_t)(i%2);config.globally_tied=(uint32_t)(i/2);config.variance_floor_ratio=i==3?0.4f:0.1f;
        ShiroRsModel *initialized=NULL;assert(shiro_rs_initialize(model,clone,&config,&initialized)==0);
        char fixture[160];snprintf(fixture,sizeof(fixture),"tests/fixtures/%s",fixtures[i]);model_equal(initialized,fixture);
        assert(shiro_rs_model_release(&initialized)==0);
    }
    size_t length=99;assert(shiro_rs_dataset_length(clone,&length)==0&&length==1);
    ShiroRsObservation *snapshot=NULL;ShiroRsBytes *segmentation=NULL,*expected_segmentation=NULL;
    assert(shiro_rs_dataset_get_observation(clone,0,&snapshot)==0);
    assert(shiro_rs_dataset_get_segmentation_bytes(clone,0,&segmentation)==0);
    assert(shiro_rs_states_segmentation_bytes(states,model,&expected_segmentation)==0);
    char *a=copied(segmentation),*b=copied(expected_segmentation);size_t n=0,m=0;
    assert(shiro_rs_bytes_length(segmentation,&n)==0&&shiro_rs_bytes_length(expected_segmentation,&m)==0&&n==m&&memcmp(a,b,n)==0);
    free(a);free(b);assert(shiro_rs_bytes_release(&segmentation)==0);assert(shiro_rs_bytes_release(&expected_segmentation)==0);
    assert(shiro_rs_dataset_release(&clone)==0);assert(shiro_rs_observation_release(&observation)==0);assert(shiro_rs_states_release(&states)==0);
    ShiroRsBytes *observation_wire=NULL;assert(shiro_rs_observation_write_bytes(snapshot,&observation_wire)==0);
    assert(shiro_rs_bytes_release(&observation_wire)==0);assert(shiro_rs_observation_release(&snapshot)==0);
    size_t capacity=strlen(array)+strlen(path)+128;char *document=malloc(capacity);assert(document);
    snprintf(document,capacity,"{\"file_list\":[{\"filename\":\"%s\",\"states\":%s}]}",path,array);
    bytes=owned(document,strlen(document));assert(shiro_rs_dataset_read_document(model,bytes,12,&data)==0);
    ShiroRsDataset *retained=data;assert(shiro_rs_dataset_read_document(model,bytes,11,&retained)==3&&retained==data);
    config.flat_start=2;ShiroRsModel *retained_model=model;assert(shiro_rs_initialize(model,data,&config,&retained_model)==2&&retained_model==model);
    assert(shiro_rs_dataset_create(model,NULL,NULL,SIZE_MAX,&retained)==2&&retained==data);
    assert(shiro_rs_dataset_get_observation(data,1,&snapshot)==2&&!snapshot);
    assert(shiro_rs_dataset_get_segmentation_bytes(data,1,&segmentation)==2&&!segmentation);
    assert(shiro_rs_initialization_options_default(NULL)==1&&shiro_rs_dataset_release(NULL)==1);
    assert(shiro_rs_dataset_clone(data,NULL)==1&&shiro_rs_dataset_length(data,NULL)==1);
    assert(shiro_rs_initialize(model,data,NULL,&retained_model)==1&&retained_model==model);
    assert(shiro_rs_dataset_release(&data)==0&&shiro_rs_dataset_release(&data)==0);
    assert(shiro_rs_bytes_release(&bytes)==0);assert(shiro_rs_model_release(&model)==0);
    free(document);free(array);assert(remove(path)==0);
    puts("SHIRO initialization C: all9 exports, four exact original C models, paired snapshots, host loading and errors passed");
    return 0;
}
