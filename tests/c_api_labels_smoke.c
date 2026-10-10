/* Actual C complete label fields, original Lua conversion and failure checks. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ShiroRsBytes *owned(const void *data, size_t count) {
    ShiroRsBytes *result = NULL; assert(shiro_rs_bytes_create(data, count, &result) == 0); return result;
}
static ShiroRsBytes *text(const char *value) { return owned(value, strlen(value)); }
static char *copied(ShiroRsBytes *bytes, size_t *count) {
    assert(shiro_rs_bytes_length(bytes, count) == 0); char *result = malloc(*count + 1); assert(result);
    assert(shiro_rs_bytes_copy(bytes, 0, (uint8_t *)result, *count) == 0); result[*count] = 0; return result;
}
static ShiroRsBytes *fixture(const char *name) {
    char path[160]; snprintf(path, sizeof(path), "tests/fixtures/%s", name);
    FILE *file = fopen(path, "rb"); assert(file && fseek(file, 0, SEEK_END) == 0);
    long count = ftell(file); assert(count >= 0 && fseek(file, 0, SEEK_SET) == 0);
    char *data = malloc((size_t)count + 1); assert(data);
    assert(fread(data, 1, (size_t)count, file) == (size_t)count && fclose(file) == 0);
    ShiroRsBytes *result = owned(data, (size_t)count); free(data); return result;
}
static void bytes_equal(ShiroRsBytes *a, ShiroRsBytes *b) {
    size_t n, m; char *x = copied(a, &n), *y = copied(b, &m);
    assert(n == m && memcmp(x, y, n) == 0); free(x); free(y);
}
static void rows_equal(ShiroRsLabels *a, ShiroRsLabels *b, double tolerance) {
    size_t count = 0, other = 0;
    assert(shiro_rs_labels_length(a, &count) == 0 && shiro_rs_labels_length(b, &other) == 0 && count == other);
    for (size_t i = 0; i < count; ++i) {
        _Alignas(8) ShiroRsLabelInfo x, y; ShiroRsBytes *left = NULL, *right = NULL;
        assert(shiro_rs_labels_get_info(a, i, &x) == 0 && shiro_rs_labels_get_info(b, i, &y) == 0);
        assert(fabs(x.start - y.start) <= tolerance && fabs(x.end - y.end) <= tolerance);
        assert(shiro_rs_labels_get_name(a, i, &left) == 0 && shiro_rs_labels_get_name(b, i, &right) == 0);
        bytes_equal(left, right); assert(shiro_rs_bytes_release(&left) == 0 && shiro_rs_bytes_release(&right) == 0);
    }
}
static ShiroRsBytes *original_states(void) {
    ShiroRsBytes *document = fixture("labels-original-seg.json"); size_t count;
    char *json = copied(document, &count), *start = strstr(json, "\"states\""); assert(start);
    start = strchr(start, '['); assert(start); char *end = start;
    int depth = 0, quoted = 0, escaped = 0;
    for (; *end; ++end) {
        if (quoted) { if (escaped) escaped = 0; else if (*end == '\\') escaped = 1; else if (*end == '"') quoted = 0; }
        else if (*end == '"') quoted = 1;
        else if (*end == '[') ++depth;
        else if (*end == ']' && --depth == 0) { ++end; break; }
    }
    assert(depth == 0 && !quoted); ShiroRsBytes *result = owned(start, (size_t)(end-start));
    free(json); assert(shiro_rs_bytes_release(&document) == 0); return result;
}
int main(void) {
    ShiroRsBytes *input = fixture("labels-input.txt"); ShiroRsLabels *labels = NULL, *clone = NULL;
    assert(shiro_rs_labels_parse(input, &labels) == 0 && shiro_rs_bytes_release(&input) == 0);
    assert(shiro_rs_labels_clone(labels, &clone) == 0 && shiro_rs_labels_release(&labels) == 0);
    input = fixture("labels-phones.json"); ShiroRsPhoneMap *map = NULL;
    assert(shiro_rs_phone_map_read_json(input, &map) == 0 && shiro_rs_bytes_release(&input) == 0);
    ShiroRsStates *states = NULL; assert(shiro_rs_labels_to_states(clone, map, 0.01, &states) == 0);
    ShiroRsStates *retained_states = states;
    assert(shiro_rs_labels_to_states(clone, map, 0, &retained_states) == 3 && retained_states == states);
    assert(shiro_rs_labels_release(&clone) == 0 && shiro_rs_phone_map_release(&map) == 0);
    size_t count = 0; ShiroRsBytes *wire = NULL;
    assert(shiro_rs_states_write_json(states, &wire) == 0);
    input = original_states(); ShiroRsStates *expected_states = NULL; ShiroRsBytes *expected_wire = NULL;
    assert(shiro_rs_states_read_json(input, &expected_states) == 0 && shiro_rs_bytes_release(&input) == 0);
    assert(shiro_rs_states_write_json(expected_states, &expected_wire) == 0); bytes_equal(wire, expected_wire);
    assert(shiro_rs_states_release(&expected_states) == 0 && shiro_rs_bytes_release(&expected_wire) == 0);
    char *state_json = copied(wire, &count);
    assert(strstr(state_json, "\"time\":8.0") && strstr(state_json, "\"ext\":[\"aa\",2]")); free(state_json);
    assert(shiro_rs_bytes_release(&wire) == 0);
    for (uint32_t include = 0; include < 2; ++include) {
        assert(shiro_rs_labels_from_states(states, 0.01, include, &labels) == 0);
        input = fixture(include ? "labels-original-states.txt" : "labels-original-phones.txt");
        ShiroRsLabels *expected = NULL; assert(shiro_rs_labels_parse(input, &expected) == 0);
        rows_equal(labels, expected, 1e-14);
        assert(shiro_rs_bytes_release(&input) == 0 && shiro_rs_labels_release(&expected) == 0);
        assert(shiro_rs_labels_write_bytes(labels, &wire) == 0);
        char *data = copied(wire, &count); size_t rows = 0;
        for (size_t i = 0; i < count; ++i) if (data[i] == '\n') { assert(i && data[i-1] == '\r'); ++rows; }
        assert(rows == (include ? 10 : 3)); free(data);
        assert(shiro_rs_labels_parse(wire, &clone) == 0); rows_equal(labels, clone, 0);
        assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_labels_release(&clone) == 0);
        ShiroRsLabels *retained = labels;
        assert(shiro_rs_labels_from_states(states, 0.01, 2, &retained) == 2 && retained == labels);
        assert(shiro_rs_labels_from_states(states, NAN, 0, &retained) == 3 && retained == labels);
        assert(shiro_rs_labels_release(&labels) == 0);
    }
    assert(shiro_rs_states_release(&states) == 0);
    const char name[] = "retained\0UTF8\xce\xbb"; ShiroRsBytes *name_owner = owned(name, sizeof(name)-1);
    uint64_t bits[] = {UINT64_C(0x8000000000000000), UINT64_C(1), UINT64_C(0x7ff8000000000042), UINT64_C(0xfff0000000000000)};
    _Alignas(8) ShiroRsLabelInput values[2];
    for (size_t i = 0; i < 2; ++i) { memcpy(&values[i].start, &bits[2*i], sizeof(double)); memcpy(&values[i].end, &bits[2*i+1], sizeof(double)); values[i].name = name_owner; }
    assert(shiro_rs_labels_create(values, 2, &labels) == 0 && shiro_rs_bytes_release(&name_owner) == 0);
    for (size_t i = 0; i < 2; ++i) {
        _Alignas(8) ShiroRsLabelInfo info; uint64_t start, end;
        assert(shiro_rs_labels_get_info(labels, i, &info) == 0);
        memcpy(&start, &info.start, sizeof(double)); memcpy(&end, &info.end, sizeof(double));
        assert(start == bits[2*i] && end == bits[2*i+1]);
    }
    ShiroRsBytes *snapshot = NULL; assert(shiro_rs_labels_get_name(labels, 0, &snapshot) == 0);
    _Alignas(8) ShiroRsLabelInfo info = {12, 13}; assert(shiro_rs_labels_get_info(labels, 2, &info) == 2 && info.start == 12 && info.end == 13);
    ShiroRsBytes *retained_wire = snapshot;
    assert(shiro_rs_labels_get_name(labels, 2, &retained_wire) == 2 && retained_wire == snapshot);
    ShiroRsLabels *retained = labels; input = text("0 1 aa\n\nx 2 bb");
    assert(shiro_rs_labels_parse(input, &retained) == 3 && retained == labels);
    assert(shiro_rs_bytes_release(&input) == 0 && shiro_rs_labels_release(&labels) == 0);
    char *actual_name = copied(snapshot, &count); assert(count == sizeof(name)-1 && memcmp(actual_name, name, count) == 0); free(actual_name);
    ShiroRsBytes *valid = text("valid"), *bad = text("bad\tname");
    values[0] = (ShiroRsLabelInput){0, 1, valid}; values[1] = (ShiroRsLabelInput){1, 2, bad};
    assert(shiro_rs_labels_create(values, 2, &labels) == 0);
    assert(shiro_rs_labels_write_bytes(labels, &retained_wire) == 3 && retained_wire == snapshot);
    retained = labels; values[1].name = NULL;
    assert(shiro_rs_labels_create(values, 2, &retained) == 1 && retained == labels);
    assert(shiro_rs_labels_create(NULL, SIZE_MAX, &retained) == 2 && retained == labels);
    assert(shiro_rs_labels_release(&labels) == 0 && shiro_rs_bytes_release(&valid) == 0 && shiro_rs_bytes_release(&bad) == 0);
    assert(shiro_rs_labels_create(NULL, 0, &labels) == 0 && shiro_rs_labels_length(labels, &count) == 0 && count == 0);
    assert(shiro_rs_labels_write_bytes(labels, &wire) == 0 && shiro_rs_bytes_length(wire, &count) == 0 && count == 0);
    assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_labels_release(&labels) == 0);
    const char *paths[] = {"dir.name/clip.features.f", "dir.name\\clip", ".f", "clip"};
    const char *expected[] = {"dir.name/clip.features.txt", "dir.name\\clip.txt", ".txt", "clip.txt"};
    for (size_t i = 0; i < 4; ++i) {
        input = text(paths[i]); ShiroRsBytes *suffix = text(".txt");
        assert(shiro_rs_labels_output_path(input, suffix, &wire) == 0);
        char *path = copied(wire, &count); assert(strcmp(path, expected[i]) == 0); free(path);
        assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&input) == 0 && shiro_rs_bytes_release(&suffix) == 0);
    }
    assert(shiro_rs_labels_release(NULL) == 1 && shiro_rs_labels_clone(NULL, &labels) == 1);
    assert(shiro_rs_labels_get_info(NULL, 0, &info) == 1 && shiro_rs_bytes_release(&snapshot) == 0);
    puts("SHIRO labels C: all11 exports, original Lua rows, complete binary64 bits/names, CRLF, paths and failures passed");
    return 0;
}
