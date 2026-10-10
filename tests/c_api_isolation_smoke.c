/* Actual C standalone grouping, complete positions, frame slices and state data. */
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
static void equal_bytes(ShiroRsBytes *a, ShiroRsBytes *b) {
    size_t n = 0, m = 0;
    assert(shiro_rs_bytes_length(a, &n) == 0 && shiro_rs_bytes_length(b, &m) == 0 && n == m);
    void *left = malloc(n + 1), *right = malloc(n + 1); assert(left && right);
    assert(shiro_rs_bytes_copy(a, 0, left, n) == 0 && shiro_rs_bytes_copy(b, 0, right, n) == 0);
    assert(memcmp(left, right, n) == 0); free(left); free(right);
}
static ShiroRsStates *state_json(const char *json) {
    ShiroRsBytes *wire = owned(json, strlen(json)); ShiroRsStates *output = NULL;
    assert(shiro_rs_states_read_json(wire, &output) == 0 && shiro_rs_bytes_release(&wire) == 0);
    return output;
}
/* Original align-c-isolated.json state fields, with an extra crossing jump and metadata. */
static const char *source_json = "["
    "{\"out\":[0,0],\"time\":2,\"dur\":0,\"ext\":[\"a\",0,\"retained\"],\"jmp\":[{\"d\":1,\"p\":0.9},{\"d\":4,\"p\":0.1}],\"extra\":{\"nested\":[1,null,\"retained\"]}},"
    "{\"out\":[1,1],\"time\":4,\"dur\":1,\"ext\":[\"a\",1,\"retained\"]},"
    "{\"out\":[2,2],\"time\":6,\"dur\":2,\"ext\":[\"a\",2,\"retained\"]},"
    "{\"out\":[0,0],\"time\":8,\"dur\":0,\"ext\":[\"b\",0,\"retained\",{\"retained\":true}]},"
    "{\"out\":[1,1],\"time\":10,\"dur\":1,\"ext\":[\"b\",1,\"retained\"]},"
    "{\"out\":[2,2],\"time\":40,\"dur\":2,\"ext\":[\"b\",2,\"retained\"]}]";
