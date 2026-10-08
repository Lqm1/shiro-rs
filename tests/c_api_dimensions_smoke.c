/* Actual C complete model dimensions and target-width owned array transactions. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ShiroRsBytes *owned(const void *data, size_t count) {
    ShiroRsBytes *output = NULL;
    assert(shiro_rs_bytes_create(data, count, &output) == 0); return output;
}
static ShiroRsBytes *fixture(const char *name) {
    char path[160]; snprintf(path, sizeof(path), "tests/fixtures/%s", name);
    FILE *file = fopen(path, "rb"); assert(file && fseek(file, 0, SEEK_END) == 0);
    long count = ftell(file); assert(count >= 0 && fseek(file, 0, SEEK_SET) == 0);
    void *data = malloc((size_t)count + 1); assert(data);
    assert(fread(data, 1, (size_t)count, file) == (size_t)count && fclose(file) == 0);
    ShiroRsBytes *output = owned(data, (size_t)count); free(data); return output;
}
static void equal(ShiroRsArrayUsize *owner, const uintptr_t *expected, size_t count) {
    size_t n = 99; assert(shiro_rs_array_usize_length(owner, &n) == 0 && n == count);
    uintptr_t *values = malloc((count + 1) * sizeof(*values)); assert(values);
    assert(shiro_rs_array_usize_copy(owner, 0, values, count) == 0);
    assert(memcmp(values, expected, count * sizeof(*values)) == 0); free(values);
}
int main(void) {
    const char *names[] = {"empty-c.hsmm", "init-c-multi.hsmm"};
    const uintptr_t original[] = {2, 1};
    for (size_t i = 0; i < 2; ++i) {
        ShiroRsBytes *wire = fixture(names[i]); ShiroRsModel *model = NULL;
        assert(shiro_rs_model_read_bytes(wire, 16 * 1024 * 1024, &model) == 0 && shiro_rs_bytes_release(&wire) == 0);
        ShiroRsArrayUsize *values = NULL, *clone = NULL;
        assert(shiro_rs_model_dimensions(model, &values) == 0 && shiro_rs_array_usize_clone(values, &clone) == 0);
        assert(shiro_rs_array_usize_release(&values) == 0 && shiro_rs_model_dimensions(model, &values) == 0);
        assert(shiro_rs_model_release(&model) == 0); equal(values, original, 2); equal(clone, original, 2);
        assert(shiro_rs_array_usize_release(&values) == 0 && shiro_rs_array_usize_release(&clone) == 0);
    }
    const char *json = "{\"ndurstate\":1,\"streamdef\":[{\"nstate\":1,\"ndim\":7},{\"nstate\":2,\"ndim\":1},{\"nstate\":1,\"ndim\":19}]}";
    ShiroRsBytes *definition = owned(json, strlen(json)); ShiroRsModel *model = NULL;
    ShiroRsArrayUsize *values = NULL;
    assert(shiro_rs_model_from_definition(definition, &model) == 0 && shiro_rs_bytes_release(&definition) == 0);
    assert(shiro_rs_model_dimensions(model, &values) == 0 && shiro_rs_model_dimensions(model, NULL) == 1);
    assert(shiro_rs_model_release(&model) == 0); const uintptr_t widths[] = {7, 1, 19}; equal(values, widths, 3);
    assert(shiro_rs_array_usize_release(&values) == 0);
    uintptr_t input[] = {0, UINTPTR_MAX, UINTPTR_MAX - 1, 1};
    const uintptr_t expected[] = {0, UINTPTR_MAX, UINTPTR_MAX - 1, 1};
    assert(shiro_rs_array_usize_create(input, 4, &values) == 0); input[1] = 42; equal(values, expected, 4);
    uintptr_t output[] = {73, 73, 73, 73};
    assert(shiro_rs_array_usize_copy(values, 1, output, 2) == 0 && output[0] == UINTPTR_MAX && output[1] == UINTPTR_MAX - 1 && output[2] == 73 && output[3] == 73);
    const uintptr_t offsets[] = {3, UINTPTR_MAX, 5}, counts[] = {2, 1, 0};
    for (size_t i = 0; i < 3; ++i) {
        for (size_t j = 0; j < 4; ++j) output[j] = 73;
        assert(shiro_rs_array_usize_copy(values, offsets[i], output, counts[i]) == 2);
        for (size_t j = 0; j < 4; ++j) assert(output[j] == 73);
    }
    assert(shiro_rs_array_usize_copy(values, 4, NULL, 0) == 0 && shiro_rs_array_usize_copy(values, 0, NULL, 1) == 1);
    ShiroRsArrayUsize *retained = values;
    assert(shiro_rs_array_usize_create(NULL, 1, &retained) == 1 && retained == values);
    assert(shiro_rs_array_usize_create(NULL, UINTPTR_MAX, &retained) == 2 && retained == values);
    assert(shiro_rs_array_usize_create((const uintptr_t *)(uintptr_t)1, 1, &retained) == 1 && retained == values);
    assert(shiro_rs_model_dimensions(NULL, &retained) == 1 && retained == values);
    assert(shiro_rs_array_usize_clone(values, NULL) == 1 && shiro_rs_array_usize_length(values, NULL) == 1 && shiro_rs_array_usize_release(NULL) == 1);
    ShiroRsArrayUsize *clone = NULL;
    assert(shiro_rs_array_usize_clone(values, &clone) == 0 && shiro_rs_array_usize_release(&values) == 0);
    equal(clone, expected, 4); assert(shiro_rs_array_usize_release(&clone) == 0);
    assert(shiro_rs_array_usize_create(NULL, 0, &values) == 0); size_t count = 99;
    assert(shiro_rs_array_usize_length(values, &count) == 0 && count == 0);
    assert(shiro_rs_array_usize_copy(values, 0, NULL, 0) == 0 && shiro_rs_array_usize_release(&values) == 0 && shiro_rs_array_usize_release(&values) == 0);
    puts("SHIRO dimensions C: all6 exports, full stream order/target-width integers and ownership passed"); return 0;
}
