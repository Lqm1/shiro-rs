/* Actual C complete alignment ABI, original C cases and native host workflow. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
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
    assert(fread(result, 1, (size_t)count, file) == (size_t)count); assert(fclose(file) == 0);
    result[count] = 0; *size = (size_t)count; return result;
}
static ShiroRsBytes *owned(const void *data, size_t size) {
    ShiroRsBytes *result = NULL; assert(shiro_rs_bytes_create(data, size, &result) == 0); return result;
}
static char *copied(ShiroRsBytes *owner) {
    size_t size = 0; assert(shiro_rs_bytes_length(owner, &size) == 0);
    char *result = malloc(size + 1); assert(result);
    assert(shiro_rs_bytes_copy(owner, 0, (unsigned char *)result, size) == 0); result[size] = 0; return result;
}
/* Lexically extract the complete states array; quoted brackets/escapes are data. */
static char *states_array(const char *document) {
    const char *start = strstr(document, "\"states\""); assert(start);
    start = strchr(start, '['); assert(start);
    int depth = 0, string = 0, escape = 0; const char *end = start;
    for (; *end; ++end) {
        if (string) { if (escape) escape = 0; else if (*end == '\\') escape = 1; else if (*end == '"') string = 0; }
        else if (*end == '"') string = 1;
        else if (*end == '[') ++depth;
        else if (*end == ']' && --depth == 0) { ++end; break; }
    }
    assert(depth == 0 && !string); size_t size = (size_t)(end - start);
    char *result = malloc(size + 1); assert(result); memcpy(result, start, size); result[size] = 0; return result;
}
static ShiroRsStates *read_states(const char *json) {
    ShiroRsBytes *bytes = owned(json, strlen(json)); ShiroRsStates *result = NULL;
    assert(shiro_rs_states_read_json(bytes, &result) == 0); assert(shiro_rs_bytes_release(&bytes) == 0); return result;
}
static char *states_json(ShiroRsStates *states) {
    ShiroRsBytes *bytes = NULL; assert(shiro_rs_states_write_json(states, &bytes) == 0);
    char *result = copied(bytes); assert(shiro_rs_bytes_release(&bytes) == 0); return result;
}
int main(void) {
    ShiroRsAlignmentOptions defaults;
    assert(shiro_rs_alignment_options_default(&defaults) == 0);
    assert(defaults.duration_mode == 0 && defaults.isolated == 0 && defaults.hsmm_temperature == 1);
    assert(defaults.duration_weight == 1 && defaults.state_radius == 5 && defaults.duration_extra == 30);
    assert(defaults.duration_extra_factor == 1 && defaults.geometric_temperature == 1 && defaults.pruning_slope == 0.3f);
    assert(shiro_rs_alignment_options_default(NULL) == 1);
    size_t size; char *data = load("tests/fixtures/init-c-aligned.hsmm", &size);
    ShiroRsBytes *bytes = owned(data, size); free(data); ShiroRsModel *model = NULL;
    assert(shiro_rs_model_read_bytes(bytes, 16*1024*1024, &model) == 0); assert(shiro_rs_bytes_release(&bytes) == 0);
    data = load("tests/fixtures/init-input.bin", &size);
    char feature_path[128]; assert(snprintf(feature_path, sizeof(feature_path), ".shiro-c-api-align-%s-%ld.f", TASK_OS, (long)TASK_PID()) > 0);
    FILE *feature = fopen(feature_path, "wb"); assert(feature && fwrite(data, 1, size, feature) == size && fclose(feature) == 0);
    bytes = owned(data, size); free(data); ShiroRsObservation *observation = NULL;
    assert(shiro_rs_observation_from_model_rawfloat(bytes, model, 12, &observation) == 0); assert(shiro_rs_bytes_release(&bytes) == 0);
    const char *references[] = {
        "align-c-embedded-hsmm.json", "align-c-embedded-hmm.json", "align-c-embedded-hsmm-pruned.json", "align-c-embedded-hmm-pruned.json",
        "align-c-isolated-hsmm.json", "align-c-isolated-hmm.json", "align-c-isolated-hsmm-pruned.json", "align-c-isolated-hmm-pruned.json", "align-c-four-hmm.json"
    };
    for (size_t index = 0; index < 9; ++index) {
        const char *source = index == 8 ? "align-c-four.json" : index >= 4 ? "align-c-isolated.json" : "align-c-embedded.json";
        char path[160]; snprintf(path, sizeof(path), "tests/fixtures/%s", source);
        data = load(path, &size); char *input = states_array(data); free(data);
        ShiroRsStates *states = read_states(input), *aligned = NULL;
        ShiroRsAlignmentOptions options = defaults;
        options.duration_mode = (uint32_t)(index % 2); options.isolated = index >= 4 && index != 8;
        if (index == 8) options.duration_mode = 1;
        else options.pruning_slope = 0.8f;
        if (index < 8 && index % 4 >= 2) { options.state_radius = 2; options.duration_extra = 8; options.pruning_slope = 0.5f; }
        assert(shiro_rs_align_states(model, observation, states, &options, &aligned) == 0);
        snprintf(path, sizeof(path), "tests/fixtures/%s", references[index]);
        data = load(path, &size); char *reference_array = states_array(data); free(data);
        ShiroRsStates *reference = read_states(reference_array); free(reference_array);
        char *expected = states_json(reference), *actual = states_json(aligned);
        assert(strcmp(actual, expected) == 0); free(actual);
        assert(shiro_rs_states_release(&aligned) == 0); assert(shiro_rs_states_release(&reference) == 0);
        const char *format = "{\"file_list\":[{\"filename\":\"%s\",\"states\":%s,\"file_metadata\":{\"keep\":true}}],\"document_metadata\":{\"keep\":[1,null]}}";
        size_t capacity = strlen(input) + strlen(expected) + 512;
        char *document = malloc(capacity), *expected_document = malloc(capacity); assert(document && expected_document);
        snprintf(document, capacity, format, feature_path, input); snprintf(expected_document, capacity, format, feature_path, expected);
        ShiroRsBytes *document_bytes = owned(document, strlen(document)), *output = NULL;
        assert(shiro_rs_align_document(model, document_bytes, &options, &output) == 0);
        actual = copied(output); assert(strcmp(actual, expected_document) == 0); free(actual);
        actual = copied(document_bytes); assert(strcmp(actual, document) == 0); free(actual);
        ShiroRsAlignmentOptions invalid = options; invalid.isolated = 2;
        ShiroRsStates *retained = states;
        assert(shiro_rs_align_states(model, observation, states, &invalid, &retained) == 2 && retained == states);
        ShiroRsBytes *retained_bytes = output;
        assert(shiro_rs_align_document(model, document_bytes, &invalid, &retained_bytes) == 2 && retained_bytes == output);
        assert(shiro_rs_align_states(model, observation, states, NULL, &retained) == 1 && retained == states);
        assert(shiro_rs_align_states(model, observation, states, &options, NULL) == 1);
        assert(shiro_rs_align_document(model, document_bytes, NULL, &retained_bytes) == 1);
        assert(shiro_rs_align_document(model, document_bytes, &options, NULL) == 1);
        assert(shiro_rs_bytes_release(&output) == 0); assert(shiro_rs_bytes_release(&document_bytes) == 0);
        assert(shiro_rs_states_release(&states) == 0);
        free(document); free(expected_document); free(expected); free(input);
    }
    assert(remove(feature_path) == 0);
    assert(shiro_rs_observation_release(&observation) == 0); assert(shiro_rs_model_release(&model) == 0);
    puts("SHIRO alignment C: all3 exports, all9 original C cases, complete descriptor defaults and host metadata passed");
    return 0;
}
