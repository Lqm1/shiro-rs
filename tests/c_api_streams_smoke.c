/* Actual C native streaming callbacks, partial transfers and original fixtures. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct Channel {
    const unsigned char *input;
    size_t length, position, chunk, calls, fail_at, flushes, consumed;
    unsigned char *output;
    size_t output_length;
} Channel;
static Channel channel(const unsigned char *input, size_t count, size_t chunk) {
    Channel value = {0}; value.input = input; value.length = count; value.chunk = chunk; value.fail_at = SIZE_MAX; return value;
}
static uint32_t step(Channel *value) {
    ++value->calls;
    if (value->calls == 1) return SHIRO_RS_IO_INTERRUPTED;
    return value->position >= value->fail_at ? SHIRO_RS_IO_ERROR : SHIRO_RS_IO_SUCCESS;
}
static uint32_t read_bytes(void *context, uint8_t *buffer, size_t capacity, size_t *count) {
    Channel *value = context; uint32_t status = step(value); if (status) return status;
    size_t n = value->length - value->position; if (n > capacity) n = capacity; if (n > value->chunk) n = value->chunk;
    if (n) memcpy(buffer, value->input + value->position, n); value->position += n; *count = n; return 0;
}
static uint32_t write_bytes(void *context, const uint8_t *buffer, size_t capacity, size_t *count) {
    Channel *value = context; uint32_t status = step(value); if (status) return status;
    size_t n = capacity < value->chunk ? capacity : value->chunk;
    unsigned char *next = realloc(value->output, value->output_length + n + 1); assert(next); value->output = next;
    if (n) memcpy(next + value->output_length, buffer, n); value->output_length += n; value->position += n; *count = n; return 0;
}
static uint32_t flush(void *context) { Channel *value = context; ++value->flushes; return step(value); }
static uint32_t fill(void *context, const uint8_t **buffer, size_t *count) {
    Channel *value = context; uint32_t status = step(value); if (status) return status;
    size_t n = value->length - value->position; if (n > value->chunk) n = value->chunk;
    *buffer = n ? value->input + value->position : NULL; *count = n; return 0;
}
static void consume(void *context, size_t count) {
    Channel *value = context; assert(count <= value->chunk && count <= value->length - value->position);
    value->position += count; value->consumed += count;
}
static uint32_t excessive_read(void *context, uint8_t *buffer, size_t capacity, size_t *count) {
    (void)context; (void)buffer; *count = capacity + 1; return 0;
}
static uint32_t excessive_write(void *context, const uint8_t *buffer, size_t capacity, size_t *count) {
    (void)context; (void)buffer; *count = capacity + 1; return 0;
}
static uint32_t invalid_fill(void *context, const uint8_t **buffer, size_t *count) {
    *buffer = NULL; *count = context ? SIZE_MAX : 1; return 0;
}
static void unexpected_consume(void *context, size_t count) { (void)context; (void)count; assert(0 && "invalid fill cannot reach consume"); }
static ShiroRsBytes *owned(const void *data, size_t count) {
    ShiroRsBytes *value = NULL; assert(shiro_rs_bytes_create(data, count, &value) == 0); return value;
}
static void equal_bytes(ShiroRsBytes *owner, const void *expected, size_t count) {
    size_t length = 99; assert(shiro_rs_bytes_length(owner, &length) == 0 && length == count);
    unsigned char *copy = malloc(count + 1); assert(copy);
    assert(shiro_rs_bytes_copy(owner, 0, copy, count) == 0 && memcmp(copy, expected, count) == 0); free(copy);
}
static unsigned char *load(const char *path, size_t *count) {
    FILE *file = fopen(path, "rb"); assert(file); assert(fseek(file, 0, SEEK_END) == 0); long length = ftell(file); assert(length >= 0); rewind(file);
    *count = (size_t)length; unsigned char *bytes = malloc(*count + 1); assert(bytes);
    assert(fread(bytes, 1, *count, file) == *count && fclose(file) == 0); return bytes;
}
static ShiroRsBytes *fixture(const char *path) {
    size_t count; unsigned char *data = load(path, &count); ShiroRsBytes *owner = owned(data, count); free(data); return owner;
}
static void equal_string(ShiroRsStrings *owner, size_t index, const char *expected) {
    ShiroRsBytes *snapshot = NULL; assert(shiro_rs_strings_get(owner, index, &snapshot) == 0);
    equal_bytes(snapshot, expected, strlen(expected)); assert(shiro_rs_bytes_release(&snapshot) == 0);
}
int main(void) {
    assert(SHIRO_RS_IO_SUCCESS == 0 && SHIRO_RS_IO_INTERRUPTED == 1 && SHIRO_RS_IO_ERROR == 2);
    const unsigned char bits[] = {0,0,0,0,0,0,0,0x80,1,0,0,0,0x45,0x23,0xc1,0x7f,0,0,0x80,0x7f,0,0,0x80,0xff};
    const size_t chunks[] = {1, 7, 16384};
    for (size_t i = 0; i < 3; ++i) {
        Channel source = channel(bits, sizeof(bits), chunks[i]); ShiroRsReadStream reader = {&source, read_bytes}; ShiroRsArrayF32 *values = NULL;
        assert(shiro_rs_rawfloat_read_stream(&reader, 6, &values) == 0 && source.position == sizeof(bits));
        ShiroRsBytes *snapshot = NULL; assert(shiro_rs_rawfloat_write_bytes(values, &snapshot) == 0); equal_bytes(snapshot, bits, sizeof(bits)); assert(shiro_rs_bytes_release(&snapshot) == 0);
        Channel target = channel(NULL, 0, chunks[i]); ShiroRsWriteStream writer = {&target, write_bytes, flush};
        assert(shiro_rs_rawfloat_write_stream(values, &writer) == 0 && target.output_length == sizeof(bits) && memcmp(target.output, bits, sizeof(bits)) == 0 && target.flushes == 0);
        assert(shiro_rs_write_stream_flush(&writer) == 0 && target.flushes == 1); free(target.output);
        for (size_t error = 0; error < 3; ++error) {
            source = channel(bits, error == 0 ? sizeof(bits) - 1 : sizeof(bits), 1); if (error == 2) source.fail_at = 5;
            ShiroRsArrayF32 *retained = values; assert(shiro_rs_rawfloat_read_stream(&reader, error == 1 ? 1 : 6, &retained) == 3 && retained == values);
        }
        target = channel(NULL, 0, 1); target.fail_at = 5;
        assert(shiro_rs_rawfloat_write_stream(values, &writer) == 3 && target.output_length == 5 && memcmp(target.output, bits, 5) == 0 && target.flushes == 0); free(target.output);
        target = channel(NULL, 0, 0); assert(shiro_rs_rawfloat_write_stream(values, &writer) == 3 && target.output_length == 0); free(target.output);
        ShiroRsReadStream invalid_reader = {NULL, excessive_read}; ShiroRsArrayF32 *retained = values;
        assert(shiro_rs_rawfloat_read_stream(&invalid_reader, 6, &retained) == 3 && retained == values);
        ShiroRsWriteStream invalid_writer = {NULL, excessive_write, NULL}; assert(shiro_rs_rawfloat_write_stream(values, &invalid_writer) == 3);
        invalid_reader.read = NULL; assert(shiro_rs_rawfloat_read_stream(&invalid_reader, 6, &retained) == 3 && retained == values);
        assert(shiro_rs_rawfloat_read_stream(NULL, 6, &retained) == 1 && shiro_rs_write_stream_flush(NULL) == 1);
        source = channel(bits, sizeof(bits), 1); assert(shiro_rs_rawfloat_read_stream(&reader, 6, NULL) == 1 && source.calls == 0);
        target = channel(NULL, 0, 1); assert(shiro_rs_write_stream_flush(&writer) == 3 && target.flushes == 1);
        assert(shiro_rs_write_stream_flush(&writer) == 0 && target.flushes == 2);
        assert(shiro_rs_array_f32_release(&values) == 0);
    }
    size_t count; unsigned char *raw = load("tests/fixtures/init-input.bin", &count); const size_t dimensions[] = {2, 1};
    ShiroRsBytes *input = owned(raw, count); ShiroRsObservation *reference = NULL; ShiroRsBytes *expected_wire = NULL;
    assert(shiro_rs_observation_read_rawfloat(input, dimensions, 2, 1000, &reference) == 0 && shiro_rs_observation_write_bytes(reference, &expected_wire) == 0);
    size_t wire_length; assert(shiro_rs_bytes_length(expected_wire, &wire_length) == 0); unsigned char *wire = malloc(wire_length + 1); assert(wire);
    assert(shiro_rs_bytes_copy(expected_wire, 0, wire, wire_length) == 0);
    for (size_t i = 0; i < 3; ++i) {
        Channel source = channel(raw, count, chunks[i]); ShiroRsReadStream reader = {&source, read_bytes}; ShiroRsObservation *observation = NULL; ShiroRsBytes *snapshot = NULL;
        assert(shiro_rs_observation_read_stream(&reader, dimensions, 2, 1000, &observation) == 0 && source.position == count);
        assert(shiro_rs_observation_write_bytes(observation, &snapshot) == 0); equal_bytes(snapshot, wire, wire_length);
        assert(shiro_rs_bytes_release(&snapshot) == 0 && shiro_rs_observation_release(&observation) == 0);
    }
    free(raw); free(wire); assert(shiro_rs_bytes_release(&input) == 0 && shiro_rs_bytes_release(&expected_wire) == 0 && shiro_rs_observation_release(&reference) == 0);
    input = owned("data", 4); ShiroRsPath *directory = NULL; ShiroRsStrings *padding = NULL;
    assert(shiro_rs_path_from_utf8(input, &directory) == 0 && shiro_rs_bytes_release(&input) == 0 && shiro_rs_strings_create(NULL, 0, &padding) == 0);
    raw = load("tests/fixtures/index-original.txt", &count);
    const char *names[] = {"clip one", "sub/clip.two", "silent"};
    const char *phones0[] = {"aa", "bb"}, *phones1[] = {"cc", "", "dd"}; const char **phones[] = {phones0, phones1, NULL}; const size_t phone_counts[] = {2, 3, 0};
    uint32_t kind = shiro_rs_path_native_encoding(); assert(kind == 1 || kind == 2);
    for (size_t i = 0; i < 3; ++i) {
        Channel source = channel(raw, count, chunks[i]); ShiroRsBufferedReadStream reader = {&source, fill, consume}; ShiroRsIndexEntries *entries = NULL;
        assert(shiro_rs_index_read_stream(&reader, directory, padding, padding, &entries) == 0 && source.position == count && source.consumed == count);
        size_t length; assert(shiro_rs_index_entries_length(entries, &length) == 0 && length == 3);
        for (size_t j = 0; j < 3; ++j) {
            ShiroRsPath *stem = NULL; ShiroRsStrings *values = NULL; ShiroRsBytes *snapshot = NULL;
            assert(shiro_rs_index_entries_get_stem(entries, j, &stem) == 0 && shiro_rs_path_native_bytes(stem, &snapshot) == 0);
            char path[64]; int n = snprintf(path, sizeof(path), "data%c%s", kind == 2 ? '\\' : '/', names[j]); assert(n > 0);
            size_t units = (size_t)n * (kind == 2 ? 2 : 1); unsigned char *expected = calloc(units + 1, 1); assert(expected);
            for (int k = 0; k < n; ++k) expected[(size_t)k * (kind == 2 ? 2 : 1)] = (unsigned char)path[k]; equal_bytes(snapshot, expected, units); free(expected);
            assert(shiro_rs_index_entries_get_phonemes(entries, j, &values) == 0 && shiro_rs_strings_length(values, &length) == 0 && length == phone_counts[j]);
            for (size_t k = 0; k < length; ++k) equal_string(values, k, phones[j][k]);
            assert(shiro_rs_path_release(&stem) == 0 && shiro_rs_strings_release(&values) == 0 && shiro_rs_bytes_release(&snapshot) == 0);
        }
        const char *bad[] = {"\n\r\nbad\n", "clip,\xff\n"};
        for (size_t j = 0; j < 2; ++j) {
            source = channel((const unsigned char *)bad[j], strlen(bad[j]), 1); ShiroRsIndexEntries *retained = entries;
            assert(shiro_rs_index_read_stream(&reader, directory, padding, padding, &retained) == 3 && retained == entries);
        }
        source = channel(raw, count, 1); source.fail_at = 9; ShiroRsIndexEntries *retained = entries;
        assert(shiro_rs_index_read_stream(&reader, directory, padding, padding, &retained) == 3 && retained == entries && source.position == 9);
        for (size_t j = 0; j < 2; ++j) {
            ShiroRsBufferedReadStream invalid = {j ? &source : NULL, invalid_fill, unexpected_consume};
            assert(shiro_rs_index_read_stream(&invalid, directory, padding, padding, &retained) == 3 && retained == entries);
        }
        source = channel(raw, count, 1); assert(shiro_rs_index_read_stream(&reader, directory, padding, padding, NULL) == 1 && source.calls == 0);
        assert(shiro_rs_index_entries_release(&entries) == 0);
    }
    free(raw); assert(shiro_rs_path_release(&directory) == 0 && shiro_rs_strings_release(&padding) == 0);
    input = fixture("tests/fixtures/labels-input.txt"); ShiroRsLabels *labels = NULL; ShiroRsBytes *label_wire = NULL;
    assert(shiro_rs_labels_parse(input, &labels) == 0 && shiro_rs_labels_write_bytes(labels, &label_wire) == 0 && shiro_rs_bytes_release(&input) == 0);
    assert(shiro_rs_bytes_length(label_wire, &wire_length) == 0); wire = malloc(wire_length + 1); assert(wire); assert(shiro_rs_bytes_copy(label_wire, 0, wire, wire_length) == 0);
    input = fixture("tests/fixtures/init-c-aligned.hsmm"); ShiroRsModel *model = NULL;
    assert(shiro_rs_model_read_bytes(input, 16 * 1024 * 1024, &model) == 0 && shiro_rs_bytes_release(&input) == 0);
    input = fixture("tests/fixtures/align-c-isolated.json"); ShiroRsUntiedModel *untied = NULL;
    assert(shiro_rs_untie(model, input, &untied) == 0 && shiro_rs_bytes_release(&input) == 0);
    size_t summary_length; unsigned char *summary = load("tests/fixtures/untie-c-summary.txt", &summary_length);
    for (size_t i = 0; i < 3; ++i) {
        Channel target = channel(NULL, 0, chunks[i]); ShiroRsWriteStream writer = {&target, write_bytes, flush};
        assert(shiro_rs_labels_write_stream(labels, &writer) == 0 && target.output_length == wire_length && memcmp(target.output, wire, wire_length) == 0 && target.flushes == 0); free(target.output);
        target = channel(NULL, 0, chunks[i]);
        assert(shiro_rs_untied_model_write_summary_stream(untied, &writer) == 0 && target.output_length == summary_length && memcmp(target.output, summary, summary_length) == 0 && target.flushes == 0); free(target.output);
    }
    Channel target = channel(NULL, 0, 1); target.fail_at = 7; ShiroRsWriteStream writer = {&target, write_bytes, flush};
    assert(shiro_rs_labels_write_stream(labels, &writer) == 3 && target.output_length == 7 && memcmp(target.output, wire, 7) == 0); free(target.output);
    target = channel(NULL, 0, 1); target.fail_at = 7;
    assert(shiro_rs_untied_model_write_summary_stream(untied, &writer) == 3 && target.output_length == 7 && memcmp(target.output, summary, 7) == 0); free(target.output);
    input = fixture("tests/fixtures/align-c-isolated.json"); ShiroRsAssignment assignments[] = {{0,0,0},{SIZE_MAX,SIZE_MAX,SIZE_MAX}}; ShiroRsUntiedModel *invalid = NULL;
    assert(shiro_rs_untied_model_create(model, input, assignments, 2, &invalid) == 0 && shiro_rs_bytes_release(&input) == 0);
    target = channel(NULL, 0, 1); assert(shiro_rs_untied_model_write_summary_stream(invalid, &writer) == 3 && target.calls == 0 && target.output_length == 0);
    assert(shiro_rs_labels_write_stream(NULL, &writer) == 1 && shiro_rs_untied_model_write_summary_stream(untied, NULL) == 1);
    assert(shiro_rs_untied_model_release(&invalid) == 0 && shiro_rs_untied_model_release(&untied) == 0 && shiro_rs_model_release(&model) == 0);
    assert(shiro_rs_labels_release(&labels) == 0 && shiro_rs_bytes_release(&label_wire) == 0); free(wire); free(summary);
    puts("SHIRO streams C: all7 exports, direct partial IO/full fields/original fixtures/callbacks/failures passed"); return 0;
}
