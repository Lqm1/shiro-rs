/* Actual C complete typed states, files and segmentation documents. */
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
static ShiroRsBytes *text(const char *value) { return owned(value, strlen(value)); }
static ShiroRsBytes *fixture(const char *name) {
    char path[160]; assert(snprintf(path, sizeof(path), "tests/fixtures/%s", name) > 0);
    FILE *file = fopen(path, "rb"); assert(file && fseek(file, 0, SEEK_END) == 0);
    long count = ftell(file); assert(count >= 0 && fseek(file, 0, SEEK_SET) == 0);
    void *data = malloc((size_t)count + 1); assert(data);
    assert(fread(data, 1, (size_t)count, file) == (size_t)count && fclose(file) == 0);
    ShiroRsBytes *output = owned(data, (size_t)count); free(data); return output;
}
static void equal(ShiroRsBytes *left, ShiroRsBytes *right) {
    uintptr_t a = 0, b = 0;
    assert(shiro_rs_bytes_length(left, &a) == 0 && shiro_rs_bytes_length(right, &b) == 0 && a == b);
    unsigned char *x = malloc(a + 1), *y = malloc(b + 1); assert(x && y);
    assert(shiro_rs_bytes_copy(left, 0, x, a) == 0 && shiro_rs_bytes_copy(right, 0, y, b) == 0);
    assert(memcmp(x, y, a) == 0); free(x); free(y);
}
static void equal_text(ShiroRsBytes *owner, const char *expected) {
    ShiroRsBytes *value = text(expected); equal(owner, value); assert(shiro_rs_bytes_release(&value) == 0);
}
static double value(uint64_t bits) { double output; memcpy(&output, &bits, sizeof(output)); return output; }
static uint64_t bits(double input) { uint64_t output; memcpy(&output, &input, sizeof(output)); return output; }

