/* Independent C rawfloat/deinterleaving/legacy-wire and metadata oracle. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ShiroRsBytes *owned(const unsigned char *source, size_t count) {
    ShiroRsBytes *result = NULL;
    assert(shiro_rs_bytes_create(source, count, &result) == 0);
    return result;
}
static void equal(ShiroRsBytes *owner, const unsigned char *expected, size_t count) {
    size_t length = 99;
    assert(shiro_rs_bytes_length(owner, &length) == 0 && length == count);
    unsigned char *actual = malloc(count + 1);
    assert(actual && shiro_rs_bytes_copy(owner, 0, actual, count) == 0);
    assert(memcmp(actual, expected, count) == 0);
    free(actual);
}
static void scalar(unsigned char *wire, size_t *position, uint32_t bits) {
    wire[(*position)++] = 0xca;
    for (int shift = 24; shift >= 0; shift -= 8) wire[(*position)++] = (unsigned char)(bits >> shift);
}
static uint32_t bits(float value) {
    uint32_t result; memcpy(&result, &value, sizeof(result)); return result;
}
int main(void) {
    const uint32_t patterns[] = {0,0x80000000,1,0x80000001,0x3f800001,0x3f000000};
    unsigned char raw[24], expected[128]; size_t position = 0;
    for (size_t i = 0; i < 6; ++i) for (size_t j = 0; j < 4; ++j) raw[i*4+j] = (unsigned char)(patterns[i] >> (j*8));
    const unsigned char header[] = {0x93,2,0x92,2,1,0x92,0x94};
    memcpy(expected, header, sizeof(header)); position = sizeof(header);
    for (size_t i = 0; i < 6; ++i) if (i % 3 != 2) scalar(expected, &position, patterns[i]);
    expected[position++] = 0x92; scalar(expected, &position, patterns[2]); scalar(expected, &position, patterns[5]);
    const size_t dimensions[] = {2,1};
    ShiroRsBytes *source = owned(raw, sizeof(raw)), *output = NULL;
    ShiroRsObservation *observation = NULL, *clone = NULL;
    assert(shiro_rs_observation_read_rawfloat(source, dimensions, 2, 2, &observation) == 0);
    assert(shiro_rs_observation_clone(observation, &clone) == 0);
    assert(shiro_rs_observation_release(&observation) == 0 && !observation);
    assert(shiro_rs_observation_write_bytes(clone, &output) == 0);
    equal(output, expected, position);
    assert(shiro_rs_bytes_release(&output) == 0);
    ShiroRsObservation *retained = clone;
    assert(shiro_rs_observation_read_rawfloat(source, dimensions, 2, 1, &retained) == 3 && retained == clone);
    assert(shiro_rs_observation_read_rawfloat(source, NULL, 1, 2, &retained) == 1 && retained == clone);
    assert(shiro_rs_observation_read_rawfloat(source, NULL, SIZE_MAX, 2, &retained) == 2 && retained == clone);
    assert(shiro_rs_observation_clone(clone, NULL) == 1);
    assert(shiro_rs_observation_write_bytes(clone, NULL) == 1);
    assert(shiro_rs_observation_release(&clone) == 0);
    const char definition[] = "{\"ndurstate\":4,\"streamdef\":[{\"nstate\":3,\"ndim\":2},{\"nstate\":3,\"ndim\":1}]}";
    ShiroRsBytes *configuration = owned((const unsigned char *)definition, strlen(definition));
    ShiroRsModel *model = NULL;
    assert(shiro_rs_model_from_definition(configuration, &model) == 0);
    assert(shiro_rs_bytes_release(&configuration) == 0);
    assert(shiro_rs_observation_from_model_rawfloat(source, model, 2, &observation) == 0);
    assert(shiro_rs_bytes_release(&source) == 0);
    assert(shiro_rs_observation_write_bytes(observation, &output) == 0);
    equal(output, expected, position);
    assert(shiro_rs_bytes_release(&output) == 0);
    assert(shiro_rs_observation_release(&observation) == 0);
    const char states_json[] = "[{\"time\":1.75,\"dur\":0,\"out\":[0,0],\"jmp\":[{\"d\":0,\"p\":0.123456789,\"edge\":\"keep\"},{\"d\":1,\"p\":0.9}],\"ext\":[\"aa\",0,{\"nested\":[true,null,5]}],\"custom\":{\"value\":\"keep\"}},{\"time\":4.125,\"dur\":1,\"out\":[1,1],\"jmp\":[],\"ext\":[\"aa\",1,\"tail\"]}]";
    const char canonical[] = "[{\"time\":1.75,\"dur\":0,\"out\":[0,0],\"jmp\":[{\"d\":0,\"edge\":\"keep\",\"p\":0.123456789},{\"d\":1,\"p\":0.9}],\"ext\":[\"aa\",0,{\"nested\":[true,null,5]}],\"custom\":{\"value\":\"keep\"}},{\"time\":4.125,\"dur\":1,\"out\":[1,1],\"jmp\":[],\"ext\":[\"aa\",1,\"tail\"]}]";
    source = owned((const unsigned char *)states_json, strlen(states_json));
    ShiroRsStates *states = NULL, *state_clone = NULL;
    assert(shiro_rs_states_read_json(source, &states) == 0);
    assert(shiro_rs_bytes_release(&source) == 0);
    assert(shiro_rs_states_clone(states, &state_clone) == 0);
    assert(shiro_rs_states_release(&states) == 0 && !states);
    assert(shiro_rs_states_write_json(state_clone, &output) == 0);
    equal(output, (const unsigned char *)canonical, strlen(canonical));
    assert(shiro_rs_bytes_release(&output) == 0);
    const unsigned char seg_header[] = {0x93,0x92,1,4,0x92,0,1,0x92,0x92,0,1,0x92,0,1,0x92,0x92,0};
    memcpy(expected, seg_header, sizeof(seg_header)); position = sizeof(seg_header);
    scalar(expected, &position, bits((float)0.123456789)); expected[position++] = 1;
    scalar(expected, &position, bits((float)(1.0 - 0.123456789)));
    expected[position++] = 0x91; expected[position++] = 1; scalar(expected, &position, bits(1.0f));
    assert(shiro_rs_states_segmentation_bytes(state_clone, model, &output) == 0);
    equal(output, expected, position);
    assert(shiro_rs_bytes_release(&output) == 0);
    ShiroRsStates *state_retained = state_clone;
    source = owned((const unsigned char *)"[] []", 5);
    assert(shiro_rs_states_read_json(source, &state_retained) == 3 && state_retained == state_clone);
    assert(shiro_rs_states_clone(state_clone, NULL) == 1);
    assert(shiro_rs_states_write_json(state_clone, NULL) == 1);
    assert(shiro_rs_states_segmentation_bytes(state_clone, NULL, &output) == 1 && !output);
    assert(shiro_rs_bytes_release(&source) == 0);
    assert(shiro_rs_states_release(&state_clone) == 0);
    assert(shiro_rs_states_release(&state_clone) == 0);
    assert(shiro_rs_model_release(&model) == 0);
    assert(shiro_rs_states_release(NULL) == 1 && shiro_rs_observation_release(NULL) == 1);
    puts("SHIRO samples C: all10 exports, complete deinterleaving bits, metadata and independent legacy segmentation wire passed");
    return 0;
}
