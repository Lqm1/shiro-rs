/* Actual C complete utterance workflows and independently owned result fields. */
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

static ShiroRsBytes *owned(const void *data, size_t count) {
    ShiroRsBytes *out = NULL;
    assert(shiro_rs_bytes_create(data, count, &out) == 0); return out;
}
static ShiroRsBytes *text(const char *data) { return owned(data, strlen(data)); }
static unsigned char *copy_bytes(ShiroRsBytes *owner, uintptr_t *count) {
    assert(shiro_rs_bytes_length(owner, count) == 0);
    unsigned char *data = malloc(*count + 1); assert(data);
    assert(shiro_rs_bytes_copy(owner, 0, data, *count) == 0); return data;
}
static ShiroRsBytes *fixture(const char *name) {
    char path[160]; assert(snprintf(path, sizeof(path), "tests/fixtures/%s", name) > 0);
    FILE *file = fopen(path, "rb"); assert(file && fseek(file, 0, SEEK_END) == 0);
    long count = ftell(file); assert(count >= 0 && fseek(file, 0, SEEK_SET) == 0);
    void *data = malloc((size_t)count + 1); assert(data);
    assert(fread(data, 1, (size_t)count, file) == (size_t)count && fclose(file) == 0);
    ShiroRsBytes *out = owned(data, (size_t)count); free(data); return out;
}
static void equal_bytes(ShiroRsBytes *a, ShiroRsBytes *b) {
    uintptr_t n = 0, m = 0; unsigned char *x = copy_bytes(a, &n), *y = copy_bytes(b, &m);
    if (n != m || memcmp(x, y, n) != 0) {
        fprintf(stderr, "Byte mismatch: %zu versus %zu bytes\n", (size_t)n, (size_t)m);
        fwrite(x, 1, n, stderr); fputc('\n', stderr); fwrite(y, 1, m, stderr); fputc('\n', stderr);
        assert(0);
    }
    free(x); free(y);
}
static float *copy_array(ShiroRsArrayF32 *owner, uintptr_t *count) {
    assert(shiro_rs_array_f32_length(owner, count) == 0);
    float *data = malloc((*count + 1) * sizeof(*data)); assert(data);
    assert(shiro_rs_array_f32_copy(owner, 0, data, *count) == 0); return data;
}
static void equal_arrays(ShiroRsArrayF32 *a, ShiroRsArrayF32 *b) {
    uintptr_t n = 0, m = 0; float *x = copy_array(a, &n), *y = copy_array(b, &m);
    assert(n == m && memcmp(x, y, n * sizeof(*x)) == 0); free(x); free(y);
}
static ShiroRsModel *read_model(const char *name) {
    ShiroRsBytes *wire = fixture(name); ShiroRsModel *out = NULL;
    assert(shiro_rs_model_read_bytes(wire, 16 * 1024 * 1024, &out) == 0);
    assert(shiro_rs_bytes_release(&wire) == 0); return out;
}
static void equal_models(ShiroRsModel *a, ShiroRsModel *b) {
    assert((a == NULL) == (b == NULL)); if (!a) return;
    ShiroRsBytes *x = NULL, *y = NULL;
    assert(shiro_rs_model_write_bytes(a, 0, &x) == 0 && shiro_rs_model_write_bytes(b, 0, &y) == 0);
    equal_bytes(x, y); assert(shiro_rs_bytes_release(&x) == 0 && shiro_rs_bytes_release(&y) == 0);
}
static void equal_documents(ShiroRsSegmentationDocument *a, ShiroRsSegmentationDocument *b) {
    ShiroRsBytes *x = NULL, *y = NULL;
    assert(shiro_rs_document_write_json(a, &x) == 0 && shiro_rs_document_write_json(b, &y) == 0);
    equal_bytes(x, y); assert(shiro_rs_bytes_release(&x) == 0 && shiro_rs_bytes_release(&y) == 0);
}
static void original_document(ShiroRsSegmentationDocument *actual, const char *name) {
    ShiroRsBytes *wire = fixture(name); ShiroRsSegmentationDocument *original = NULL;
    assert(shiro_rs_document_read_json(wire, &original) == 0);
    equal_documents(actual, original);
    assert(shiro_rs_document_release(&original) == 0 && shiro_rs_bytes_release(&wire) == 0);
}
typedef struct Fields {
    ShiroRsPhoneMap *phonemap;
    ShiroRsModelDefinition *definition;
    ShiroRsStrings *phones;
    ShiroRsSegmentationDocument *initial;
    ShiroRsModel *models[3];
    ShiroRsIterationReports *reports;
    ShiroRsSegmentationDocument *alignment;
    ShiroRsLabels *labels;
} Fields;
static Fields fields(ShiroRsUtterances *owner) {
    Fields out = {0};
    assert(shiro_rs_utterances_get_phonemap(owner, &out.phonemap) == 0);
    assert(shiro_rs_utterances_get_definition(owner, &out.definition) == 0);
    assert(shiro_rs_utterances_get_phones(owner, &out.phones) == 0);
    assert(shiro_rs_utterances_get_initial_segmentation(owner, &out.initial) == 0);
    assert(shiro_rs_utterances_get_iterations(owner, &out.reports) == 0);
    assert(shiro_rs_utterances_get_alignment(owner, &out.alignment) == 0);
    assert(shiro_rs_utterances_get_labels(owner, &out.labels) == 0);
    for (uint32_t stage = 0; stage < 3; ++stage) {
        uint32_t present = 73; assert(shiro_rs_utterances_has_model(owner, stage, &present) == 0);
        assert(present <= 1);
        assert(shiro_rs_utterances_get_model(owner, stage, &out.models[stage]) == (present ? 0 : 2));
    }
    return out;
}
static void free_fields(Fields *owner) {
    assert(shiro_rs_phone_map_release(&owner->phonemap) == 0);
    assert(shiro_rs_definition_release(&owner->definition) == 0);
    assert(shiro_rs_strings_release(&owner->phones) == 0);
    assert(shiro_rs_document_release(&owner->initial) == 0);
    assert(shiro_rs_document_release(&owner->alignment) == 0);
    assert(shiro_rs_iteration_reports_release(&owner->reports) == 0);
    assert(shiro_rs_labels_release(&owner->labels) == 0);
    for (size_t i = 0; i < 3; ++i) assert(shiro_rs_model_release(&owner->models[i]) == 0);
}
static ShiroRsUtterances *reconstruct(Fields *values) {
    _Alignas(8) ShiroRsUtterancesInput input = {values->phonemap, values->definition, values->phones, values->initial,
        values->models[0], values->models[1], values->models[2], values->reports, values->alignment, values->labels};
    ShiroRsUtterances *out = NULL; assert(shiro_rs_utterances_create(&input, &out) == 0);
    input.model = NULL; ShiroRsUtterances *retained = out;
    assert(shiro_rs_utterances_create(&input, &retained) == 1 && retained == out); return out;
}
static void equal_fields(Fields *a, Fields *b) {
    ShiroRsBytes *x = NULL, *y = NULL;
    assert(shiro_rs_phone_map_write_json(a->phonemap, &x) == 0 && shiro_rs_phone_map_write_json(b->phonemap, &y) == 0);
    equal_bytes(x, y); assert(shiro_rs_bytes_release(&x) == 0 && shiro_rs_bytes_release(&y) == 0);
    assert(shiro_rs_definition_write_json(a->definition, &x) == 0 && shiro_rs_definition_write_json(b->definition, &y) == 0);
    equal_bytes(x, y); assert(shiro_rs_bytes_release(&x) == 0 && shiro_rs_bytes_release(&y) == 0);
    equal_documents(a->initial, b->initial); equal_documents(a->alignment, b->alignment);
    for (size_t stage = 0; stage < 3; ++stage) equal_models(a->models[stage], b->models[stage]);
    uintptr_t n = 0, m = 0;
    assert(shiro_rs_strings_length(a->phones, &n) == 0 && shiro_rs_strings_length(b->phones, &m) == 0 && n == m);
    for (uintptr_t i = 0; i < n; ++i) {
        assert(shiro_rs_strings_get(a->phones, i, &x) == 0 && shiro_rs_strings_get(b->phones, i, &y) == 0);
        equal_bytes(x, y); assert(shiro_rs_bytes_release(&x) == 0 && shiro_rs_bytes_release(&y) == 0);
    }
    assert(shiro_rs_iteration_reports_length(a->reports, &n) == 0 && shiro_rs_iteration_reports_length(b->reports, &m) == 0 && n == m);
    for (uintptr_t i = 0; i < n; ++i) {
        ShiroRsIterationReport *r = NULL, *s = NULL; _Alignas(8) ShiroRsIterationInfo p = {0}, q = {0};
        assert(shiro_rs_iteration_reports_get(a->reports, i, &r) == 0 && shiro_rs_iteration_reports_get(b->reports, i, &s) == 0);
        assert(shiro_rs_iteration_report_info(r, &p) == 0 && shiro_rs_iteration_report_info(s, &q) == 0);
        assert(p.iteration == q.iteration && memcmp(&p.temperature, &q.temperature, sizeof(float)) == 0);
        assert(memcmp(&p.mean_log_likelihood, &q.mean_log_likelihood, sizeof(float)) == 0);
        uintptr_t rows = 0, other = 0;
        assert(shiro_rs_iteration_report_file_count(r, &rows) == 0 && shiro_rs_iteration_report_file_count(s, &other) == 0 && rows == other);
        for (uintptr_t j = 0; j < rows; ++j) {
            ShiroRsArrayF32 *u = NULL, *v = NULL;
            assert(shiro_rs_iteration_report_get_file(r, j, &u) == 0 && shiro_rs_iteration_report_get_file(s, j, &v) == 0);
            equal_arrays(u, v); assert(shiro_rs_array_f32_release(&u) == 0 && shiro_rs_array_f32_release(&v) == 0);
        }
        assert(shiro_rs_iteration_report_release(&r) == 0 && shiro_rs_iteration_report_release(&s) == 0);
    }
    assert(shiro_rs_labels_length(a->labels, &n) == 0 && shiro_rs_labels_length(b->labels, &m) == 0 && n == m);
    for (uintptr_t i = 0; i < n; ++i) {
        _Alignas(8) ShiroRsLabelInfo p = {0}, q = {0};
        assert(shiro_rs_labels_get_info(a->labels, i, &p) == 0 && shiro_rs_labels_get_info(b->labels, i, &q) == 0);
        assert(memcmp(&p.start, &q.start, sizeof(double)) == 0 && memcmp(&p.end, &q.end, sizeof(double)) == 0);
        assert(shiro_rs_labels_get_name(a->labels, i, &x) == 0 && shiro_rs_labels_get_name(b->labels, i, &y) == 0);
        equal_bytes(x, y); assert(shiro_rs_bytes_release(&x) == 0 && shiro_rs_bytes_release(&y) == 0);
    }
}
static void roundtrip(ShiroRsUtterances *owner) {
    Fields expected = fields(owner); ShiroRsUtterances *rebuilt = reconstruct(&expected), *cloned = NULL;
    assert(shiro_rs_utterances_clone(rebuilt, &cloned) == 0 && shiro_rs_utterances_release(&rebuilt) == 0);
    assert(shiro_rs_utterances_release(&owner) == 0);
    Fields actual = fields(cloned); assert(shiro_rs_utterances_release(&cloned) == 0);
    equal_fields(&expected, &actual); free_fields(&expected); free_fields(&actual);
}
static ShiroRsFeatures *fixture_features(void) {
    ShiroRsBytes *wire = fixture("utterances-c-features.bin"); uintptr_t count = 0;
    unsigned char *data = copy_bytes(wire, &count); assert(count == 40 * 13 * sizeof(float));
    ShiroRsArrayF32 *values = NULL; ShiroRsFeatures *out = NULL;
    assert(shiro_rs_array_f32_create((const float *)data, count / sizeof(float), &values) == 0);
    assert(shiro_rs_features_create(40, 13, values, &out) == 0);
    assert(shiro_rs_array_f32_release(&values) == 0 && shiro_rs_bytes_release(&wire) == 0); free(data); return out;
}
static void original_fields(Fields *values, uint32_t mode) {
    const char *models[] = {"utterances-c-uninit.hsmm", "utterances-c-flat.hsmm", "utterances-c-trained.hsmm"};
    for (size_t i = 0; i < 3; ++i) {
        if (i < 2 && mode != 0) { assert(!values->models[i]); continue; }
        ShiroRsModel *expected = read_model(models[i]); equal_models(values->models[i], expected);
        assert(shiro_rs_model_release(&expected) == 0);
    }
    original_document(values->initial, "utterances-c-initial.json");
    original_document(values->alignment, "utterances-c-aligned.json");
    ShiroRsBytes *wire = fixture("utterances-c-phonemap.json"), *expected = NULL, *actual = NULL;
    ShiroRsPhoneMap *map = NULL; assert(shiro_rs_phone_map_read_json(wire, &map) == 0);
    assert(shiro_rs_phone_map_write_json(map, &expected) == 0 && shiro_rs_phone_map_write_json(values->phonemap, &actual) == 0);
    equal_bytes(expected, actual);
    assert(shiro_rs_phone_map_release(&map) == 0 && shiro_rs_bytes_release(&wire) == 0);
    assert(shiro_rs_bytes_release(&expected) == 0 && shiro_rs_bytes_release(&actual) == 0);
    ShiroRsModel *definition_model = NULL, *uninitialized = read_model(models[0]);
    assert(shiro_rs_definition_build(values->definition, &definition_model) == 0);
    equal_models(definition_model, uninitialized);
    assert(shiro_rs_model_release(&definition_model) == 0 && shiro_rs_model_release(&uninitialized) == 0);
    uintptr_t count = 0; assert(shiro_rs_strings_length(values->phones, &count) == 0 && count == 5);
    for (uintptr_t i = 0; i < count; ++i) {
        ShiroRsBytes *name = NULL, *reference = text(i % 2 ? "utt" : "sil");
        assert(shiro_rs_strings_get(values->phones, i, &name) == 0); equal_bytes(name, reference);
        assert(shiro_rs_bytes_release(&name) == 0 && shiro_rs_bytes_release(&reference) == 0);
    }
    assert(shiro_rs_iteration_reports_length(values->reports, &count) == 0 && count == (mode == 2 ? 0 : 2));
    for (uintptr_t i = 0; i < count; ++i) {
        ShiroRsIterationReport *report = NULL; _Alignas(8) ShiroRsIterationInfo info = {0}; ShiroRsArrayF32 *row = NULL;
        assert(shiro_rs_iteration_reports_get(values->reports, i, &report) == 0);
        assert(shiro_rs_iteration_report_info(report, &info) == 0 && info.iteration == i);
        volatile float temperature = (float)sqrt((double)(i + 1) / 2.0);
        assert(info.temperature == temperature);
        uintptr_t rows = 0; assert(shiro_rs_iteration_report_file_count(report, &rows) == 0 && rows == 1);
        assert(shiro_rs_iteration_report_get_file(report, 0, &row) == 0);
        float *data = copy_array(row, &rows); assert(rows == 1);
        volatile float mean = data[0] / temperature; assert(info.mean_log_likelihood == mean); free(data);
        assert(shiro_rs_array_f32_release(&row) == 0 && shiro_rs_iteration_report_release(&report) == 0);
    }
    FILE *labels = fopen("tests/fixtures/utterances-c-labels.txt", "rb"); assert(labels);
    assert(shiro_rs_labels_length(values->labels, &count) == 0);
    for (uintptr_t i = 0; i < count; ++i) {
        _Alignas(8) double start, end; char name[80]; assert(fscanf(labels, "%lf\t%lf\t%79s", &start, &end, name) == 3);
        _Alignas(8) ShiroRsLabelInfo info = {0}; ShiroRsBytes *actual_name = NULL, *reference = text(name);
        assert(shiro_rs_labels_get_info(values->labels, i, &info) == 0);
        assert(fabs(info.start - start) < 1e-14 && fabs(info.end - end) < 1e-14);
        assert(shiro_rs_labels_get_name(values->labels, i, &actual_name) == 0); equal_bytes(actual_name, reference);
        assert(shiro_rs_bytes_release(&actual_name) == 0 && shiro_rs_bytes_release(&reference) == 0);
    }
    char extra[2]; assert(fscanf(labels, "%1s", extra) == EOF && fclose(labels) == 0);
}
typedef struct Draws { ShiroRsDitherSequence *rng; uintptr_t count, fail_after; } Draws;
static uint32_t uniform(void *context, float *out) {
    Draws *state = context; assert(state && out);
    if (state->count == state->fail_after) return 7;
    uint32_t status = shiro_rs_dither_next_uniform(state->rng, out);
    if (!status) ++state->count; return status;
}
static uint32_t le32(const unsigned char *data) {
    return (uint32_t)data[0] | (uint32_t)data[1] << 8 | (uint32_t)data[2] << 16 | (uint32_t)data[3] << 24;
}
static ShiroRsArrayF32 *decoded_wave(uint32_t *rate) {
    ShiroRsBytes *wire = fixture("utterances-input.wav"); uintptr_t n = 0;
    unsigned char *data = copy_bytes(wire, &n);
    assert(n >= 12 && memcmp(data, "RIFF", 4) == 0 && memcmp(data + 8, "WAVE", 4) == 0);
    size_t offset = 12, sample_offset = 0, sample_bytes = 0; int format = 0;
    while (offset + 8 <= n) {
        uint32_t count = le32(data + offset + 4); assert(count <= n - offset - 8);
        const unsigned char *payload = data + offset + 8;
        if (memcmp(data + offset, "fmt ", 4) == 0) {
            assert(count >= 16 && payload[0] == 1 && payload[1] == 0 && payload[2] == 1 && payload[3] == 0);
            assert(payload[14] == 16 && payload[15] == 0); *rate = le32(payload + 4); format = 1;
        } else if (memcmp(data + offset, "data", 4) == 0) { sample_offset = offset + 8; sample_bytes = count; }
        offset += 8 + count + (count & 1);
    }
    assert(format && sample_offset && sample_bytes % 2 == 0);
    float *samples = malloc(sample_bytes / 2 * sizeof(float)); assert(samples);
    for (size_t i = 0; i < sample_bytes / 2; ++i) {
        unsigned integer = data[sample_offset + i * 2] | (unsigned)data[sample_offset + i * 2 + 1] << 8;
        int signed_integer = integer >= 32768 ? (int)integer - 65536 : (int)integer;
        samples[i] = (float)signed_integer / 32768.0f;
    }
    ShiroRsArrayF32 *out = NULL; assert(shiro_rs_array_f32_create(samples, sample_bytes / 2, &out) == 0);
    free(samples); free(data); assert(shiro_rs_bytes_release(&wire) == 0); return out;
}
static void equal_wave(ShiroRsSegmentedWave *a, ShiroRsSegmentedWave *b) {
    ShiroRsAudio *x = NULL, *y = NULL; uint32_t r = 0, s = 0;
    assert(shiro_rs_segmented_wave_get_audio(a, &x) == 0 && shiro_rs_segmented_wave_get_audio(b, &y) == 0);
    assert(shiro_rs_audio_sample_rate(x, &r) == 0 && shiro_rs_audio_sample_rate(y, &s) == 0 && r == s);
    ShiroRsArrayF32 *u = NULL, *v = NULL;
    assert(shiro_rs_audio_get_samples(x, &u) == 0 && shiro_rs_audio_get_samples(y, &v) == 0); equal_arrays(u, v);
    assert(shiro_rs_audio_release(&x) == 0 && shiro_rs_audio_release(&y) == 0);
    assert(shiro_rs_array_f32_release(&u) == 0 && shiro_rs_array_f32_release(&v) == 0);
    ShiroRsFeatures *f = NULL, *g = NULL; _Alignas(8) ShiroRsFeatureInfo p = {0}, q = {0};
    assert(shiro_rs_segmented_wave_get_features(a, &f) == 0 && shiro_rs_segmented_wave_get_features(b, &g) == 0);
    assert(shiro_rs_features_get_info(f, &p) == 0 && shiro_rs_features_get_info(g, &q) == 0);
    assert(p.frames == q.frames && p.columns == q.columns);
    assert(shiro_rs_features_get_values(f, &u) == 0 && shiro_rs_features_get_values(g, &v) == 0); equal_arrays(u, v);
    assert(shiro_rs_features_release(&f) == 0 && shiro_rs_features_release(&g) == 0);
    assert(shiro_rs_array_f32_release(&u) == 0 && shiro_rs_array_f32_release(&v) == 0);
    ShiroRsUtterances *h = NULL, *j = NULL;
    assert(shiro_rs_segmented_wave_get_utterances(a, &h) == 0 && shiro_rs_segmented_wave_get_utterances(b, &j) == 0);
    Fields first = fields(h), second = fields(j);
    assert(shiro_rs_utterances_release(&h) == 0 && shiro_rs_utterances_release(&j) == 0);
    equal_fields(&first, &second); free_fields(&first); free_fields(&second);
}
static void original_wave(ShiroRsAudio *audio, ShiroRsFeatures *features) {
    uint32_t rate = 0; assert(shiro_rs_audio_sample_rate(audio, &rate) == 0 && rate == 16000);
    ShiroRsArrayF32 *values = NULL; assert(shiro_rs_audio_get_samples(audio, &values) == 0);
    uintptr_t n = 0, m = 0; float *actual = copy_array(values, &n);
    ShiroRsBytes *wire = fixture("utterances-c-audio.bin"); unsigned char *reference = copy_bytes(wire, &m);
    assert(m == n * sizeof(float) && memcmp(actual, reference, m) == 0);
    free(actual); free(reference); assert(shiro_rs_array_f32_release(&values) == 0 && shiro_rs_bytes_release(&wire) == 0);
    _Alignas(8) ShiroRsFeatureInfo info = {0}; assert(shiro_rs_features_get_info(features, &info) == 0 && info.frames == 40 && info.columns == 13);
    assert(shiro_rs_features_get_values(features, &values) == 0); actual = copy_array(values, &n);
    wire = fixture("utterances-c-features.bin"); reference = copy_bytes(wire, &m); assert(m == n * sizeof(float));
    for (uintptr_t i = 0; i < n; ++i) {
        float expected; memcpy(&expected, reference + i * sizeof(float), sizeof(float));
        assert(fabs((double)actual[i] - expected) / fmax(fabs(expected), 1.0) <= 2e-5);
    }
    free(actual); free(reference); assert(shiro_rs_array_f32_release(&values) == 0 && shiro_rs_bytes_release(&wire) == 0);
}
static uint32_t float_bits(float value) { uint32_t out; memcpy(&out, &value, sizeof(out)); return out; }
static uint64_t double_bits(double value) { uint64_t out; memcpy(&out, &value, sizeof(out)); return out; }
static void expect_text(ShiroRsBytes *actual, const char *value) {
    ShiroRsBytes *expected = text(value); equal_bytes(actual, expected); assert(shiro_rs_bytes_release(&expected) == 0);
}
static void arbitrary_results(void) {
    const uint32_t weight_bits = UINT32_C(0x7fc12345), row_bits[] = {UINT32_C(0x80000000), UINT32_C(0x7fc54321), UINT32_C(0x7f800000), 1};
    const uint64_t time_bits = UINT64_C(0x7ff8123456789abc), end_bits = UINT64_C(0xfff0000000000000);
    const char *map_text = "{\"phone_map\":{},\"nested\":{\"data\":[null,true]}}";
    const char *state_fields[] = {"[[0.25,{\"name\":\"voice\"}]]", "[null,\"voice\",42,{\"nested\":[true]}]", "{\"dur\":false,\"time\":\"shadow\"}"};
    const char *file_text = "{\"filename\":\"shadow\",\"states\":false}";
    const char *document_text = "{\"file_list\":\"shadow\",\"nested\":[null,7]}";
    ShiroRsBytes *wire = text(map_text); ShiroRsPhoneMap *map = NULL;
    assert(shiro_rs_phone_map_read_json(wire, &map) == 0 && shiro_rs_bytes_release(&wire) == 0);
    _Alignas(8) ShiroRsStreamDefinition stream = {UINTPTR_MAX, 0, UINTPTR_MAX, 0}; memcpy(&stream.weight, &weight_bits, sizeof(float));
    _Alignas(8) ShiroRsDurationConstraint constraint = {UINTPTR_MAX, 1, INT32_MIN, 1, 0}; ShiroRsModelDefinition *definition = NULL;
    assert(shiro_rs_definition_create(UINTPTR_MAX, &stream, 1, &constraint, 1, &definition) == 0);
    const char raw_name[] = "voice\0.param";
    ShiroRsBytes *name = owned(raw_name, sizeof(raw_name) - 1), *empty_name = owned(NULL, 0);
    const ShiroRsBytes *names[] = {empty_name, name, empty_name}; ShiroRsStrings *phones = NULL;
    assert(shiro_rs_strings_create(names, 3, &phones) == 0 && shiro_rs_bytes_release(&empty_name) == 0);
    ShiroRsBytes *json_fields[3]; for (size_t i = 0; i < 3; ++i) json_fields[i] = text(state_fields[i]);
    uintptr_t integers[] = {0, UINTPTR_MAX}; ShiroRsArrayUsize *outputs = NULL;
    assert(shiro_rs_array_usize_create(integers, 2, &outputs) == 0);
    _Alignas(8) ShiroRsStateInput state = {0, 1, UINTPTR_MAX, outputs, json_fields[0], json_fields[1], json_fields[2]};
    memcpy(&state.time, &time_bits, sizeof(double)); ShiroRsStates *states = NULL;
    assert(shiro_rs_states_create(&state, 1, &states) == 0 && shiro_rs_array_usize_release(&outputs) == 0);
    for (size_t i = 0; i < 3; ++i) assert(shiro_rs_bytes_release(&json_fields[i]) == 0);
    ShiroRsSegmentedFile *file = NULL; wire = text(file_text);
    assert(shiro_rs_segmented_file_create(name, states, wire, &file) == 0);
    assert(shiro_rs_states_release(&states) == 0 && shiro_rs_bytes_release(&wire) == 0);
    const ShiroRsSegmentedFile *files[] = {file, file}; ShiroRsSegmentationDocument *document = NULL; wire = text(document_text);
    assert(shiro_rs_document_create(files, 2, wire, &document) == 0);
    assert(shiro_rs_segmented_file_release(&file) == 0 && shiro_rs_bytes_release(&wire) == 0);
    float row_values[4]; memcpy(row_values, row_bits, sizeof(row_values));
    ShiroRsArrayF32 *row = NULL, *empty_row = NULL;
    assert(shiro_rs_array_f32_create(row_values, 4, &row) == 0 && shiro_rs_array_f32_create(NULL, 0, &empty_row) == 0);
    _Alignas(8) ShiroRsIterationInfo iteration = {UINTPTR_MAX, 0, 0}; const uint32_t mean_bits = UINT32_C(0xff800000);
    memcpy(&iteration.temperature, &weight_bits, sizeof(float)); memcpy(&iteration.mean_log_likelihood, &mean_bits, sizeof(float));
    const ShiroRsArrayF32 *rows[] = {row, empty_row, row}; ShiroRsIterationReport *report = NULL;
    assert(shiro_rs_iteration_report_create(&iteration, rows, 3, &report) == 0);
    const ShiroRsIterationReport *reports[] = {report, report}; ShiroRsIterationReports *collection = NULL;
    assert(shiro_rs_iteration_reports_create(reports, 2, &collection) == 0);
    assert(shiro_rs_iteration_report_release(&report) == 0 && shiro_rs_array_f32_release(&row) == 0 && shiro_rs_array_f32_release(&empty_row) == 0);
    _Alignas(8) ShiroRsLabelInput label = {0, 0, name}; memcpy(&label.start, &time_bits, sizeof(double)); memcpy(&label.end, &end_bits, sizeof(double));
    ShiroRsLabels *labels = NULL; assert(shiro_rs_labels_create(&label, 1, &labels) == 0 && shiro_rs_bytes_release(&name) == 0);
    ShiroRsModel *model = read_model("utterances-c-trained.hsmm"); ShiroRsUtterances *owners[4] = {0};
    for (size_t mask = 0; mask < 4; ++mask) {
        _Alignas(8) ShiroRsUtterancesInput input = {map, definition, phones, document, mask & 1 ? model : NULL, mask & 2 ? model : NULL,
            model, collection, document, labels}; ShiroRsUtterances *out = NULL;
        assert(shiro_rs_utterances_create(&input, &out) == 0 && shiro_rs_utterances_clone(out, &owners[mask]) == 0);
        assert(shiro_rs_utterances_release(&out) == 0);
    }
    assert(shiro_rs_phone_map_release(&map) == 0 && shiro_rs_definition_release(&definition) == 0 && shiro_rs_strings_release(&phones) == 0);
    assert(shiro_rs_document_release(&document) == 0 && shiro_rs_iteration_reports_release(&collection) == 0);
    assert(shiro_rs_labels_release(&labels) == 0 && shiro_rs_model_release(&model) == 0);
    ShiroRsAudio *audio = NULL; ShiroRsFeatures *features = NULL;
    assert(shiro_rs_array_f32_create(row_values, 4, &row) == 0);
    assert(shiro_rs_audio_create(0, row, &audio) == 0 && shiro_rs_features_create(UINTPTR_MAX, 0, row, &features) == 0);
    ShiroRsSegmentedWave *out = NULL, *cloned = NULL;
    assert(shiro_rs_segmented_wave_create(audio, features, owners[3], &out) == 0 && shiro_rs_segmented_wave_clone(out, &cloned) == 0);
    assert(shiro_rs_audio_release(&audio) == 0 && shiro_rs_features_release(&features) == 0 && shiro_rs_array_f32_release(&row) == 0);
    assert(shiro_rs_utterances_release(&owners[3]) == 0);
    assert(shiro_rs_segmented_wave_release(&out) == 0);
    assert(shiro_rs_segmented_wave_get_audio(cloned, &audio) == 0 && shiro_rs_segmented_wave_get_features(cloned, &features) == 0);
    ShiroRsUtterances *result = NULL; assert(shiro_rs_segmented_wave_get_utterances(cloned, &result) == 0 && shiro_rs_segmented_wave_release(&cloned) == 0);
    uint32_t rate = 99; _Alignas(8) ShiroRsFeatureInfo shape = {0};
    assert(shiro_rs_audio_sample_rate(audio, &rate) == 0 && rate == 0 && shiro_rs_features_get_info(features, &shape) == 0 && shape.frames == UINTPTR_MAX && shape.columns == 0);
    assert(shiro_rs_audio_get_samples(audio, &row) == 0); uintptr_t count = 0; float *data = copy_array(row, &count);
    assert(count == 4 && memcmp(data, row_bits, sizeof(row_bits)) == 0); free(data); assert(shiro_rs_array_f32_release(&row) == 0);
    assert(shiro_rs_features_get_values(features, &row) == 0); data = copy_array(row, &count);
    assert(count == 4 && memcmp(data, row_bits, sizeof(row_bits)) == 0); free(data); assert(shiro_rs_array_f32_release(&row) == 0);
    assert(shiro_rs_audio_release(&audio) == 0 && shiro_rs_features_release(&features) == 0);
    owners[3] = result;
    for (size_t mask = 0; mask < 4; ++mask) {
        Fields values = fields(owners[mask]);
        assert(shiro_rs_phone_map_write_json(values.phonemap, &wire) == 0); expect_text(wire, map_text); assert(shiro_rs_bytes_release(&wire) == 0);
        _Alignas(8) ShiroRsDefinitionInfo info = {0}; _Alignas(8) ShiroRsStreamDefinition actual_stream = {0}; _Alignas(8) ShiroRsDurationConstraint actual_constraint = {0};
        assert(shiro_rs_definition_get_info(values.definition, &info) == 0 && info.duration_states == UINTPTR_MAX && info.streams == 1 && info.duration_constraints == 1);
        assert(shiro_rs_definition_get_stream(values.definition, 0, &actual_stream) == 0);
        assert(actual_stream.states == UINTPTR_MAX && actual_stream.dimensions == 0 && actual_stream.mixtures == UINTPTR_MAX && float_bits(actual_stream.weight) == weight_bits);
        assert(shiro_rs_definition_get_constraint(values.definition, 0, &actual_constraint) == 0);
        assert(actual_constraint.index == UINTPTR_MAX && actual_constraint.has_minimum == 1 && actual_constraint.minimum == INT32_MIN && actual_constraint.has_maximum == 1 && actual_constraint.maximum == 0);
        uintptr_t count = 0; assert(shiro_rs_strings_length(values.phones, &count) == 0 && count == 3);
        for (uintptr_t i = 0; i < count; ++i) {
            assert(shiro_rs_strings_get(values.phones, i, &wire) == 0); ShiroRsBytes *expected = owned(raw_name, i == 1 ? sizeof(raw_name) - 1 : 0);
            equal_bytes(wire, expected); assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected) == 0);
        }
        ShiroRsSegmentationDocument *documents[] = {values.initial, values.alignment};
        for (size_t which = 0; which < 2; ++which) {
            assert(shiro_rs_document_length(documents[which], &count) == 0 && count == 2);
            assert(shiro_rs_document_get_attributes(documents[which], &wire) == 0); expect_text(wire, document_text); assert(shiro_rs_bytes_release(&wire) == 0);
            for (uintptr_t i = 0; i < 2; ++i) {
                assert(shiro_rs_document_get_file(documents[which], i, &file) == 0);
                assert(shiro_rs_segmented_file_get_filename(file, &wire) == 0); ShiroRsBytes *expected = owned(raw_name, sizeof(raw_name) - 1);
                equal_bytes(wire, expected); assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected) == 0);
                assert(shiro_rs_segmented_file_get_attributes(file, &wire) == 0); expect_text(wire, file_text); assert(shiro_rs_bytes_release(&wire) == 0);
                assert(shiro_rs_segmented_file_get_states(file, &states) == 0 && shiro_rs_segmented_file_release(&file) == 0);
                assert(shiro_rs_states_length(states, &count) == 0 && count == 1); _Alignas(8) ShiroRsStateInfo state_info = {0};
                assert(shiro_rs_states_get_info(states, 0, &state_info) == 0);
                assert(double_bits(state_info.time) == time_bits && state_info.has_duration == 1 && state_info.duration == UINTPTR_MAX && state_info.has_outputs == 1 && state_info.has_jumps == 1);
                assert(shiro_rs_states_get_outputs(states, 0, &outputs) == 0); uintptr_t actual[2] = {0};
                assert(shiro_rs_array_usize_length(outputs, &count) == 0 && count == 2 && shiro_rs_array_usize_copy(outputs, 0, actual, 2) == 0);
                assert(actual[0] == 0 && actual[1] == UINTPTR_MAX && shiro_rs_array_usize_release(&outputs) == 0);
                for (uint32_t j = 0; j < 3; ++j) {
                    assert(shiro_rs_states_get_json_field(states, 0, j, &wire) == 0); expect_text(wire, state_fields[j]); assert(shiro_rs_bytes_release(&wire) == 0);
                }
                assert(shiro_rs_states_release(&states) == 0);
            }
            wire = text("retained"); ShiroRsBytes *retained = wire;
            assert(shiro_rs_document_write_json(documents[which], &retained) == 3 && retained == wire && shiro_rs_bytes_release(&wire) == 0);
        }
        ShiroRsModel *expected = read_model("utterances-c-trained.hsmm");
        assert((values.models[0] != NULL) == ((mask & 1) != 0) && (values.models[1] != NULL) == ((mask & 2) != 0));
        for (size_t i = 0; i < 3; ++i) if (values.models[i]) equal_models(values.models[i], expected);
        assert(shiro_rs_model_release(&expected) == 0);
        assert(shiro_rs_iteration_reports_length(values.reports, &count) == 0 && count == 2);
        for (uintptr_t i = 0; i < 2; ++i) {
            assert(shiro_rs_iteration_reports_get(values.reports, i, &report) == 0);
            _Alignas(8) ShiroRsIterationInfo actual = {0}; assert(shiro_rs_iteration_report_info(report, &actual) == 0);
            assert(actual.iteration == UINTPTR_MAX && float_bits(actual.temperature) == weight_bits && float_bits(actual.mean_log_likelihood) == mean_bits);
            assert(shiro_rs_iteration_report_file_count(report, &count) == 0 && count == 3);
            for (uintptr_t j = 0; j < 3; ++j) {
                assert(shiro_rs_iteration_report_get_file(report, j, &row) == 0);
                float *data = copy_array(row, &count); assert(count == (j == 1 ? 0 : 4));
                assert(memcmp(data, row_bits, count * sizeof(float)) == 0); free(data); assert(shiro_rs_array_f32_release(&row) == 0);
            }
            assert(shiro_rs_iteration_report_release(&report) == 0);
        }
        assert(shiro_rs_labels_length(values.labels, &count) == 0 && count == 1); _Alignas(8) ShiroRsLabelInfo label_info = {0};
        assert(shiro_rs_labels_get_info(values.labels, 0, &label_info) == 0 && double_bits(label_info.start) == time_bits && double_bits(label_info.end) == end_bits);
        assert(shiro_rs_labels_get_name(values.labels, 0, &wire) == 0); ShiroRsBytes *expected_name = owned(raw_name, sizeof(raw_name) - 1);
        equal_bytes(wire, expected_name); assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected_name) == 0);
        free_fields(&values);
    }
    for (size_t i = 0; i < 4; ++i) assert(shiro_rs_utterances_release(&owners[i]) == 0);
}
int main(void) {
    arbitrary_results();
    _Alignas(8) ShiroRsUtteranceOptions options = {0}; assert(shiro_rs_utterance_options_default(&options) == 0);
    assert(options.utterances == 1 && options.hop_seconds == 0.1 && options.minimum_silence_seconds == 0.3);
    assert(options.minimum_voicing_seconds == 0.3 && options.iterations == 15);
    options.utterances = 2; options.iterations = 2;
    ShiroRsFeatures *features = fixture_features(); ShiroRsBytes *filename = text("sample.param");
    for (uint32_t mode = 0; mode < 3; ++mode) {
        ShiroRsModel *model = mode ? read_model(mode == 1 ? "utterances-c-flat.hsmm" : "utterances-c-trained.hsmm") : NULL;
        _Alignas(8) ShiroRsModelSource source = {mode, model}; ShiroRsUtterances *out = NULL;
        assert(shiro_rs_utterances_split_features(features, filename, &options, &source, &out) == 0);
        assert(shiro_rs_model_release(&model) == 0);
        Fields values = fields(out); original_fields(&values, mode); free_fields(&values);
        uint32_t present = 73; assert(shiro_rs_utterances_has_model(out, 3, &present) == 2 && present == 73);
        ShiroRsUtterances *retained = out; _Alignas(8) ShiroRsUtteranceOptions invalid = options; invalid.hop_seconds = 0;
        _Alignas(8) ShiroRsModelSource fresh = {0, NULL};
        assert(shiro_rs_utterances_split_features(features, filename, &invalid, &fresh, &retained) == 3 && retained == out);
        roundtrip(out);
    }
    _Alignas(8) const ShiroRsUtteranceOptions nondefaults[] = {{2, 0.1, 0.3, 0.3, 0}, {1, 0.125, 0.25, 0.375, 1}};
    const char extended_name[] = "audio-\xf0\x9f\x8e\xb5\0.param";
    ShiroRsBytes *unicode_name = owned(extended_name, sizeof(extended_name) - 1);
    for (size_t i = 0; i < 2; ++i) {
        _Alignas(8) ShiroRsModelSource source = {0, NULL}; ShiroRsUtterances *out = NULL;
        assert(shiro_rs_utterances_split_features(features, unicode_name, &nondefaults[i], &source, &out) == 0);
        Fields values = fields(out); uintptr_t count = 0;
        assert(shiro_rs_strings_length(values.phones, &count) == 0 && count == nondefaults[i].utterances * 2 + 1);
        assert(shiro_rs_iteration_reports_length(values.reports, &count) == 0 && count == nondefaults[i].iterations);
        ShiroRsSegmentationDocument *documents[] = {values.initial, values.alignment};
        for (size_t j = 0; j < 2; ++j) {
            ShiroRsSegmentedFile *file = NULL; ShiroRsBytes *name = NULL;
            assert(shiro_rs_document_get_file(documents[j], 0, &file) == 0);
            assert(shiro_rs_segmented_file_get_filename(file, &name) == 0); equal_bytes(name, unicode_name);
            assert(shiro_rs_segmented_file_release(&file) == 0 && shiro_rs_bytes_release(&name) == 0);
        }
        free_fields(&values); roundtrip(out);
    }
    assert(shiro_rs_bytes_release(&unicode_name) == 0);
    assert(shiro_rs_features_release(&features) == 0);
    uint32_t rate = 0; ShiroRsArrayF32 *samples = decoded_wave(&rate);
    for (uint32_t kind = 0; kind < 3; ++kind) {
        Draws draws = {NULL, 0, UINTPTR_MAX}; assert(shiro_rs_dither_linux_gnu(&draws.rng) == 0);
        _Alignas(8) ShiroRsWaveSplitInput input = {{rate, 16, 1, 0}, samples, filename, 13, kind};
        _Alignas(8) ShiroRsModelSource source = {0, NULL}; ShiroRsSegmentedWave *out = NULL, *rebuilt = NULL, *cloned = NULL;
        assert(shiro_rs_utterances_split_wave(&input, &options, &source, uniform, &draws, &out) == 0 && draws.count > 0);
        assert(shiro_rs_dither_release(&draws.rng) == 0);
        ShiroRsAudio *audio = NULL; ShiroRsFeatures *extracted = NULL; ShiroRsUtterances *result = NULL;
        assert(shiro_rs_segmented_wave_get_audio(out, &audio) == 0 && shiro_rs_segmented_wave_get_features(out, &extracted) == 0);
        assert(shiro_rs_segmented_wave_get_utterances(out, &result) == 0);
        if (kind == 0) original_wave(audio, extracted);
        assert(shiro_rs_segmented_wave_create(audio, extracted, result, &rebuilt) == 0);
        assert(shiro_rs_audio_release(&audio) == 0 && shiro_rs_features_release(&extracted) == 0 && shiro_rs_utterances_release(&result) == 0);
        assert(shiro_rs_segmented_wave_clone(rebuilt, &cloned) == 0 && shiro_rs_segmented_wave_release(&rebuilt) == 0);
        equal_wave(out, cloned); assert(shiro_rs_segmented_wave_release(&out) == 0 && shiro_rs_segmented_wave_release(&cloned) == 0);
        draws.count = 0; draws.fail_after = 3; assert(shiro_rs_dither_linux_gnu(&draws.rng) == 0);
        assert(shiro_rs_utterances_split_wave(&input, &options, &source, uniform, &draws, NULL) == 1 && draws.count == 0);
        input.dimensions = 1;
        assert(shiro_rs_utterances_split_wave(&input, &options, &source, uniform, &draws, &out) == 3 && draws.count == 0);
        input.dimensions = 13;
        assert(shiro_rs_utterances_split_wave(&input, &options, &source, uniform, &draws, &out) == 3 && !out && draws.count == 3);
        assert(shiro_rs_dither_release(&draws.rng) == 0);
    }
    for (uint32_t mode = 0; mode < 3; ++mode) {
        const char *model_name = mode == 1 ? "utterances-c-flat.hsmm" : "utterances-c-trained.hsmm";
        ShiroRsModel *model = read_model(model_name), *expected = read_model(model_name);
        for (uint32_t modified = 0; modified < 2; ++modified) {
            Draws draws = {NULL, 0, UINTPTR_MAX}; assert(shiro_rs_dither_linux_gnu(&draws.rng) == 0);
            _Alignas(8) ShiroRsWaveSplitInput input = {{modified ? rate / 2 : rate, modified ? 0 : 16, modified ? UINT16_MAX : 1, modified}, samples, filename, 13, 0};
            _Alignas(8) ShiroRsModelSource source = {mode, model}; ShiroRsSegmentedWave *out = NULL;
            assert(shiro_rs_utterances_split_wave(&input, &nondefaults[0], &source, uniform, &draws, &out) == 0 && draws.count > 0);
            assert(shiro_rs_dither_release(&draws.rng) == 0); equal_models(model, expected);
            ShiroRsUtterances *result = NULL; assert(shiro_rs_segmented_wave_get_utterances(out, &result) == 0);
            assert(shiro_rs_segmented_wave_release(&out) == 0);
            Fields values = fields(result); uintptr_t count = 0;
            assert(shiro_rs_iteration_reports_length(values.reports, &count) == 0 && count == 0);
            assert((values.models[0] != NULL) == (mode == 0) && (values.models[1] != NULL) == (mode == 0));
            if (mode) equal_models(values.models[2], expected);
            free_fields(&values); roundtrip(result);
        }
        assert(shiro_rs_model_release(&model) == 0 && shiro_rs_model_release(&expected) == 0);
    }
    assert(shiro_rs_array_f32_release(&samples) == 0 && shiro_rs_bytes_release(&filename) == 0);
    puts("SHIRO utterances actual C: all21 exports, full finite fields and decoded-wave ownership pass");
    return 0;
}
