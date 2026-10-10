/* Actual C complete feature API and independently recorded original C cases. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint32_t integer(FILE *file) {
    unsigned char b[4]; assert(fread(b, 1, 4, file) == 4);
    return (uint32_t)b[0] | (uint32_t)b[1] << 8 | (uint32_t)b[2] << 16 | (uint32_t)b[3] << 24;
}
static float scalar(FILE *file) { uint32_t bits = integer(file); float output; memcpy(&output, &bits, 4); return output; }
static ShiroRsArrayF32 *array(const float *values, size_t count) {
    ShiroRsArrayF32 *owner = NULL; assert(shiro_rs_array_f32_create(values, count, &owner) == 0); return owner;
}
static float *copied(ShiroRsArrayF32 *owner, size_t *count) {
    assert(shiro_rs_array_f32_length(owner, count) == 0);
    float *values = malloc((*count + 1) * sizeof(*values)); assert(values);
    assert(shiro_rs_array_f32_copy(owner, 0, values, *count) == 0); return values;
}
int main(void) {
    _Alignas(8) ShiroRsFeatureOptions options;
    assert(shiro_rs_feature_options_default(&options) == 0);
    assert(options.kind == 0 && options.order == 12 && options.channels == 36 && options.frame_length == 1024);
    assert(options.hop == 256 && options.sample_rate_hz == 32000 && options.minimum_bandwidth_hz == 400 && options.warp == 1);
    assert(options.include_dc == 0 && options.energy == 0 && options.delta == 0 && options.acceleration == 0);
    FILE *file = fopen("tests/fixtures/c-xxcc.bin", "rb"); assert(file);
    char magic[4]; assert(fread(magic, 1, 4, file) == 4 && memcmp(magic, "XCC1", 4) == 0 && integer(file) == 72);
    size_t total = 0;
    for (size_t record = 0; record < 72; ++record) {
        assert(shiro_rs_feature_options_default(&options) == 0);
        options.kind = integer(file); options.energy = integer(file); uint32_t flags = integer(file);
        options.frame_length = integer(file); options.hop = scalar(file); size_t count = integer(file);
        float *input = malloc((count + 1) * sizeof(*input)); assert(input);
        for (size_t i = 0; i < count; ++i) input[i] = scalar(file);
        size_t frames = integer(file), columns = integer(file);
        options.channels = 12; options.sample_rate_hz = 16000; options.warp = 0.85f;
        options.include_dc = flags & 1; options.delta = (flags >> 1) & 1; options.acceleration = (flags >> 2) & 1;
        ShiroRsArrayF32 *signal = array(input, count); free(input);
        ShiroRsFeatures *result = NULL, *clone = NULL;
        assert(shiro_rs_features_extract(signal, &options, &result) == 0 && shiro_rs_array_f32_release(&signal) == 0);
        assert(shiro_rs_features_clone(result, &clone) == 0 && shiro_rs_features_release(&result) == 0);
        _Alignas(8) ShiroRsFeatureInfo info; ShiroRsArrayF32 *values = NULL;
        assert(shiro_rs_features_get_info(clone, &info) == 0 && info.frames == frames && info.columns == columns);
        assert(shiro_rs_features_get_values(clone, &values) == 0 && shiro_rs_features_release(&clone) == 0);
        float *actual = copied(values, &count); assert(count == frames * columns);
        for (size_t i = 0; i < count; ++i) {
            float expected = scalar(file);
            if (isnan(expected)) assert(isnan(actual[i]));
            else if (isinf(expected)) assert(actual[i] == expected);
            else assert(fabs((double)actual[i] - (double)expected) / fmax(fabs((double)expected), 1) < 2e-5);
            ++total;
        }
        free(actual); assert(shiro_rs_array_f32_release(&values) == 0);
    }
    assert(fgetc(file) == EOF && fclose(file) == 0);
    const uint32_t bits[] = {0x80000000, 1, 0x7fc12345, 0x7f800000}; float input[4]; memcpy(input, bits, sizeof(input));
    ShiroRsArrayF32 *signal = array(input, 4), *values = NULL;
    ShiroRsFeatures *result = NULL, *clone = NULL;
    assert(shiro_rs_features_create(UINTPTR_MAX, UINTPTR_MAX - 1, signal, &result) == 0);
    assert(shiro_rs_features_clone(result, &clone) == 0 && shiro_rs_features_release(&result) == 0 && shiro_rs_array_f32_release(&signal) == 0);
    _Alignas(8) ShiroRsFeatureInfo info;
    assert(shiro_rs_features_get_info(clone, &info) == 0 && info.frames == UINTPTR_MAX && info.columns == UINTPTR_MAX - 1);
    assert(shiro_rs_features_get_values(clone, &values) == 0);
    ShiroRsFeatures *retained = clone; signal = array(NULL, 0);
    for (size_t field = 0; field < 5; ++field) {
        assert(shiro_rs_feature_options_default(&options) == 0);
        switch (field) {case 0: options.kind = 3; break; case 1: options.energy = 3; break; case 2: options.include_dc = 2; break; case 3: options.delta = 2; break; default: options.acceleration = 2;}
        assert(shiro_rs_features_extract(signal, &options, &retained) == 2 && retained == clone);
    }
    assert(shiro_rs_feature_options_default(&options) == 0); options.hop = 0;
    assert(shiro_rs_features_extract(signal, &options, &retained) == 3 && retained == clone);
    assert(shiro_rs_features_extract(signal, NULL, &retained) == 1 && retained == clone);
    assert(shiro_rs_features_create(0, 0, NULL, &retained) == 1 && retained == clone);
    assert(shiro_rs_features_get_info(NULL, &info) == 1 && info.frames == UINTPTR_MAX);
    assert(shiro_rs_features_get_values(clone, NULL) == 1 && shiro_rs_features_clone(clone, NULL) == 1 && shiro_rs_feature_options_default(NULL) == 1 && shiro_rs_features_release(NULL) == 1);
    assert(shiro_rs_features_release(&clone) == 0); size_t count = 0; float *actual = copied(values, &count);
    assert(count == 4 && memcmp(actual, bits, sizeof(bits)) == 0); free(actual);
    assert(shiro_rs_array_f32_release(&values) == 0 && shiro_rs_features_release(&clone) == 0);
    assert(shiro_rs_feature_options_default(&options) == 0);
    options.hop = 17.5f; options.include_dc = 1; options.energy = 1; options.delta = 1; options.acceleration = 1;
    assert(shiro_rs_features_extract(signal, &options, &result) == 0);
    assert(shiro_rs_features_get_info(result, &info) == 0 && info.frames == 0 && info.columns == 42);
    assert(shiro_rs_features_get_values(result, &values) == 0); actual = copied(values, &count); assert(count == 0); free(actual);
    assert(shiro_rs_array_f32_release(&values) == 0 && shiro_rs_features_release(&result) == 0 && shiro_rs_array_f32_release(&signal) == 0);
    printf("SHIRO features C: all7 exports, all72 original C cases/%zu values, full fields/bits and ownership passed\n", total); return 0;
}
