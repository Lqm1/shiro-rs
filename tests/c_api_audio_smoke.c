/* Actual C complete audio fields, original wave outputs and uniform callbacks. */
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

static ShiroRsArrayF32 *array(const float *values, size_t count) {
    ShiroRsArrayF32 *owner = NULL; assert(shiro_rs_array_f32_create(values, count, &owner) == 0); return owner;
}
static ShiroRsBytes *fixture(const char *name) {
    char path[160]; snprintf(path, sizeof(path), "tests/fixtures/%s", name);
    FILE *file = fopen(path, "rb"); assert(file && fseek(file, 0, SEEK_END) == 0);
    long count = ftell(file); assert(count >= 0 && fseek(file, 0, SEEK_SET) == 0);
    uint8_t *data = malloc((size_t)count + 1); assert(data);
    assert(fread(data, 1, (size_t)count, file) == (size_t)count && fclose(file) == 0);
    ShiroRsBytes *owner = NULL; assert(shiro_rs_bytes_create(data, (size_t)count, &owner) == 0); free(data); return owner;
}
static uint8_t *copied_bytes(ShiroRsBytes *owner, size_t *count) {
    assert(shiro_rs_bytes_length(owner, count) == 0); uint8_t *values = malloc(*count + 1); assert(values);
    assert(shiro_rs_bytes_copy(owner, 0, values, *count) == 0); return values;
}
static float *copied(ShiroRsArrayF32 *owner, size_t *count) {
    assert(shiro_rs_array_f32_length(owner, count) == 0); float *values = malloc((*count + 1) * 4); assert(values);
    assert(shiro_rs_array_f32_copy(owner, 0, values, *count) == 0); return values;
}
static uint32_t bits(const uint8_t *b) { return (uint32_t)b[0] | (uint32_t)b[1] << 8 | (uint32_t)b[2] << 16 | (uint32_t)b[3] << 24; }
static float scalar(const uint8_t *b) { uint32_t word = bits(b); float value; memcpy(&value, &word, 4); return value; }
static float *snapshot(ShiroRsAudio *owner, uint32_t *rate, size_t *count) {
    ShiroRsAudio *clone = NULL; ShiroRsArrayF32 *values = NULL;
    assert(shiro_rs_audio_clone(owner, &clone) == 0 && shiro_rs_audio_release(&owner) == 0);
    assert(shiro_rs_audio_sample_rate(clone, rate) == 0 && shiro_rs_audio_get_samples(clone, &values) == 0);
    assert(shiro_rs_audio_release(&clone) == 0); float *output = copied(values, count);
    assert(shiro_rs_array_f32_release(&values) == 0); return output;
}
static uint32_t sequence(void *context, float *output) { return shiro_rs_dither_next_uniform(context, output); }
struct draws {size_t position; int fail_at;};
static uint32_t controlled(void *context, float *output) {
    struct draws *state = context; size_t position = state->position++;
    if ((int)position == state->fail_at || position >= 3) return 7;
    const float values[] = {0, 0.5f, 1}; *output = values[position]; return 0;
}
int main(void) {
    ShiroRsAudioOptions options; assert(shiro_rs_audio_options_default(&options) == 0);
    assert(options.normalize == 0 && options.dither_level == 0 && options.has_output_sample_rate == 0 && options.output_sample_rate == 0 && options.boundary == 0 && options.kernel == 0);
    ShiroRsBytes *wire = fixture("c-audio-input.wav");
    const char *names[] = {"plain", "normalized", "up", "down", "normalized-down"};
    const uint32_t rates[] = {0, 0, 32000, 8000, 8000}, normalize[] = {0, 1, 0, 0, 1};
    for (size_t i = 0; i < 5; ++i) {
        assert(shiro_rs_audio_options_default(&options) == 0);
        options.normalize = normalize[i]; options.boundary = options.kernel = 1;
        options.has_output_sample_rate = rates[i] != 0; options.output_sample_rate = rates[i];
        ShiroRsAudio *output = NULL;
        assert(shiro_rs_audio_prepare_wave_bytes(wire, 1024, &options, NULL, NULL, &output) == 0);
        uint32_t rate = 0; size_t count = 0; float *actual = snapshot(output, &rate, &count);
        assert(rate == (rates[i] ? rates[i] : 16000));
        char name[100]; snprintf(name, sizeof(name), "c-audio-input.%s.raw", names[i]);
        ShiroRsBytes *expected_owner = fixture(name); size_t length = 0; uint8_t *expected = copied_bytes(expected_owner, &length);
        assert(count * 4 == length);
        for (size_t j = 0; j < count; ++j) { double e = scalar(expected + 4 * j); assert(fabs((double)actual[j] - e) / fmax(fabs(e), 1) < 2e-7); }
        free(actual); free(expected); assert(shiro_rs_bytes_release(&expected_owner) == 0);
    }
    for (size_t is_linux = 0; is_linux < 2; ++is_linux) {
        ShiroRsBytes *expected_owner = fixture(is_linux ? "c-dither-linux.bin" : "c-dither-windows.bin"); size_t length = 0;
        uint8_t *expected = copied_bytes(expected_owner, &length); assert(length == 12 + 64 * 12);
        ShiroRsDitherSequence *rng = NULL;
        assert((is_linux ? shiro_rs_dither_linux_gnu(&rng) : shiro_rs_dither_windows(&rng)) == 0);
        assert(shiro_rs_dither_next_uniform(rng, NULL) == 1);
        for (size_t j = 0; j < 64; ++j) {
            float draw; uint32_t actual_bits;
            assert(shiro_rs_dither_next_uniform(rng, &draw) == 0); memcpy(&actual_bits, &draw, 4);
            assert(actual_bits == bits(expected + 12 + j * 8 + 4));
        }
        assert(shiro_rs_dither_release(&rng) == 0);
        assert((is_linux ? shiro_rs_dither_linux_gnu(&rng) : shiro_rs_dither_windows(&rng)) == 0);
        const float zero[64] = {0}; ShiroRsArrayF32 *signal = array(zero, 64);
        ShiroRsWaveInfo header = {8000, 32, 1, 1}; assert(shiro_rs_audio_options_default(&options) == 0); options.dither_level = 1;
        ShiroRsAudio *output = NULL;
        assert(shiro_rs_audio_prepare(signal, &header, &options, sequence, rng, &output) == 0);
        uint32_t rate; size_t count; float *actual = snapshot(output, &rate, &count); assert(rate == 8000 && count == 64);
        for (size_t j = 0; j < count; ++j) {uint32_t word; memcpy(&word, &actual[j], 4); assert(word == bits(expected + 12 + 64 * 8 + j * 4));}
        free(actual); free(expected);
        assert(shiro_rs_dither_release(&rng) == 0 && shiro_rs_dither_release(&rng) == 0 && shiro_rs_array_f32_release(&signal) == 0 && shiro_rs_bytes_release(&expected_owner) == 0);
    }
    ShiroRsDitherSequence *rng = NULL; assert(shiro_rs_dither_linux_gnu(&rng) == 0);
    assert(shiro_rs_audio_options_default(&options) == 0); options.dither_level = 0.125f;
    ShiroRsAudio *output = NULL;
    assert(shiro_rs_audio_prepare_wave_bytes(wire, 1024, &options, sequence, rng, &output) == 0);
    uint32_t rate; size_t count; float *actual = snapshot(output, &rate, &count);
    ShiroRsBytes *expected_owner = fixture("c-audio-input.dither-linux.raw"); size_t length = 0; uint8_t *expected = copied_bytes(expected_owner, &length);
    assert(count * 4 == length);
    for (size_t j = 0; j < count; ++j) {uint32_t word; memcpy(&word, &actual[j], 4); assert(word == bits(expected + j * 4));}
    free(actual); free(expected); assert(shiro_rs_bytes_release(&expected_owner) == 0 && shiro_rs_dither_release(&rng) == 0);
    const float input[] = {-0.25f, 0.5f, 0}; ShiroRsArrayF32 *signal = array(input, 3);
    ShiroRsWaveInfo header = {8000, 0, UINT16_MAX, 0}; struct draws state = {0, -1};
    assert(shiro_rs_audio_options_default(&options) == 0); options.normalize = 1; options.dither_level = 0.125f;
    assert(shiro_rs_audio_prepare(signal, &header, &options, controlled, &state, &output) == 0);
    actual = snapshot(output, &rate, &count); assert(rate == 8000 && count == 3 && state.position == 3);
    assert(actual[0] == -0.625f && actual[1] == 1 && actual[2] == 0.125f); free(actual);
    const uint32_t words[] = {0x80000000, 1, 0x7fc12345, 0x7f800000}; float values[4]; memcpy(values, words, sizeof(values));
    ShiroRsArrayF32 *source = array(values, 4); ShiroRsAudio *owner = NULL;
    assert(shiro_rs_audio_create(UINT32_MAX, source, &owner) == 0 && shiro_rs_array_f32_release(&source) == 0);
    ShiroRsAudio *retained = owner; state.position = 0; state.fail_at = 1;
    assert(shiro_rs_audio_prepare(signal, &header, &options, controlled, &state, &retained) == 3 && retained == owner && state.position == 2);
    assert(shiro_rs_audio_prepare(signal, &header, &options, NULL, NULL, &retained) == 3 && retained == owner);
    for (size_t field = 0; field < 4; ++field) {
        assert(shiro_rs_audio_options_default(&options) == 0);
        switch(field) {case 0: options.normalize = 2; break; case 1: options.has_output_sample_rate = 2; break; case 2: options.boundary = 2; break; default: options.kernel = 2;}
        assert(shiro_rs_audio_prepare(signal, &header, &options, NULL, NULL, &retained) == 2 && retained == owner);
    }
    assert(shiro_rs_audio_options_default(&options) == 0); options.has_output_sample_rate = 1;
    assert(shiro_rs_audio_prepare(signal, &header, &options, NULL, NULL, &retained) == 3 && retained == owner);
    options.has_output_sample_rate = 0; options.output_sample_rate = UINT32_MAX; options.dither_level = -1; state.position = 0;
    assert(shiro_rs_audio_prepare(signal, &header, &options, controlled, &state, &output) == 0 && state.position == 0);
    actual = snapshot(output, &rate, &count); assert(rate == 8000 && count == 3 && memcmp(actual, input, sizeof(input)) == 0); free(actual);
    assert(shiro_rs_audio_prepare_wave_bytes(wire, 1, &options, NULL, NULL, &retained) == 3 && retained == owner);
    assert(shiro_rs_audio_prepare(signal, NULL, &options, NULL, NULL, &retained) == 1 && retained == owner);
    assert(shiro_rs_audio_options_default(NULL) == 1 && shiro_rs_audio_get_samples(owner, NULL) == 1 && shiro_rs_audio_clone(owner, NULL) == 1);
    assert(shiro_rs_audio_create(0, NULL, &retained) == 1 && retained == owner);
    assert(shiro_rs_dither_windows(NULL) == 1 && shiro_rs_dither_linux_gnu(NULL) == 1 && shiro_rs_dither_release(NULL) == 1 && shiro_rs_audio_release(NULL) == 1);
    actual = snapshot(owner, &rate, &count); assert(rate == UINT32_MAX && count == 4 && memcmp(actual, words, sizeof(words)) == 0); free(actual);
    assert(shiro_rs_array_f32_release(&signal) == 0 && shiro_rs_bytes_release(&wire) == 0);
    puts("SHIRO audio C: all12 exports, original5 wave cases/both64-draw sequences/full fields/callbacks and ownership passed"); return 0;
}
