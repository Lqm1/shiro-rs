/* Actual optimized C model workflow and original-wire verification. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ShiroRsBytes *load(const char *path) {
    FILE *file = fopen(path, "rb");
    assert(file && fseek(file, 0, SEEK_END) == 0);
    long length = ftell(file);
    assert(length >= 0 && fseek(file, 0, SEEK_SET) == 0);
    unsigned char *data = malloc((size_t)length + 1);
    assert(data && fread(data, 1, (size_t)length, file) == (size_t)length);
    assert(fclose(file) == 0);
    ShiroRsBytes *result = NULL;
    assert(shiro_rs_bytes_create(data, (size_t)length, &result) == 0);
    free(data);
    return result;
}

static void equal(ShiroRsBytes *left, ShiroRsBytes *right) {
    size_t a = 0, b = 0;
    assert(shiro_rs_bytes_length(left, &a) == 0);
    assert(shiro_rs_bytes_length(right, &b) == 0 && a == b);
    unsigned char *x = malloc(a + 1), *y = malloc(b + 1);
    assert(x && y);
    assert(shiro_rs_bytes_copy(left, 0, x, a) == 0);
    assert(shiro_rs_bytes_copy(right, 0, y, b) == 0);
    assert(memcmp(x, y, a) == 0);
    free(x); free(y);
}

int main(void) {
    assert(shiro_rs_abi_version() == 1);
    ShiroRsBytes *definition = load("tests/fixtures/modeldef.json");
    ShiroRsBytes *expected = load("tests/fixtures/empty-c.hsmm"), *wire = NULL;
    ShiroRsModel *model = NULL, *clone = NULL;
    assert(shiro_rs_model_from_definition(definition, &model) == 0);
    assert(shiro_rs_bytes_release(&definition) == 0);
    assert(shiro_rs_model_clone(model, &clone) == 0);
    assert(shiro_rs_model_release(&model) == 0 && !model);
    assert(shiro_rs_model_release(&model) == 0);
    assert(shiro_rs_model_write_bytes(clone, 0, &wire) == 0);
    equal(expected, wire);
    assert(shiro_rs_bytes_release(&wire) == 0);
    assert(shiro_rs_bytes_release(&expected) == 0);
    assert(shiro_rs_model_release(&clone) == 0);
    const char *paths[] = {
        "tests/fixtures/empty-c.hsmm",
        "tests/fixtures/cmu-arctic-all-speakers.hsmm",
        "tests/fixtures/init-c-multi.hsmm",
        "tests/fixtures/untie-c-weighted-input.hsmm",
        "tests/fixtures/rest-c-isolated-daem.hsmm",
        "tests/fixtures/utterances-c-trained.hsmm"
    };
    for (size_t i = 0; i < sizeof(paths)/sizeof(paths[0]); ++i) {
        expected = load(paths[i]);
        assert(shiro_rs_model_read_bytes(expected, 16 * 1024 * 1024, &model) == 0);
        assert(shiro_rs_model_write_bytes(model, i == 1 ? 1 : 0, &wire) == 0);
        equal(expected, wire);
        ShiroRsModel *sentinel = model;
        assert(shiro_rs_model_read_bytes(expected, 0, &sentinel) == 3 && sentinel == model);
        assert(shiro_rs_model_clone(NULL, &sentinel) == 1 && sentinel == model);
        ShiroRsBytes *retained = wire;
        assert(shiro_rs_model_write_bytes(model, 2, &retained) == 2 && retained == wire);
        assert(shiro_rs_model_write_bytes(model, 0, NULL) == 1);
        assert(shiro_rs_model_read_bytes(expected, 16 * 1024 * 1024, NULL) == 1);
        assert(shiro_rs_model_from_definition(NULL, &sentinel) == 1 && sentinel == model);
        assert(shiro_rs_model_clone(model, NULL) == 1);
        assert(shiro_rs_bytes_release(&expected) == 0);
        assert(shiro_rs_model_release(&model) == 0);
        assert(shiro_rs_model_read_bytes(wire, 16 * 1024 * 1024, &model) == 0);
        assert(shiro_rs_bytes_release(&wire) == 0);
        assert(shiro_rs_model_release(&model) == 0);
    }
    assert(shiro_rs_model_release(NULL) == 1);
    puts("SHIRO models C: all5 exports, original definition/model bytes, both schemas and owner/error transactions passed");
    return 0;
}
