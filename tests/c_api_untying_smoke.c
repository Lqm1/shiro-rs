/* Actual C complete untying result and original C interoperability fixtures. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ShiroRsBytes *owned(const char *data, size_t count) {
    ShiroRsBytes *output = NULL; assert(shiro_rs_bytes_create((const uint8_t *)data, count, &output) == 0); return output;
}
static ShiroRsBytes *fixture(const char *name) {
    char path[160]; snprintf(path, sizeof(path), "tests/fixtures/%s", name);
    FILE *file = fopen(path, "rb"); assert(file && fseek(file, 0, SEEK_END) == 0);
    long count = ftell(file); assert(count >= 0 && fseek(file, 0, SEEK_SET) == 0);
    char *data = malloc((size_t)count + 1); assert(data);
    assert(fread(data, 1, (size_t)count, file) == (size_t)count && fclose(file) == 0);
    ShiroRsBytes *output = owned(data, (size_t)count); free(data); return output;
}
static void equal_bytes(ShiroRsBytes *a, ShiroRsBytes *b) {
    size_t n = 0, m = 0; assert(shiro_rs_bytes_length(a, &n) == 0 && shiro_rs_bytes_length(b, &m) == 0 && n == m);
    unsigned char *left = malloc(n + 1), *right = malloc(n + 1); assert(left && right);
    assert(shiro_rs_bytes_copy(a, 0, left, n) == 0 && shiro_rs_bytes_copy(b, 0, right, n) == 0);
    assert(memcmp(left, right, n) == 0); free(left); free(right);
}
int main(void) {
    ShiroRsBytes *wire = fixture("init-c-aligned.hsmm"); ShiroRsModel *model = NULL;
    assert(shiro_rs_model_read_bytes(wire, 16 * 1024 * 1024, &model) == 0 && shiro_rs_bytes_release(&wire) == 0);
    ShiroRsBytes *document = fixture("align-c-isolated.json"); ShiroRsUntiedModel *result = NULL, *clone = NULL;
    assert(shiro_rs_untie(model, document, &result) == 0);
    assert(shiro_rs_untied_model_clone(result, &clone) == 0 && shiro_rs_untied_model_release(&result) == 0);
    size_t count = 0; assert(shiro_rs_untied_model_length(clone, &count) == 0 && count == 6);
    for (size_t i = 0; i < count; ++i) {
        _Alignas(8) ShiroRsAssignment value;
        assert(shiro_rs_untied_model_get_assignment(clone, i, &value) == 0);
        assert(value.state == i && value.file == 0 && value.segment == i);
    }
    ShiroRsModel *snapshot = NULL; ShiroRsBytes *doc = NULL, *summary = NULL;
    assert(shiro_rs_untied_model_get_model(clone, &snapshot) == 0 && shiro_rs_untied_model_get_document(clone, &doc) == 0);
    assert(shiro_rs_untied_model_summary_bytes(clone, &summary) == 0 && shiro_rs_untied_model_release(&clone) == 0);
    assert(shiro_rs_model_write_bytes(snapshot, 0, &wire) == 0);
    ShiroRsBytes *expected = fixture("untie-c.hsmm"); equal_bytes(wire, expected);
    assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected) == 0 && shiro_rs_model_release(&snapshot) == 0);
    expected = fixture("untie-c-summary.txt"); equal_bytes(summary, expected);
    assert(shiro_rs_bytes_release(&expected) == 0 && shiro_rs_bytes_release(&summary) == 0);
    /* Normalize only field ordering through the same native document type. */
    expected = fixture("untie-c.json");
    assert(shiro_rs_untied_model_create(model, expected, NULL, 0, &result) == 0 && shiro_rs_bytes_release(&expected) == 0);
    assert(shiro_rs_untied_model_get_document(result, &expected) == 0); equal_bytes(doc, expected);
    assert(shiro_rs_bytes_release(&doc) == 0 && shiro_rs_bytes_release(&expected) == 0 && shiro_rs_untied_model_release(&result) == 0);
    _Alignas(8) ShiroRsAssignment entry = {SIZE_MAX, 0, 2};
    assert(shiro_rs_untied_model_create(model, document, &entry, 1, &result) == 0);
    _Alignas(8) ShiroRsAssignment value = {0, 0, 0};
    assert(shiro_rs_untied_model_get_assignment(result, 0, &value) == 0 && value.state == SIZE_MAX && value.file == 0 && value.segment == 2);
    assert(shiro_rs_untied_model_get_assignment(result, 1, &value) == 2 && value.state == SIZE_MAX && value.segment == 2);
    assert(shiro_rs_untied_model_summary_bytes(result, &summary) == 0);
    char text[128]; snprintf(text, sizeof(text), "%" PRIuPTR " 0 2 a 2\n", (uintptr_t)SIZE_MAX);
    expected = owned(text, strlen(text)); equal_bytes(summary, expected); assert(shiro_rs_bytes_release(&expected) == 0);
    ShiroRsUntiedModel *retained = result;
    assert(shiro_rs_untied_model_create(model, document, NULL, SIZE_MAX, &retained) == 2 && retained == result);
    const char *bad_json = "{}"; expected = owned(bad_json, strlen(bad_json));
    assert(shiro_rs_untie(model, expected, &retained) == 3 && retained == result); assert(shiro_rs_bytes_release(&expected) == 0);
    _Alignas(8) ShiroRsAssignment invalid[] = {{0, 0, 0}, {4, 99, 0}};
    assert(shiro_rs_untied_model_create(model, document, invalid, 2, &clone) == 0);
    ShiroRsBytes *retained_summary = summary;
    assert(shiro_rs_untied_model_summary_bytes(clone, &retained_summary) == 3 && retained_summary == summary);
    assert(shiro_rs_untied_model_release(&clone) == 0);
    assert(shiro_rs_untied_model_create(model, document, NULL, 0, &clone) == 0);
    assert(shiro_rs_untied_model_summary_bytes(clone, &wire) == 0 && shiro_rs_bytes_length(wire, &count) == 0 && count == 0);
    assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_untied_model_release(&clone) == 0);
    assert(shiro_rs_untied_model_release(NULL) == 1 && shiro_rs_untied_model_clone(result, NULL) == 1 && shiro_rs_untied_model_length(result, NULL) == 1);
    assert(shiro_rs_untied_model_release(&result) == 0 && shiro_rs_untied_model_release(&result) == 0);
    assert(shiro_rs_bytes_release(&summary) == 0);
    assert(shiro_rs_model_write_bytes(model, 0, &wire) == 0); expected = fixture("init-c-aligned.hsmm"); equal_bytes(wire, expected);
    assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected) == 0);
    expected = fixture("align-c-isolated.json"); equal_bytes(document, expected);
    assert(shiro_rs_bytes_release(&document) == 0 && shiro_rs_bytes_release(&expected) == 0 && shiro_rs_model_release(&model) == 0);
    puts("SHIRO untying C: all9 exports, original C model/JSON/summary, complete assignments and failure transactions passed"); return 0;
}