static const char *local_json[] = {
    "[{\"out\":[0,0],\"time\":2,\"dur\":0,\"ext\":[\"a\",0,\"retained\"],\"jmp\":[{\"d\":1,\"p\":0.9}],\"extra\":{\"nested\":[1,null,\"retained\"]}},"
    "{\"out\":[1,1],\"time\":4,\"dur\":1,\"ext\":[\"a\",1,\"retained\"]},{\"out\":[2,2],\"time\":6,\"dur\":2,\"ext\":[\"a\",2,\"retained\"]}]",
    "[{\"out\":[0,0],\"time\":2,\"dur\":0,\"ext\":[\"b\",0,\"retained\",{\"retained\":true}]},"
    "{\"out\":[1,1],\"time\":4,\"dur\":1,\"ext\":[\"b\",1,\"retained\"]},{\"out\":[2,2],\"time\":34,\"dur\":2,\"ext\":[\"b\",2,\"retained\"]}]"
};
int main(void) {
    ShiroRsBytes *wire = fixture("init-c-aligned.hsmm"); ShiroRsModel *model = NULL;
    assert(shiro_rs_model_read_bytes(wire, 16 * 1024 * 1024, &model) == 0 && shiro_rs_bytes_release(&wire) == 0);
    ShiroRsBytes *raw = fixture("init-input.bin"); ShiroRsObservation *source = NULL;
    assert(shiro_rs_observation_from_model_rawfloat(raw, model, 12, &source) == 0);
    ShiroRsStates *source_states = state_json(source_json);
    ShiroRsIsolatedGroups *groups = NULL, *clone = NULL;
    assert(shiro_rs_isolated_groups(model, source, source_states, &groups) == 0);
    assert(shiro_rs_isolated_groups_clone(groups, &clone) == 0 && shiro_rs_isolated_groups_release(&groups) == 0);
    size_t count = 0; assert(shiro_rs_isolated_groups_length(clone, &count) == 0 && count == 2);
    ShiroRsObservation *samples[2] = {NULL, NULL}; ShiroRsStates *locals[2] = {NULL, NULL};
    for (size_t i = 0; i < 2; ++i) {
        _Alignas(8) ShiroRsIsolatedGroupInfo positions;
        assert(shiro_rs_isolated_groups_get_info(clone, i, &positions) == 0);
        assert(positions.first_state == i * 3 && positions.first_frame == i * 6);
        assert(shiro_rs_isolated_groups_get_observation(clone, i, &samples[i]) == 0);
        assert(shiro_rs_isolated_groups_get_states(clone, i, &locals[i]) == 0);
    }
    assert(shiro_rs_isolated_groups_release(&clone) == 0);
    size_t raw_count = 0; assert(shiro_rs_bytes_length(raw, &raw_count) == 0 && raw_count % 12 == 0);
    uint8_t *raw_values = malloc(raw_count); assert(raw_values);
    assert(shiro_rs_bytes_copy(raw, 0, raw_values, raw_count) == 0);
    for (size_t i = 0; i < 2; ++i) {
        ShiroRsBytes *slice = owned(raw_values + i * raw_count / 2, raw_count / 2), *expected = NULL;
        ShiroRsObservation *sample = NULL;
        assert(shiro_rs_observation_from_model_rawfloat(slice, model, 6, &sample) == 0);
        assert(shiro_rs_observation_write_bytes(sample, &expected) == 0 && shiro_rs_observation_write_bytes(samples[i], &wire) == 0);
        equal_bytes(wire, expected);
        assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected) == 0 && shiro_rs_bytes_release(&slice) == 0);
        assert(shiro_rs_observation_release(&sample) == 0 && shiro_rs_observation_release(&samples[i]) == 0);
        ShiroRsStates *states = state_json(local_json[i]);
        assert(shiro_rs_states_write_json(states, &expected) == 0 && shiro_rs_states_write_json(locals[i], &wire) == 0);
        equal_bytes(wire, expected);
        assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected) == 0);
        assert(shiro_rs_states_release(&states) == 0 && shiro_rs_states_release(&locals[i]) == 0);
    }
    free(raw_values); assert(shiro_rs_bytes_release(&raw) == 0);
    _Alignas(8) ShiroRsIsolatedGroupInput input[] = {{SIZE_MAX, SIZE_MAX - 1, source, source_states}, {SIZE_MAX, SIZE_MAX - 1, source, source_states}};
    assert(shiro_rs_isolated_groups_create(input, 2, &groups) == 0);
    _Alignas(8) ShiroRsIsolatedGroupInfo positions = {0, 0};
    assert(shiro_rs_isolated_groups_get_info(groups, 1, &positions) == 0 && positions.first_state == SIZE_MAX && positions.first_frame == SIZE_MAX - 1);
    assert(shiro_rs_isolated_groups_get_info(groups, 2, &positions) == 2 && positions.first_state == SIZE_MAX);
    ShiroRsIsolatedGroups *retained = groups; input[1].states = NULL;
    assert(shiro_rs_isolated_groups_create(input, 2, &retained) == 1 && retained == groups);
    assert(shiro_rs_isolated_groups_create(NULL, SIZE_MAX, &retained) == 2 && retained == groups);
    ShiroRsStates *empty = state_json("[]");
    assert(shiro_rs_isolated_groups(model, source, empty, &retained) == 3 && retained == groups);
    assert(shiro_rs_states_release(&empty) == 0);
    assert(shiro_rs_isolated_groups_get_observation(groups, 0, &samples[0]) == 0 && shiro_rs_isolated_groups_get_states(groups, 0, &locals[0]) == 0);
    ShiroRsBytes *expected_sample = NULL, *expected_states = NULL;
    assert(shiro_rs_observation_write_bytes(source, &expected_sample) == 0 && shiro_rs_states_write_json(source_states, &expected_states) == 0);
    assert(shiro_rs_isolated_groups_release(&groups) == 0 && shiro_rs_model_release(&model) == 0);
    assert(shiro_rs_observation_release(&source) == 0 && shiro_rs_states_release(&source_states) == 0);
    assert(shiro_rs_observation_write_bytes(samples[0], &wire) == 0); equal_bytes(wire, expected_sample);
    assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected_sample) == 0);
    assert(shiro_rs_states_write_json(locals[0], &wire) == 0); equal_bytes(wire, expected_states);
    assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&expected_states) == 0);
    assert(shiro_rs_observation_release(&samples[0]) == 0 && shiro_rs_states_release(&locals[0]) == 0);
    assert(shiro_rs_isolated_groups_create(NULL, 0, &groups) == 0 && shiro_rs_isolated_groups_length(groups, &count) == 0 && count == 0);
    assert(shiro_rs_isolated_groups_clone(groups, NULL) == 1 && shiro_rs_isolated_groups_release(NULL) == 1 && shiro_rs_isolated_groups_release(&groups) == 0);
    puts("SHIRO isolation C: all8 exports, complete positions/samples/states and ownership passed"); return 0;
}
