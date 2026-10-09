/* Actual C training, complete reports and original C model compatibility. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <float.h>
#include <math.h>
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
    assert(fread(result, 1, (size_t)count, file) == (size_t)count && fclose(file) == 0);
    result[count] = 0; *size = (size_t)count; return result;
}
static ShiroRsBytes *owned(const void *source, size_t size) {
    ShiroRsBytes *output = NULL; assert(shiro_rs_bytes_create(source, size, &output) == 0); return output;
}
static void model_equal(ShiroRsModel *model, const char *path) {
    size_t size, count = 0; char *expected = load(path, &size);
    ShiroRsBytes *wire = NULL; assert(shiro_rs_model_write_bytes(model, 0, &wire) == 0);
    assert(shiro_rs_bytes_length(wire, &count) == 0 && count == size);
    unsigned char *actual = malloc(size); assert(actual);
    assert(shiro_rs_bytes_copy(wire, 0, actual, size) == 0 && memcmp(actual, expected, size) == 0);
    free(actual); free(expected); assert(shiro_rs_bytes_release(&wire) == 0);
}
typedef struct CsvChannel { unsigned char *bytes; size_t length, calls, flushes, chunk, fail; } CsvChannel;
static uint32_t csv_write(void *context, const uint8_t *bytes, size_t capacity, size_t *count) {
    CsvChannel *value = context;
    if (++value->calls == 1) return SHIRO_RS_IO_INTERRUPTED;
    if (value->length >= value->fail) return SHIRO_RS_IO_ERROR;
    size_t n = capacity < value->chunk ? capacity : value->chunk;
    if (n) { unsigned char *next = realloc(value->bytes, value->length + n); assert(next); value->bytes = next; memcpy(next + value->length, bytes, n); }
    value->length += n; *count = n; return SHIRO_RS_IO_SUCCESS;
}
static uint32_t csv_flush(void *context) { ++((CsvChannel *)context)->flushes; return 0; }
static uint32_t csv_excessive(void *context, const uint8_t *bytes, size_t capacity, size_t *count) {
    (void)context; (void)bytes; *count = capacity + 1; return 0;
}
static void likelihood_csv_equal(ShiroRsTrainingResult *result, const char *expected) {
    ShiroRsBytes *bytes = NULL; size_t count = 0;
    assert(shiro_rs_training_result_likelihood_csv_bytes(result, &bytes) == 0);
    ShiroRsBytes *retained = bytes;
    assert(shiro_rs_training_result_likelihood_csv_bytes(NULL, &retained) == 1 && retained == bytes);
    assert(shiro_rs_bytes_length(bytes, &count) == 0 && count > 0);
    char *text = malloc(count + 1); assert(text);
    assert(shiro_rs_bytes_copy(bytes, 0, (uint8_t *)text, count) == 0); text[count] = 0;
    const size_t chunks[] = {1, 13, 16384};
    for (size_t i = 0; i < 3; ++i) {
        CsvChannel channel = {0}; channel.chunk = chunks[i]; channel.fail = SIZE_MAX;
        ShiroRsWriteStream stream = {&channel, csv_write, csv_flush};
        assert(shiro_rs_training_result_write_likelihood_csv_stream(result, &stream) == 0);
        assert(channel.length == count && memcmp(channel.bytes, text, count) == 0 && channel.flushes == 0); free(channel.bytes);
    }
    CsvChannel channel = {0}; channel.chunk = 1; channel.fail = 7;
    ShiroRsWriteStream stream = {&channel, csv_write, csv_flush};
    assert(shiro_rs_training_result_write_likelihood_csv_stream(result, &stream) == 3);
    assert(channel.length == 7 && memcmp(channel.bytes, text, 7) == 0 && channel.flushes == 0); free(channel.bytes);
    memset(&channel, 0, sizeof(channel)); channel.fail = SIZE_MAX;
    assert(shiro_rs_training_result_write_likelihood_csv_stream(result, &stream) == 3 && channel.length == 0 && channel.flushes == 0);
    assert(shiro_rs_training_result_write_likelihood_csv_stream(NULL, &stream) == 1 && shiro_rs_training_result_write_likelihood_csv_stream(result, NULL) == 1);
    stream.write = NULL; assert(shiro_rs_training_result_write_likelihood_csv_stream(result, &stream) == 3);
    stream.write = csv_excessive; assert(shiro_rs_training_result_write_likelihood_csv_stream(result, &stream) == 3);
    assert(text[count - 1] == '\n'); const char *actual = text;
    while (*expected) {
        if (*expected == '\r') { ++expected; continue; }
        if (*expected == ',' || *expected == '\n') { assert(*actual == *expected); ++actual; ++expected; continue; }
        char *actual_end, *expected_end;
        float a = strtof(actual, &actual_end), e = strtof(expected, &expected_end);
        assert(actual_end != actual && expected_end != expected && fabsf(a - e) <= 1e-5f);
        actual = actual_end; expected = expected_end;
    }
    assert(*actual == 0); free(text); assert(shiro_rs_bytes_release(&bytes) == 0);
}
static char *states_array(const char *document) {
    const char *start = strstr(document, "\"states\""); assert(start); start = strchr(start, '['); assert(start);
    int depth = 0, quoted = 0, escaped = 0; const char *end = start;
    for (; *end; ++end) {
        if (quoted) { if (escaped) escaped = 0; else if (*end == '\\') escaped = 1; else if (*end == '"') quoted = 0; }
        else if (*end == '"') quoted = 1;
        else if (*end == '[') ++depth;
        else if (*end == ']' && --depth == 0) { ++end; break; }
    }
    assert(depth == 0 && !quoted); size_t size = (size_t)(end - start);
    char *output = malloc(size + 1); assert(output); memcpy(output, start, size); output[size] = 0; return output;
}
typedef struct Events { size_t count; ShiroRsIterationReport *reports[8]; } Events;
static void progress(void *context, const ShiroRsIterationReport *report) {
    Events *events = context; assert(events && events->count < 8);
    assert(shiro_rs_iteration_report_clone(report, &events->reports[events->count]) == 0); ++events->count;
}
static void reports_equal(const ShiroRsIterationReport *a, const ShiroRsIterationReport *b) {
    ShiroRsIterationInfo x, y;
    assert(shiro_rs_iteration_report_info(a, &x) == 0 && shiro_rs_iteration_report_info(b, &y) == 0);
    assert(x.iteration == y.iteration && x.temperature == y.temperature && x.mean_log_likelihood == y.mean_log_likelihood);
    size_t count = 0, other = 0;
    assert(shiro_rs_iteration_report_file_count(a, &count) == 0 && shiro_rs_iteration_report_file_count(b, &other) == 0 && count == other);
    for (size_t i = 0; i < count; ++i) {
        ShiroRsArrayF32 *left = NULL, *right = NULL; size_t n = 0, m = 0;
        assert(shiro_rs_iteration_report_get_file(a, i, &left) == 0 && shiro_rs_iteration_report_get_file(b, i, &right) == 0);
        assert(shiro_rs_array_f32_length(left, &n) == 0 && shiro_rs_array_f32_length(right, &m) == 0 && n == m);
        float *l = malloc(n * sizeof(float)), *r = malloc(n * sizeof(float)); assert(l && r);
        assert(shiro_rs_array_f32_copy(left, 0, l, n) == 0 && shiro_rs_array_f32_copy(right, 0, r, n) == 0);
        assert(memcmp(l, r, n * sizeof(float)) == 0); free(l); free(r);
        assert(shiro_rs_array_f32_release(&left) == 0 && shiro_rs_array_f32_release(&right) == 0);
    }
    ShiroRsArrayF32 *invalid = NULL; assert(shiro_rs_iteration_report_get_file(a, count, &invalid) == 2 && !invalid);
}
typedef struct Case { const char *model, *likelihood; size_t iterations; uint32_t mode, anneal, mean, isolated; } Case;
static void arbitrary_fields(ShiroRsModel *model) {
    const uint32_t bits[] = {0x80000000u, 0x7f800000u, 0xff800000u, 0x7fc01234u, 1u};
    float values[5]; memcpy(values, bits, sizeof(values));
    ShiroRsArrayF32 *row = NULL, *empty = NULL;
    assert(shiro_rs_array_f32_create(values, 5, &row) == 0);
    assert(shiro_rs_array_f32_create(NULL, 0, &empty) == 0);
    const ShiroRsArrayF32 *rows[] = {row, empty, row};
    ShiroRsIterationInfo info = {SIZE_MAX, -0.0f, INFINITY}, replacement = {7, 2.0f, 3.0f};
    ShiroRsIterationReport *report = NULL;
    assert(shiro_rs_iteration_report_create(&info, rows, 3, &report) == 0);
    const ShiroRsIterationReport *reports[] = {report, report}, *invalid[] = {report, NULL};
    ShiroRsTrainingResult *result = NULL, *clone = NULL;
    assert(shiro_rs_training_result_create(model, reports, 2, &result) == 0);
    assert(shiro_rs_training_result_clone(result, &clone) == 0);
    ShiroRsTrainingResult *retained = result;
    assert(shiro_rs_training_result_create(model, invalid, 2, &retained) == 1 && retained == result);
    assert(shiro_rs_training_result_create(model, NULL, 0, NULL) == 1);
    assert(shiro_rs_training_result_replace(result, model, invalid, 2) == 1);
    assert(shiro_rs_training_result_replace(result, model, NULL, SIZE_MAX) == 2);
    const ShiroRsArrayF32 *invalid_rows[] = {row, NULL};
    assert(shiro_rs_iteration_report_replace(report, &replacement, invalid_rows, 2) == 1);
    ShiroRsIterationInfo observed;
    assert(shiro_rs_iteration_report_info(report, &observed) == 0 && observed.iteration == SIZE_MAX);
    assert(shiro_rs_iteration_report_replace(report, &replacement, NULL, SIZE_MAX) == 2);
    assert(shiro_rs_iteration_report_replace(report, &replacement, NULL, 0) == 0);
    assert(shiro_rs_training_result_replace(result, model, reports, 1) == 0);
    ShiroRsIterationReport *snapshot = NULL;
    assert(shiro_rs_training_result_get_report(result, 0, &snapshot) == 0);
    assert(shiro_rs_iteration_report_info(snapshot, &observed) == 0);
    assert(observed.iteration == 7 && observed.temperature == 2.0f && observed.mean_log_likelihood == 3.0f);
    size_t count = 99;
    assert(shiro_rs_iteration_report_file_count(snapshot, &count) == 0 && count == 0);
    assert(shiro_rs_iteration_report_release(&snapshot) == 0);
    /* Replace the model as well, and release its input before reading the result. */
    size_t size; char *source = load("tests/fixtures/rest-c-hsmm-one.hsmm", &size);
    ShiroRsBytes *bytes = owned(source, size); free(source);
    ShiroRsModel *replacement_model = NULL;
    assert(shiro_rs_model_read_bytes(bytes, 16 * 1024 * 1024, &replacement_model) == 0);
    assert(shiro_rs_bytes_release(&bytes) == 0);
    assert(shiro_rs_training_result_replace(result, replacement_model, NULL, 0) == 0);
    assert(shiro_rs_model_release(&replacement_model) == 0);
    assert(shiro_rs_training_result_length(result, &count) == 0 && count == 0);
    ShiroRsModel *saved = NULL;
    assert(shiro_rs_training_result_get_model(result, &saved) == 0);
    model_equal(saved, "tests/fixtures/rest-c-hsmm-one.hsmm");
    assert(shiro_rs_model_release(&saved) == 0 && shiro_rs_training_result_release(&result) == 0);
    assert(shiro_rs_iteration_report_release(&report) == 0);
    assert(shiro_rs_array_f32_release(&row) == 0 && shiro_rs_array_f32_release(&empty) == 0);
    assert(shiro_rs_training_result_length(clone, &count) == 0 && count == 2);
    for (size_t index = 0; index < 2; ++index) {
        assert(shiro_rs_training_result_get_report(clone, index, &snapshot) == 0);
        assert(shiro_rs_iteration_report_info(snapshot, &observed) == 0);
        assert(observed.iteration == SIZE_MAX && signbit(observed.temperature) && observed.temperature == 0.0f && observed.mean_log_likelihood == INFINITY);
        assert(shiro_rs_iteration_report_file_count(snapshot, &count) == 0 && count == 3);
        for (size_t file = 0; file < 3; ++file) {
            ShiroRsArrayF32 *copied = NULL;
            assert(shiro_rs_iteration_report_get_file(snapshot, file, &copied) == 0);
            assert(shiro_rs_array_f32_length(copied, &count) == 0 && count == (file == 1 ? 0u : 5u));
            float actual[5];
            assert(shiro_rs_array_f32_copy(copied, 0, actual, count) == 0);
            if (count) assert(memcmp(actual, bits, sizeof(bits)) == 0);
            assert(shiro_rs_array_f32_release(&copied) == 0);
        }
        assert(shiro_rs_iteration_report_release(&snapshot) == 0);
    }
    assert(shiro_rs_training_result_get_model(clone, &saved) == 0);
    model_equal(saved, "tests/fixtures/init-c-aligned.hsmm");
    assert(shiro_rs_model_release(&saved) == 0 && shiro_rs_training_result_release(&clone) == 0);
    assert(shiro_rs_training_result_create(NULL, NULL, 0, &result) == 1);
    assert(shiro_rs_training_result_replace(NULL, model, NULL, 0) == 1);
    assert(shiro_rs_iteration_report_replace(NULL, &info, NULL, 0) == 1);
}
int main(void) {
    size_t size; char *source = load("tests/fixtures/init-c-aligned.hsmm", &size);
    ShiroRsBytes *bytes = owned(source, size); free(source); ShiroRsModel *model = NULL;
    assert(shiro_rs_model_read_bytes(bytes, 16 * 1024 * 1024, &model) == 0 && shiro_rs_bytes_release(&bytes) == 0);
    arbitrary_fields(model);
    source = load("tests/fixtures/init-input.bin", &size);
    char path[128]; snprintf(path, sizeof(path), ".shiro-c-api-training-%s-%ld.f", TASK_OS, (long)TASK_PID());
    FILE *file = fopen(path, "wb"); assert(file && fwrite(source, 1, size, file) == size && fclose(file) == 0); free(source);
    source = load("tests/fixtures/align-c-isolated.json", &size); char *array = states_array(source); free(source);
    size_t capacity = strlen(array) + strlen(path) + 128; char *document = malloc(capacity); assert(document);
    snprintf(document, capacity, "{\"file_list\":[{\"filename\":\"%s\",\"states\":%s}]}", path, array);
    bytes = owned(document, strlen(document));
    const Case cases[] = {
        {"hsmm-one", "hsmm-one", 1, 0, 0, 0, 0}, {"hsmm-two", "hsmm-two", 2, 0, 0, 0, 0},
        {"daem", "daem", 3, 0, 1, 0, 0}, {"hmm", "hmm", 2, 1, 0, 0, 0},
        {"hsmm-one", "mean", 1, 0, 0, 1, 0}, {"isolated", "isolated", 2, 0, 0, 0, 1},
        {"isolated-hmm", "isolated-hmm", 2, 1, 0, 0, 1}, {"isolated-daem", "isolated-daem", 3, 0, 1, 0, 1},
        {"isolated-mean", "isolated-mean", 2, 0, 0, 1, 1},
    };
    ShiroRsTrainingOptions config; assert(shiro_rs_training_options_default(&config) == 0);
    assert(config.iterations == 1 && config.duration_mode == 0 && config.hsmm_temperature == 1 && config.duration_weight == 1);
    assert(config.state_radius == 5 && config.duration_extra == 30 && config.duration_extra_factor == 1);
    assert(config.geometric_temperature == 1 && config.pruning_slope == 0.3f && config.termination_threshold == 1);
    assert(config.deterministic_annealing == 0 && config.mean_frame_likelihood == 0 && config.workers == 1);
    for (size_t k = 0; k < sizeof(cases)/sizeof(cases[0]); ++k) {
        Case test = cases[k]; ShiroRsTrainingFiles *files = NULL, *clone = NULL;
        assert(shiro_rs_training_files_read_document(model, bytes, 12, test.isolated, &files) == 0);
        assert(shiro_rs_training_files_clone(files, &clone) == 0 && shiro_rs_training_files_release(&files) == 0);
        size_t count = 0; assert(shiro_rs_training_files_length(clone, &count) == 0 && count == 1);
        ShiroRsDataset *snapshot = NULL; assert(shiro_rs_training_files_get_dataset(clone, 0, &snapshot) == 0);
        assert(shiro_rs_dataset_length(snapshot, &count) == 0 && count == (test.isolated ? 2 : 1));
        const ShiroRsDataset *inputs[] = {snapshot}; assert(shiro_rs_training_files_create(inputs, 1, &files) == 0);
        assert(shiro_rs_dataset_release(&snapshot) == 0 && shiro_rs_training_files_release(&clone) == 0);
        assert(shiro_rs_training_options_default(&config) == 0);
        config.iterations = test.iterations; config.duration_mode = test.mode; config.deterministic_annealing = test.anneal;
        config.mean_frame_likelihood = test.mean; config.termination_threshold = 0; config.pruning_slope = 0.8f;
        Events events = {0}; ShiroRsTrainingResult *trained = NULL, *copy = NULL;
        assert(shiro_rs_train_with_progress(model, files, &config, progress, &events, &trained) == 0);
        assert(shiro_rs_training_result_length(trained, &count) == 0 && count == test.iterations && events.count == count);
        assert(shiro_rs_training_result_clone(trained, &copy) == 0 && shiro_rs_training_result_release(&trained) == 0);
        ShiroRsModel *result = NULL; assert(shiro_rs_training_result_get_model(copy, &result) == 0);
        char fixture[160]; snprintf(fixture, sizeof(fixture), "tests/fixtures/rest-c-%s.hsmm", test.model); model_equal(result, fixture);
        snprintf(fixture, sizeof(fixture), "tests/fixtures/rest-c-%s.likelihood", test.likelihood); source = load(fixture, &size); likelihood_csv_equal(copy, source);
        char *cursor = source;
        for (size_t i = 0; i < count; ++i) {
            ShiroRsIterationReport *report = NULL; assert(shiro_rs_training_result_get_report(copy, i, &report) == 0);
            reports_equal(report, events.reports[i]); ShiroRsIterationInfo info;
            assert(shiro_rs_iteration_report_info(report, &info) == 0 && info.iteration == i && isfinite(info.mean_log_likelihood));
            float temperature = test.anneal ? (float)sqrt((double)((float)(i+1)/(float)test.iterations)) : 1.0f;
            assert(info.temperature == temperature);
            ShiroRsArrayF32 *row = NULL; size_t n = 0;
            assert(shiro_rs_iteration_report_get_file(report, 0, &row) == 0 && shiro_rs_array_f32_length(row, &n) == 0);
            float values[2]; assert(n == (test.isolated ? 2 : 1) && shiro_rs_array_f32_copy(row, 0, values, n) == 0);
            volatile float mean = 0;
            for (size_t j = 0; j < n; ++j) { char *end; float expected = strtof(cursor, &end); assert(end != cursor && fabsf(values[j] - expected) <= 1e-5f); cursor = end; if (j+1<n) {assert(*cursor == ','); ++cursor;} volatile float group_mean = values[j]/(float)n; mean = mean + group_mean; }
            while (*cursor == '\r' || *cursor == '\n') ++cursor;
            /* Force native binary32 rounding even with i686 x87 registers. */
            volatile float normalized_mean = mean / temperature;
            assert(info.mean_log_likelihood == normalized_mean);
            assert(shiro_rs_array_f32_release(&row) == 0 && shiro_rs_iteration_report_release(&report) == 0);
        }
        assert(*cursor == 0); free(source); assert(shiro_rs_training_result_release(&copy) == 0);
        for (size_t i = 0; i < events.count; ++i) { ShiroRsIterationInfo info; assert(shiro_rs_iteration_report_info(events.reports[i], &info) == 0); assert(shiro_rs_iteration_report_release(&events.reports[i]) == 0); }
        assert(shiro_rs_model_release(&result) == 0 && shiro_rs_training_files_release(&files) == 0);
    }
    ShiroRsTrainingFiles *files = NULL; assert(shiro_rs_training_files_read_document(model, bytes, 12, 1, &files) == 0);
    for (uint32_t mode = 0; mode < 2; ++mode) {
        assert(shiro_rs_training_options_default(&config) == 0); config.iterations = 5; config.duration_mode = mode; config.termination_threshold = FLT_MAX;
        Events events = {0}; ShiroRsTrainingResult *trained = NULL; size_t count = 0;
        assert(shiro_rs_train_with_progress(model, files, &config, progress, &events, &trained) == 0);
        assert(shiro_rs_training_result_length(trained, &count) == 0 && count == 2 && events.count == 2);
        assert(shiro_rs_training_result_release(&trained) == 0);
        for (size_t i = 0; i < events.count; ++i) assert(shiro_rs_iteration_report_release(&events.reports[i]) == 0);
    }
    ShiroRsTrainingFiles *retained_files = files;
    assert(shiro_rs_training_files_read_document(model, bytes, 11, 1, &retained_files) == 3 && retained_files == files);
    assert(shiro_rs_training_files_read_document(model, bytes, 12, 2, &retained_files) == 2 && retained_files == files);
    assert(shiro_rs_training_files_create(NULL, SIZE_MAX, &retained_files) == 2 && retained_files == files);
    assert(shiro_rs_training_files_release(&files) == 0 && shiro_rs_bytes_release(&bytes) == 0);
    assert(shiro_rs_training_files_create(NULL, 0, &files) == 0 && shiro_rs_training_options_default(&config) == 0);
    config.iterations = 0; ShiroRsTrainingResult *trained = NULL;
    assert(shiro_rs_train(model, files, &config, &trained) == 0);
    CsvChannel empty_csv = {0}; empty_csv.chunk = 1; empty_csv.fail = SIZE_MAX;
    ShiroRsWriteStream empty_stream = {&empty_csv, csv_write, csv_flush};
    assert(shiro_rs_training_result_write_likelihood_csv_stream(trained, &empty_stream) == 0 && empty_csv.calls == 0 && empty_csv.flushes == 0 && empty_csv.length == 0);
    ShiroRsTrainingResult *retained = trained; config.duration_mode = 2;
    assert(shiro_rs_train(model, files, &config, &retained) == 2 && retained == trained);
    config.duration_mode = 0; config.iterations = 1;
    assert(shiro_rs_train(model, files, &config, &retained) == 3 && retained == trained);
    assert(shiro_rs_train(model, files, NULL, &retained) == 1 && retained == trained);
    ShiroRsIterationReport *invalid = NULL; assert(shiro_rs_training_result_get_report(trained, 0, &invalid) == 2 && !invalid);
    assert(shiro_rs_training_options_default(NULL) == 1 && shiro_rs_training_files_release(NULL) == 1);
    assert(shiro_rs_training_result_release(NULL) == 1 && shiro_rs_iteration_report_release(NULL) == 1);
    assert(shiro_rs_training_result_release(&trained) == 0 && shiro_rs_training_result_release(&trained) == 0);
    assert(shiro_rs_training_files_release(&files) == 0);
    model_equal(model, "tests/fixtures/init-c-aligned.hsmm"); assert(shiro_rs_model_release(&model) == 0);
    free(array); free(document); assert(remove(path) == 0);
    puts("SHIRO training C: all25 exports, nine original C models/likelihoods/CSV, direct partial CSV streams, arbitrary result/report fields, atomic replacement, independent ownership, callbacks and failures passed"); return 0;
}