static ShiroRsStates *rebuild_states(ShiroRsStates *original) {
    uintptr_t count = 0; assert(shiro_rs_states_length(original, &count) == 0);
    ShiroRsStateInput *inputs = calloc(count + 1, sizeof(*inputs));
    ShiroRsBytes **bytes = calloc(count * 3 + 1, sizeof(*bytes));
    ShiroRsArrayUsize **arrays = calloc(count + 1, sizeof(*arrays)); assert(inputs && bytes && arrays);
    for (uintptr_t i = 0; i < count; ++i) {
        ShiroRsStateInfo info = {0}; assert(shiro_rs_states_get_info(original, i, &info) == 0);
        inputs[i].time = info.time; inputs[i].has_duration = info.has_duration; inputs[i].duration = info.duration;
        if (info.has_outputs) {
            assert(shiro_rs_states_get_outputs(original, i, &arrays[i]) == 0); inputs[i].outputs = arrays[i];
        } else { assert(shiro_rs_states_get_outputs(original, i, &arrays[i]) == 2 && !arrays[i]); }
        if (info.has_jumps) {
            assert(shiro_rs_states_get_json_field(original, i, 0, &bytes[i * 3]) == 0); inputs[i].jumps = bytes[i * 3];
        } else { assert(shiro_rs_states_get_json_field(original, i, 0, &bytes[i * 3]) == 2 && !bytes[i * 3]); }
        assert(shiro_rs_states_get_json_field(original, i, 1, &bytes[i * 3 + 1]) == 0);
        assert(shiro_rs_states_get_json_field(original, i, 2, &bytes[i * 3 + 2]) == 0);
        inputs[i].metadata = bytes[i * 3 + 1]; inputs[i].attributes = bytes[i * 3 + 2];
    }
    ShiroRsStates *output = NULL; assert(shiro_rs_states_create(inputs, count, &output) == 0);
    for (uintptr_t i = 0; i < count * 3; ++i) assert(shiro_rs_bytes_release(&bytes[i]) == 0);
    for (uintptr_t i = 0; i < count; ++i) assert(shiro_rs_array_usize_release(&arrays[i]) == 0);
    free(inputs); free(bytes); free(arrays); return output;
}
int main(void) {
    const char *names[] = {"utterances-c-initial.json", "utterances-c-aligned.json", "align-c-isolated.json", "init-segmentation.json"};
    for (size_t case_index = 0; case_index < 4; ++case_index) {
        ShiroRsBytes *source = fixture(names[case_index]), *expected = NULL, *attributes = NULL;
        ShiroRsSegmentationDocument *parsed = NULL;
        assert(shiro_rs_document_read_json(source, &parsed) == 0 && shiro_rs_bytes_release(&source) == 0);
        assert(shiro_rs_document_write_json(parsed, &expected) == 0 && shiro_rs_document_get_attributes(parsed, &attributes) == 0);
        uintptr_t count = 0; assert(shiro_rs_document_length(parsed, &count) == 0);
        ShiroRsSegmentedFile **files = calloc(count + 1, sizeof(*files)); assert(files);
        for (uintptr_t i = 0; i < count; ++i) {
            ShiroRsSegmentedFile *file = NULL, *snapshot = NULL;
            assert(shiro_rs_document_get_file(parsed, i, &file) == 0 && shiro_rs_segmented_file_clone(file, &snapshot) == 0);
            assert(shiro_rs_segmented_file_release(&file) == 0);
            ShiroRsBytes *filename = NULL, *attrs = NULL, *state_wire = NULL, *rebuilt_wire = NULL;
            ShiroRsStates *original = NULL;
            assert(shiro_rs_segmented_file_get_filename(snapshot, &filename) == 0);
            assert(shiro_rs_segmented_file_get_attributes(snapshot, &attrs) == 0);
            assert(shiro_rs_segmented_file_get_states(snapshot, &original) == 0 && shiro_rs_segmented_file_release(&snapshot) == 0);
            ShiroRsStates *typed = rebuild_states(original);
            assert(shiro_rs_states_write_json(original, &state_wire) == 0 && shiro_rs_states_write_json(typed, &rebuilt_wire) == 0);
            equal(state_wire, rebuilt_wire);
            assert(shiro_rs_segmented_file_create(filename, typed, attrs, &files[i]) == 0);
            assert(shiro_rs_states_release(&original) == 0 && shiro_rs_states_release(&typed) == 0);
            assert(shiro_rs_bytes_release(&filename) == 0 && shiro_rs_bytes_release(&attrs) == 0);
            assert(shiro_rs_bytes_release(&state_wire) == 0 && shiro_rs_bytes_release(&rebuilt_wire) == 0);
        }
        assert(shiro_rs_document_release(&parsed) == 0);
        ShiroRsSegmentationDocument *rebuilt = NULL, *snapshot = NULL;
        assert(shiro_rs_document_create((const ShiroRsSegmentedFile *const *)files, count, attributes, &rebuilt) == 0);
        for (uintptr_t i = 0; i < count; ++i) assert(shiro_rs_segmented_file_release(&files[i]) == 0);
        free(files); assert(shiro_rs_bytes_release(&attributes) == 0);
        assert(shiro_rs_document_clone(rebuilt, &snapshot) == 0 && shiro_rs_document_release(&rebuilt) == 0);
        ShiroRsBytes *actual = NULL; assert(shiro_rs_document_write_json(snapshot, &actual) == 0); equal(actual, expected);
        assert(shiro_rs_bytes_release(&actual) == 0 && shiro_rs_bytes_release(&expected) == 0 && shiro_rs_document_release(&snapshot) == 0);
    }
    const uint64_t patterns[] = {UINT64_C(0x8000000000000000), UINT64_C(0x7ff8123456789abc), UINT64_C(0x7ff0000000000000), UINT64_C(0xfff0000000000000), 1};
    const char *metadata_text = "[\"phone\",7,{\"nested\":[null,true,\"\\u0000\"]}]";
    const char *attributes_text = "{\"dur\":false,\"ext\":42,\"jmp\":null,\"out\":[],\"time\":\"shadow\"}";
    const char *file_attributes_text = "{\"filename\":\"shadow\",\"nested\":[null,true],\"states\":0}";
    const char *top_attributes_text = "{\"file_list\":\"shadow\",\"nested\":{\"ok\":true}}";
    ShiroRsBytes *metadata = text(metadata_text), *attributes = text(attributes_text), *jumps = text("[]");
    ShiroRsBytes *file_attrs = text(file_attributes_text), *top_attrs = text(top_attributes_text);
    ShiroRsArrayUsize *outputs = NULL, *empty = NULL; const uintptr_t integers[] = {0, UINTPTR_MAX, UINTPTR_MAX - 1};
    assert(shiro_rs_array_usize_create(integers, 3, &outputs) == 0 && shiro_rs_array_usize_create(NULL, 0, &empty) == 0);
    ShiroRsStateInput inputs[5];
    for (size_t i = 0; i < 5; ++i) {
        inputs[i] = (ShiroRsStateInput){value(patterns[i]), i != 0, i == 1 ? 0 : UINTPTR_MAX,
            i == 0 ? NULL : i == 1 ? empty : outputs, i == 0 ? NULL : jumps, metadata, attributes};
    }
    ShiroRsStates *states = NULL; assert(shiro_rs_states_create(inputs, 5, &states) == 0);
    const unsigned char filename[] = "audio-\xf0\x9f\x8e\xb5\0file.wav";
    ShiroRsBytes *name = owned(filename, sizeof(filename) - 1);
    ShiroRsSegmentedFile *file = NULL; assert(shiro_rs_segmented_file_create(name, states, file_attrs, &file) == 0);
    const ShiroRsSegmentedFile *files[] = {file, file}; ShiroRsSegmentationDocument *document = NULL, *snapshot = NULL;
    assert(shiro_rs_document_create(files, 2, top_attrs, &document) == 0 && shiro_rs_segmented_file_release(&file) == 0);
    assert(shiro_rs_states_release(&states) == 0 && shiro_rs_bytes_release(&name) == 0);
    assert(shiro_rs_bytes_release(&metadata) == 0 && shiro_rs_bytes_release(&attributes) == 0 && shiro_rs_bytes_release(&jumps) == 0);
    assert(shiro_rs_array_usize_release(&outputs) == 0 && shiro_rs_array_usize_release(&empty) == 0);
    assert(shiro_rs_document_clone(document, &snapshot) == 0 && shiro_rs_document_release(&document) == 0);
    uintptr_t count = 99; assert(shiro_rs_document_length(snapshot, &count) == 0 && count == 2);
    ShiroRsBytes *actual_attrs = NULL; assert(shiro_rs_document_get_attributes(snapshot, &actual_attrs) == 0); equal(actual_attrs, top_attrs);
    assert(shiro_rs_bytes_release(&actual_attrs) == 0);
    ShiroRsSegmentedFile *saved[2] = {NULL, NULL};
    for (size_t i = 0; i < 2; ++i) assert(shiro_rs_document_get_file(snapshot, i, &saved[i]) == 0);
    ShiroRsSegmentedFile *retained_file = saved[0];
    assert(shiro_rs_document_get_file(snapshot, 2, &retained_file) == 2 && retained_file == saved[0]);
    ShiroRsBytes *retained_bytes = file_attrs;
    assert(shiro_rs_document_write_json(snapshot, &retained_bytes) == 3 && retained_bytes == file_attrs);
    ShiroRsSegmentationDocument *retained_document = snapshot;
    assert(shiro_rs_document_create(NULL, 1, top_attrs, &retained_document) == 1 && retained_document == snapshot);
    const ShiroRsSegmentedFile *invalid_files[] = {NULL};
    assert(shiro_rs_document_create(invalid_files, 1, top_attrs, &retained_document) == 1 && retained_document == snapshot);
    assert(shiro_rs_document_clone(snapshot, NULL) == 1 && shiro_rs_document_length(snapshot, NULL) == 1);
    assert(shiro_rs_document_release(&snapshot) == 0);
    for (size_t i = 0; i < 2; ++i) {
        assert(shiro_rs_segmented_file_get_filename(saved[i], &name) == 0);
        ShiroRsBytes *expected_name = owned(filename, sizeof(filename) - 1); equal(name, expected_name);
        assert(shiro_rs_bytes_release(&expected_name) == 0 && shiro_rs_bytes_release(&name) == 0);
        assert(shiro_rs_segmented_file_get_states(saved[i], &states) == 0);
        assert(shiro_rs_segmented_file_get_attributes(saved[i], &actual_attrs) == 0); equal(actual_attrs, file_attrs);
        assert(shiro_rs_bytes_release(&actual_attrs) == 0 && shiro_rs_segmented_file_release(&saved[i]) == 0);
        assert(shiro_rs_states_length(states, &count) == 0 && count == 5);
        for (size_t j = 0; j < 5; ++j) {
            ShiroRsStateInfo info = {0}; assert(shiro_rs_states_get_info(states, j, &info) == 0);
            assert(bits(info.time) == patterns[j] && info.has_duration == (j != 0) && info.duration == (j <= 1 ? 0 : UINTPTR_MAX));
            assert(info.has_outputs == (j != 0) && info.has_jumps == (j != 0));
            assert(shiro_rs_states_get_outputs(states, j, &outputs) == (j == 0 ? 2u : 0u));
            if (j != 0) {
                uintptr_t length = 99, actual[3] = {0};
                assert(shiro_rs_array_usize_length(outputs, &length) == 0 && length == (j == 1 ? 0 : 3));
                assert(shiro_rs_array_usize_copy(outputs, 0, actual, length) == 0 && memcmp(actual, integers, length * sizeof(*actual)) == 0);
                assert(shiro_rs_array_usize_release(&outputs) == 0);
            }
            for (uint32_t field = 0; field < 3; ++field) {
                ShiroRsBytes *output = NULL; uint32_t status = shiro_rs_states_get_json_field(states, j, field, &output);
                if (j == 0 && field == 0) { assert(status == 2 && !output); continue; }
                assert(status == 0); equal_text(output, field == 0 ? "[]" : field == 1 ? metadata_text : attributes_text);
                assert(shiro_rs_bytes_release(&output) == 0);
            }
        }
        retained_bytes = file_attrs;
        assert(shiro_rs_states_write_json(states, &retained_bytes) == 3 && retained_bytes == file_attrs);
        assert(shiro_rs_states_get_json_field(states, 0, 3, &retained_bytes) == 2 && retained_bytes == file_attrs);
        ShiroRsStateInfo info = {17.0, 0, 0, 0, 0};
        assert(shiro_rs_states_get_info(states, UINTPTR_MAX, &info) == 2 && info.time == 17.0);
        ShiroRsStateInput invalid = {0, 2, 0, NULL, NULL, file_attrs, file_attrs};
        ShiroRsStates *retained = states;
        assert(shiro_rs_states_create(&invalid, 1, &retained) == 2 && retained == states);
        assert(shiro_rs_states_create(NULL, 1, &retained) == 1 && retained == states);
        const unsigned char bad_utf8[] = {0xff}; ShiroRsBytes *bad = owned(bad_utf8, 1);
        assert(shiro_rs_segmented_file_create(bad, states, file_attrs, &file) == 3 && !file);
        assert(shiro_rs_bytes_release(&bad) == 0 && shiro_rs_states_release(&states) == 0);
    }
    assert(shiro_rs_states_create(NULL, 0, &states) == 0 && shiro_rs_states_length(states, &count) == 0 && count == 0);
    assert(shiro_rs_states_release(&states) == 0);
    assert(shiro_rs_document_create(NULL, 0, top_attrs, &document) == 0 && shiro_rs_document_length(document, &count) == 0 && count == 0);
    assert(shiro_rs_document_release(&document) == 0 && shiro_rs_document_release(&document) == 0);
    assert(shiro_rs_bytes_release(&file_attrs) == 0 && shiro_rs_bytes_release(&top_attrs) == 0);
    puts("SHIRO documents C: all19 exports, original typed reconstruction/full fields/bits/ownership passed"); return 0;
}
