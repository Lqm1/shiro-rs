/* Actual C complete definitions and ordered iteration reports. */
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
    assert(shiro_rs_bytes_create(data, count, &output) == 0);
    return output;
}
static ShiroRsBytes *fixture(const char *name) {
    char path[160];
    assert(snprintf(path, sizeof(path), "tests/fixtures/%s", name) > 0);
    FILE *file = fopen(path, "rb");
    assert(file && fseek(file, 0, SEEK_END) == 0);
    long count = ftell(file);
    assert(count >= 0 && fseek(file, 0, SEEK_SET) == 0);
    void *data = malloc((size_t)count + 1);
    assert(data && fread(data, 1, (size_t)count, file) == (size_t)count);
    assert(fclose(file) == 0);
    ShiroRsBytes *output = owned(data, (size_t)count);
    free(data);
    return output;
}
static void equal(ShiroRsBytes *left, ShiroRsBytes *right) {
    size_t a = 0, b = 0;
    assert(shiro_rs_bytes_length(left, &a) == 0 && shiro_rs_bytes_length(right, &b) == 0 && a == b);
    unsigned char *x = malloc(a + 1), *y = malloc(b + 1);
    assert(x && y);
    assert(shiro_rs_bytes_copy(left, 0, x, a) == 0 && shiro_rs_bytes_copy(right, 0, y, b) == 0);
    assert(memcmp(x, y, a) == 0);
    free(x); free(y);
}
static float value(uint32_t bits) {
    float output; memcpy(&output, &bits, sizeof(output)); return output;
}
static uint32_t bits(float input) {
    uint32_t output; memcpy(&output, &input, sizeof(output)); return output;
}
int main(void) {
    const char *definitions[] = {"modeldef.json", "utterances-c-definition.json", "init-definition.json"};
    const char *models[] = {"empty-c.hsmm", "utterances-c-uninit.hsmm", NULL};
    const size_t durations[] = {3, 2, 4}, stream_counts[] = {2, 1, 2}, constraint_counts[] = {1, 2, 0};
    for (size_t i = 0; i < 3; ++i) {
        ShiroRsBytes *source = fixture(definitions[i]);
        ShiroRsModelDefinition *definition = NULL, *snapshot = NULL, *reread = NULL;
        assert(shiro_rs_definition_read_json(source, &definition) == 0 && shiro_rs_bytes_release(&source) == 0);
        assert(shiro_rs_definition_clone(definition, &snapshot) == 0 && shiro_rs_definition_release(&definition) == 0);
        ShiroRsDefinitionInfo info = {0};
        assert(shiro_rs_definition_get_info(snapshot, &info) == 0);
        assert(info.duration_states == durations[i] && info.streams == stream_counts[i] && info.duration_constraints == constraint_counts[i]);
        for (size_t j = 0; j < info.streams; ++j) {
            ShiroRsStreamDefinition stream = {0};
            assert(shiro_rs_definition_get_stream(snapshot, j, &stream) == 0);
            assert(stream.states == (i == 1 ? 2 : i == 0 && j == 1 ? 2 : 3));
            assert(stream.dimensions == (i == 1 ? 13 : j == 0 ? 2 : 1));
            assert(stream.mixtures == (i == 0 && j == 1 ? 2 : 1));
            assert(bits(stream.weight) == bits(i == 0 && j == 1 ? 0.5f : 1.0f));
        }
        for (size_t j = 0; j < info.duration_constraints; ++j) {
            ShiroRsDurationConstraint constraint = {0};
            assert(shiro_rs_definition_get_constraint(snapshot, j, &constraint) == 0);
            assert(constraint.index == (i == 0 ? 1 : j == 0 ? 1 : 0));
            assert(constraint.has_minimum == 1 && constraint.minimum == (i == 0 ? 2 : 3));
            assert(constraint.has_maximum == (i == 0 ? 1u : 0u) && constraint.maximum == (i == 0 ? 10 : 0));
        }
        ShiroRsBytes *encoded = NULL, *wire = NULL, *other = NULL;
        ShiroRsModel *model = NULL, *second = NULL;
        assert(shiro_rs_definition_write_json(snapshot, &encoded) == 0 && shiro_rs_definition_read_json(encoded, &reread) == 0);
        assert(shiro_rs_definition_build(snapshot, &model) == 0 && shiro_rs_definition_build(reread, &second) == 0);
        assert(shiro_rs_model_write_bytes(model, 0, &wire) == 0 && shiro_rs_model_write_bytes(second, 0, &other) == 0);
        equal(wire, other);
        if (models[i]) {
            ShiroRsBytes *original = fixture(models[i]); equal(wire, original);
            assert(shiro_rs_bytes_release(&original) == 0);
        }
        assert(shiro_rs_bytes_release(&encoded) == 0 && shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&other) == 0);
        assert(shiro_rs_model_release(&model) == 0 && shiro_rs_model_release(&second) == 0);
        assert(shiro_rs_definition_release(&snapshot) == 0 && shiro_rs_definition_release(&reread) == 0);
    }
    const uint32_t patterns[] = {0x80000000u, 0x7fc12345u, 0x7f800000u, 0xff800000u, 1};
    ShiroRsStreamDefinition streams[5];
    for (size_t i = 0; i < 5; ++i) {
        streams[i] = (ShiroRsStreamDefinition){SIZE_MAX, 0, SIZE_MAX - 1, value(patterns[i])};
    }
    const ShiroRsDurationConstraint constraints[] = {{SIZE_MAX, 0, 99, 1, 0}, {0, 1, INT32_MIN, 0, 99}, {1, 1, 0, 1, INT32_MAX}};
    ShiroRsModelDefinition *definition = NULL, *snapshot = NULL;
    assert(shiro_rs_definition_create(SIZE_MAX, streams, 5, constraints, 3, &definition) == 0);
    assert(shiro_rs_definition_clone(definition, &snapshot) == 0 && shiro_rs_definition_release(&definition) == 0);
    ShiroRsDefinitionInfo info = {0};
    assert(shiro_rs_definition_get_info(snapshot, &info) == 0 && info.duration_states == SIZE_MAX && info.streams == 5 && info.duration_constraints == 3);
    for (size_t i = 0; i < 5; ++i) {
        ShiroRsStreamDefinition actual = {0};
        assert(shiro_rs_definition_get_stream(snapshot, i, &actual) == 0);
        assert(actual.states == SIZE_MAX && actual.dimensions == 0 && actual.mixtures == SIZE_MAX - 1 && bits(actual.weight) == patterns[i]);
    }
    for (size_t i = 0; i < 3; ++i) {
        ShiroRsDurationConstraint actual = {0};
        assert(shiro_rs_definition_get_constraint(snapshot, i, &actual) == 0);
        assert(actual.index == constraints[i].index && actual.has_minimum == constraints[i].has_minimum && actual.has_maximum == constraints[i].has_maximum);
        assert(actual.minimum == (actual.has_minimum ? constraints[i].minimum : 0) && actual.maximum == (actual.has_maximum ? constraints[i].maximum : 0));
    }
    ShiroRsBytes *sentinel = owned("retained", 8), *retained_bytes = sentinel;
    assert(shiro_rs_definition_write_json(snapshot, &retained_bytes) == 3 && retained_bytes == sentinel);
    ShiroRsModel *invalid_model = NULL;
    assert(shiro_rs_definition_build(snapshot, &invalid_model) == 3 && !invalid_model);
    ShiroRsModelDefinition *retained = snapshot;
    ShiroRsDurationConstraint invalid = {0, 2, 0, 0, 0};
    assert(shiro_rs_definition_create(0, NULL, 0, &invalid, 1, &retained) == 2 && retained == snapshot);
    assert(shiro_rs_definition_create(0, NULL, 1, NULL, 0, &retained) == 1 && retained == snapshot);
    ShiroRsStreamDefinition unchanged = streams[0];
    assert(shiro_rs_definition_get_stream(snapshot, 5, &unchanged) == 2 && bits(unchanged.weight) == patterns[0]);
    assert(shiro_rs_definition_clone(snapshot, NULL) == 1 && shiro_rs_definition_get_info(snapshot, NULL) == 1);
    assert(shiro_rs_bytes_release(&sentinel) == 0 && shiro_rs_definition_release(&snapshot) == 0 && shiro_rs_definition_release(&snapshot) == 0);
    assert(shiro_rs_definition_create(0, NULL, 0, NULL, 0, &definition) == 0);
    assert(shiro_rs_definition_get_info(definition, &info) == 0 && info.duration_states == 0 && info.streams == 0 && info.duration_constraints == 0);
    assert(shiro_rs_definition_release(&definition) == 0);

    float values[5]; for (size_t i = 0; i < 5; ++i) values[i] = value(patterns[i]);
    ShiroRsArrayF32 *row = NULL, *empty = NULL;
    assert(shiro_rs_array_f32_create(values, 5, &row) == 0 && shiro_rs_array_f32_create(NULL, 0, &empty) == 0);
    ShiroRsIterationInfo iteration = {SIZE_MAX, value(0x7fc54321u), value(0xff800000u)};
    const ShiroRsArrayF32 *rows[] = {row, empty, row};
    ShiroRsIterationReport *report = NULL, *blank = NULL;
    assert(shiro_rs_iteration_report_create(&iteration, rows, 3, &report) == 0 && shiro_rs_iteration_report_create(&iteration, NULL, 0, &blank) == 0);
    assert(shiro_rs_array_f32_release(&row) == 0 && shiro_rs_array_f32_release(&empty) == 0);
    const ShiroRsIterationReport *inputs[] = {report, blank, report};
    ShiroRsIterationReports *collection = NULL, *copy = NULL;
    assert(shiro_rs_iteration_reports_create(inputs, 3, &collection) == 0);
    assert(shiro_rs_iteration_report_release(&report) == 0 && shiro_rs_iteration_report_release(&blank) == 0);
    assert(shiro_rs_iteration_reports_clone(collection, &copy) == 0 && shiro_rs_iteration_reports_release(&collection) == 0);
    size_t count = 99;
    assert(shiro_rs_iteration_reports_length(copy, &count) == 0 && count == 3);
    ShiroRsIterationReport *saved[3] = {NULL, NULL, NULL};
    for (size_t i = 0; i < 3; ++i) assert(shiro_rs_iteration_reports_get(copy, i, &saved[i]) == 0);
    report = saved[0];
    assert(shiro_rs_iteration_reports_get(copy, 3, &report) == 2 && report == saved[0]);
    const ShiroRsIterationReport *invalid_reports[] = {NULL};
    ShiroRsIterationReports *retained_collection = copy;
    assert(shiro_rs_iteration_reports_create(invalid_reports, 1, &retained_collection) == 1 && retained_collection == copy);
    assert(shiro_rs_iteration_report_create(&iteration, NULL, 1, &report) == 1 && report == saved[0]);
    assert(shiro_rs_iteration_reports_release(&copy) == 0);
    for (size_t i = 0; i < 3; ++i) {
        ShiroRsIterationInfo actual = {0};
        assert(shiro_rs_iteration_report_info(saved[i], &actual) == 0);
        assert(actual.iteration == SIZE_MAX && bits(actual.temperature) == 0x7fc54321u && bits(actual.mean_log_likelihood) == 0xff800000u);
        assert(shiro_rs_iteration_report_file_count(saved[i], &count) == 0 && count == (i == 1 ? 0 : 3));
        for (size_t j = 0; j < count; ++j) {
            assert(shiro_rs_iteration_report_get_file(saved[i], j, &row) == 0);
            size_t length = 99;
            assert(shiro_rs_array_f32_length(row, &length) == 0 && length == (j == 1 ? 0 : 5));
            float output[5] = {0};
            assert(shiro_rs_array_f32_copy(row, 0, output, length) == 0);
            for (size_t k = 0; k < length; ++k) assert(bits(output[k]) == patterns[k]);
            assert(shiro_rs_array_f32_release(&row) == 0);
        }
        assert(shiro_rs_iteration_report_release(&saved[i]) == 0);
    }
    assert(shiro_rs_iteration_reports_create(NULL, 0, &collection) == 0);
    assert(shiro_rs_iteration_reports_length(collection, &count) == 0 && count == 0);
    assert(shiro_rs_iteration_reports_release(&collection) == 0 && shiro_rs_iteration_reports_release(&collection) == 0);
    puts("SHIRO definitions/reports C: all15 exports, original models, full fields/bits and ownership passed");
    return 0;
}
