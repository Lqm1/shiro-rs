/* Actual C complete batch owners, presets and native/SPTK/Lua workflows. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <direct.h>
#include <process.h>
#define get_process_id _getpid
#define current_directory _getcwd
#define create_directory(path) _mkdir(path)
#define remove_directory _rmdir
#define executable_suffix ".exe"
#else
#include <sys/stat.h>
#include <unistd.h>
#define get_process_id getpid
#define current_directory getcwd
#define create_directory(path) mkdir(path, 0700)
#define remove_directory rmdir
#define executable_suffix ""
#endif

static ShiroRsBytes *owned(const void *data, size_t count) {
    ShiroRsBytes *result = NULL; assert(shiro_rs_bytes_create(data, count, &result) == 0); return result;
}
static void equal_bytes(ShiroRsBytes *owner, const void *expected, size_t count) {
    size_t length = 99; assert(shiro_rs_bytes_length(owner, &length) == 0 && length == count);
    unsigned char *bytes = malloc(count + 1); assert(bytes);
    assert(shiro_rs_bytes_copy(owner, 0, bytes, count) == 0 && memcmp(bytes, expected, count) == 0); free(bytes);
}
static unsigned char *native_ascii(const char *text, uint32_t kind, size_t *count) {
    size_t length = strlen(text); *count = length * (kind == 2 ? 2 : 1); unsigned char *bytes = calloc(*count + 1, 1); assert(bytes);
    for (size_t i = 0; i < length; ++i) bytes[i * (kind == 2 ? 2 : 1)] = (unsigned char)text[i]; return bytes;
}
static ShiroRsPath *path_wire(const void *data, size_t count) {
    ShiroRsBytes *source = owned(data, count); ShiroRsPath *result = NULL;
    assert(shiro_rs_path_from_native_bytes(source, &result) == 0 && shiro_rs_bytes_release(&source) == 0); return result;
}
static ShiroRsPath *path_ascii(const char *text, uint32_t kind) {
    size_t count; unsigned char *bytes = native_ascii(text, kind, &count); ShiroRsPath *result = path_wire(bytes, count); free(bytes); return result;
}
static void equal_path(ShiroRsPath *owner, const void *expected, size_t count) {
    ShiroRsBytes *snapshot = NULL; assert(shiro_rs_path_native_bytes(owner, &snapshot) == 0);
    equal_bytes(snapshot, expected, count); assert(shiro_rs_bytes_release(&snapshot) == 0);
}
static unsigned char *load(const char *path, size_t *count) {
    FILE *file = fopen(path, "rb"); assert(file); assert(fseek(file, 0, SEEK_END) == 0); long length = ftell(file); assert(length >= 0); rewind(file);
    *count = (size_t)length; unsigned char *bytes = malloc(*count + 1); assert(bytes);
    assert(fread(bytes, 1, *count, file) == *count && fclose(file) == 0); return bytes;
}
static void save(const char *path, const void *data, size_t count) {
    FILE *file = fopen(path, "wb"); assert(file); assert(fwrite(data, 1, count, file) == count && fclose(file) == 0);
}
static void equal_files(const char *actual, const char *expected) {
    size_t a_count, e_count; unsigned char *a = load(actual, &a_count), *e = load(expected, &e_count);
    assert(a_count == e_count && memcmp(a, e, a_count) == 0); free(a); free(e);
}
static float scalar(const unsigned char *bytes) {
    uint32_t bits = (uint32_t)bytes[0] | (uint32_t)bytes[1] << 8 | (uint32_t)bytes[2] << 16 | (uint32_t)bytes[3] << 24;
    float value; memcpy(&value, &bits, sizeof(value)); return value;
}
static void output_fields(ShiroRsBatchOutputs *output, const char *stem, uint32_t kind, uint32_t mfcc) {
    uint32_t presence = 99; assert(shiro_rs_batch_outputs_has_mfcc(output, &presence) == 0 && presence == mfcc);
    const char *suffixes[] = {".raw", ".param", ".mfcc"};
    for (size_t i = 0; i < 3; ++i) {
        ShiroRsPath *path = NULL; assert(shiro_rs_batch_outputs_get_path(output, i, &path) == (i < 2 || mfcc ? 0 : 2));
        if (path) { char text[4096]; assert(snprintf(text, sizeof(text), "%s%s", stem, suffixes[i]) > 0); size_t count; unsigned char *bytes = native_ascii(text, kind, &count); equal_path(path, bytes, count); free(bytes); assert(shiro_rs_path_release(&path) == 0); }
    }
}
static uint32_t uniform(void *context, float *value) { ++*(size_t *)context; *value = 0.5f; return 0; }
static uint32_t failed_uniform(void *context, float *value) { (void)value; ++*(size_t *)context; return 7; }
int main(void) {
    uint32_t kind = shiro_rs_path_native_encoding(); assert(kind == 1 || kind == 2);
    ShiroRsBatchOptions *options = NULL; assert(shiro_rs_batch_options_default(&options) == 0);
    _Alignas(8) ShiroRsAudioOptions audio = {0}; assert(shiro_rs_batch_options_get_audio(options, &audio) == 0);
    assert(audio.normalize == 0 && audio.dither_level == 0 && audio.has_output_sample_rate == 0 && audio.output_sample_rate == 0 && audio.boundary == 0 && audio.kernel == 0);
    ShiroRsBytes *extension = NULL; assert(shiro_rs_batch_options_get_input_extension(options, &extension) == 0); equal_bytes(extension, ".wav", 4);
    assert(shiro_rs_bytes_release(&extension) == 0 && shiro_rs_batch_options_release(&options) == 0);
    const char suffix[] = ".source\0.wav"; extension = owned(suffix, sizeof(suffix) - 1);
    audio.normalize = 1; audio.dither_level = 0.125f; audio.has_output_sample_rate = 1; audio.output_sample_rate = 8000; audio.boundary = 1; audio.kernel = 1;
    assert(shiro_rs_batch_options_create(&audio, extension, &options) == 0 && shiro_rs_bytes_release(&extension) == 0);
    ShiroRsBatchOptions *cloned_options = NULL; assert(shiro_rs_batch_options_clone(options, &cloned_options) == 0 && shiro_rs_batch_options_release(&options) == 0);
    _Alignas(8) ShiroRsAudioOptions settings = {0}; assert(shiro_rs_batch_options_get_audio(cloned_options, &settings) == 0);
    assert(settings.normalize == 1 && settings.dither_level == 0.125f && settings.has_output_sample_rate == 1 && settings.output_sample_rate == 8000 && settings.boundary == 1 && settings.kernel == 1);
    assert(shiro_rs_batch_options_get_input_extension(cloned_options, &extension) == 0 && shiro_rs_batch_options_release(&cloned_options) == 0); equal_bytes(extension, suffix, sizeof(suffix) - 1); assert(shiro_rs_bytes_release(&extension) == 0);
    const unsigned char windows[] = {0x66,0,0,0xd8,0,0,0,0xdc}, unix_units[] = {0x66,0xff,0,0x80};
    const unsigned char *units = kind == 2 ? windows : unix_units; size_t count = kind == 2 ? sizeof(windows) : sizeof(unix_units);
    ShiroRsPath *source = path_wire(units, count), *empty = path_wire(NULL, 0); ShiroRsExtractor *extractor = NULL;
    for (uint32_t variant = 1; variant <= 2; ++variant) {
        assert((variant == 1 ? shiro_rs_extractor_sptk(source, source, source, &extractor) : shiro_rs_extractor_lua(source, source, source, &extractor)) == 0);
        ShiroRsExtractor *clone = NULL; assert(shiro_rs_extractor_clone(extractor, &clone) == 0 && shiro_rs_extractor_release(&extractor) == 0);
        uint32_t actual = 99; assert(shiro_rs_extractor_kind(clone, &actual) == 0 && actual == variant); actual = 99; assert(shiro_rs_extractor_get_preset(clone, &actual) == 2 && actual == 99);
        ShiroRsPath *paths[3] = {0}; for (size_t i = 0; i < 3; ++i) assert(shiro_rs_extractor_get_path(clone, i, &paths[i]) == 0);
        assert(shiro_rs_extractor_release(&clone) == 0); for (size_t i = 0; i < 3; ++i) { equal_path(paths[i], units, count); assert(shiro_rs_path_release(&paths[i]) == 0); }
    }
    for (uint32_t present = 0; present <= 1; ++present) {
        ShiroRsBatchOutputs *output = NULL, *clone = NULL; assert(shiro_rs_batch_outputs_create(source, source, present ? empty : NULL, &output) == 0);
        assert(shiro_rs_batch_outputs_clone(output, &clone) == 0 && shiro_rs_batch_outputs_release(&output) == 0);
        uint32_t presence = 99; assert(shiro_rs_batch_outputs_has_mfcc(clone, &presence) == 0 && presence == present);
        ShiroRsPath *paths[3] = {0}; for (size_t i = 0; i < 3; ++i) assert(shiro_rs_batch_outputs_get_path(clone, i, &paths[i]) == (i < 2 || present ? 0 : 2));
        assert(shiro_rs_batch_outputs_release(&clone) == 0);
        for (size_t i = 0; i < 3; ++i) if (paths[i]) { equal_path(paths[i], i < 2 ? units : (const unsigned char *)"", i < 2 ? count : 0); assert(shiro_rs_path_release(&paths[i]) == 0); }
    }
    assert(shiro_rs_path_release(&source) == 0 && shiro_rs_path_release(&empty) == 0);
    assert(shiro_rs_extractor_sptk_default(&extractor) == 0); const char *program_names[] = {"frame", "mfcc", "delta"};
    for (size_t i = 0; i < 3; ++i) { ShiroRsPath *path = NULL; assert(shiro_rs_extractor_get_path(extractor, i, &path) == 0); size_t n; unsigned char *bytes = native_ascii(program_names[i], kind, &n); equal_path(path, bytes, n); free(bytes); assert(shiro_rs_path_release(&path) == 0); }
    assert(shiro_rs_extractor_release(&extractor) == 0);
    char cwd[2048], directory[3072], stem[4096], filename[4096]; assert(current_directory(cwd, sizeof(cwd)));
    assert(snprintf(directory, sizeof(directory), "%s/target/c-api-batch-%u-%zu-%ld", cwd, kind, sizeof(void *), (long)get_process_id()) > 0 && create_directory(directory) == 0);
    assert(snprintf(stem, sizeof(stem), "%s/clip with spaces.v1", directory) > 0);
    unsigned char *wave = load("tests/fixtures/c-audio-input.wav", &count); assert(snprintf(filename, sizeof(filename), "%s.wav", stem) > 0); save(filename, wave, count); free(wave);
    ShiroRsPath *stem_owner = path_ascii(stem, kind); assert(shiro_rs_batch_options_default(&options) == 0);
    const char *originals[] = {"tests/fixtures/c-fextr-mfcc12-da.bin", "tests/fixtures/c-fextr-mfcc12-dae.bin", "tests/fixtures/c-fextr-plpcc12-da.bin"};
    for (uint32_t code = 0; code < 3; ++code) {
        _Alignas(8) ShiroRsFeatureOptions feature = {0}; assert(shiro_rs_batch_preset_feature_options(code, &feature) == 0);
        assert(feature.kind == (code == 2 ? 2 : 0) && feature.order == 12 && feature.channels == 36 && feature.frame_length == 512 && feature.hop == 80 && feature.sample_rate_hz == 16000 && feature.minimum_bandwidth_hz == 400 && feature.warp == 1 && feature.include_dc == 0 && feature.energy == (code == 1 ? 1 : 0) && feature.delta == 1 && feature.acceleration == 1);
        assert(shiro_rs_extractor_native(code, &extractor) == 0); uint32_t actual = 99; assert(shiro_rs_extractor_get_preset(extractor, &actual) == 0 && actual == code); assert(shiro_rs_extractor_kind(extractor, &actual) == 0 && actual == 0);
        ShiroRsBatchOutputs *output = NULL; assert(shiro_rs_batch_extract_file(stem_owner, options, extractor, NULL, NULL, &output) == 0); output_fields(output, stem, kind, 0);
        assert(snprintf(filename, sizeof(filename), "%s.raw", stem) > 0); equal_files(filename, "tests/fixtures/c-audio-input.plain.raw");
        assert(snprintf(filename, sizeof(filename), "%s.param", stem) > 0); size_t a_count, e_count; unsigned char *a = load(filename, &a_count), *e = load(originals[code], &e_count); assert(a_count == e_count);
        for (size_t i = 0; i < a_count; i += 4) { _Alignas(8) double av = scalar(a + i), ev = scalar(e + i); assert(fabs(av - ev) / fmax(fabs(ev), 1.0) < 2e-5); } free(a); free(e);
        ShiroRsExtractor *retained_extractor = extractor; assert(shiro_rs_extractor_native(3, &retained_extractor) == 2 && retained_extractor == extractor);
        _Alignas(8) ShiroRsFeatureOptions retained = feature; assert(shiro_rs_batch_preset_feature_options(3, &retained) == 2 && retained.kind == feature.kind);
        assert(shiro_rs_batch_extract_file(stem_owner, options, extractor, NULL, NULL, NULL) == 1);
        assert(shiro_rs_extractor_get_path(extractor, 0, &stem_owner) == 2);
        assert(shiro_rs_batch_outputs_release(&output) == 0 && shiro_rs_extractor_release(&extractor) == 0);
    }
    const char *tools = getenv("SHIRO_TEST_SPTK_DIRECTORY"), *lua = getenv("SHIRO_TEST_LUA"); assert(tools && lua);
    ShiroRsPath *programs[3] = {0}; for (size_t i = 0; i < 3; ++i) { assert(snprintf(filename, sizeof(filename), "%s/%s%s", tools, program_names[i], executable_suffix) > 0); programs[i] = path_ascii(filename, kind); }
    assert(shiro_rs_extractor_sptk(programs[0], programs[1], programs[2], &extractor) == 0); ShiroRsBatchOutputs *output = NULL;
    assert(shiro_rs_batch_extract_file(stem_owner, options, extractor, NULL, NULL, &output) == 0); output_fields(output, stem, kind, 1);
    const char *suffixes[] = {".raw", ".mfcc", ".param"}; for (size_t i = 0; i < 3; ++i) { assert(snprintf(filename, sizeof(filename), "%s%s", stem, suffixes[i]) > 0); equal_files(filename, "tests/fixtures/c-audio-input.plain.raw"); }
    assert(shiro_rs_extractor_release(&extractor) == 0); assert(snprintf(filename, sizeof(filename), "%s/mfcc-fail%s", tools, executable_suffix) > 0); ShiroRsPath *failing = path_ascii(filename, kind);
    assert(shiro_rs_extractor_sptk(programs[0], failing, programs[2], &extractor) == 0); ShiroRsBatchOutputs *retained_output = output;
    assert(shiro_rs_batch_extract_file(stem_owner, options, extractor, NULL, NULL, &retained_output) == 3 && retained_output == output);
    assert(shiro_rs_extractor_release(&extractor) == 0 && shiro_rs_batch_outputs_release(&output) == 0 && shiro_rs_path_release(&failing) == 0);
    for (size_t i = 0; i < 3; ++i) assert(shiro_rs_path_release(&programs[i]) == 0);
    char script[4096]; assert(snprintf(script, sizeof(script), "%s/extractor.lua", directory) > 0);
    const char body[] = "return function(try_execute, stem, rawfile, prefix) assert(try_execute == _G.try_execute); assert(#prefix > 0); local f = assert(io.open(rawfile, 'rb')); local bytes = f:read('*a'); f:close(); local o = assert(io.open(stem .. '.param', 'wb')); o:write(bytes); o:close() end\n"; save(script, body, sizeof(body) - 1);
    ShiroRsPath *lua_path = path_ascii(lua, kind), *script_path = path_ascii(script, kind), *directory_path = path_ascii(directory, kind);
    assert(shiro_rs_extractor_lua(lua_path, script_path, directory_path, &extractor) == 0);
    assert(shiro_rs_batch_extract_file(stem_owner, options, extractor, NULL, NULL, &output) == 0); output_fields(output, stem, kind, 0);
    assert(snprintf(filename, sizeof(filename), "%s.param", stem) > 0); equal_files(filename, "tests/fixtures/c-audio-input.plain.raw");
    save(script, "return 42\n", 10); retained_output = output;
    assert(shiro_rs_batch_extract_file(stem_owner, options, extractor, NULL, NULL, &retained_output) == 3 && retained_output == output);
    assert(shiro_rs_extractor_release(&extractor) == 0 && shiro_rs_batch_outputs_release(&output) == 0);
    assert(shiro_rs_path_release(&lua_path) == 0 && shiro_rs_path_release(&script_path) == 0 && shiro_rs_path_release(&directory_path) == 0 && shiro_rs_batch_options_release(&options) == 0);
    audio = (ShiroRsAudioOptions){0}; audio.dither_level = 0.125f; extension = owned(".wav", 4);
    assert(shiro_rs_batch_options_create(&audio, extension, &options) == 0 && shiro_rs_bytes_release(&extension) == 0 && shiro_rs_extractor_native(0, &extractor) == 0);
    size_t draws = 0; assert(shiro_rs_batch_extract_file(stem_owner, options, extractor, uniform, &draws, &output) == 0 && draws > 0);
    assert(snprintf(filename, sizeof(filename), "%s.raw", stem) > 0); equal_files(filename, "tests/fixtures/c-audio-input.plain.raw");
    retained_output = output; draws = 0; assert(shiro_rs_batch_extract_file(stem_owner, options, extractor, failed_uniform, &draws, &retained_output) == 3 && retained_output == output && draws == 1); equal_files(filename, "tests/fixtures/c-audio-input.plain.raw");
    extension = owned("\xff", 1); ShiroRsBatchOptions *retained_options = options; assert(shiro_rs_batch_options_create(&audio, extension, &retained_options) == 3 && retained_options == options); assert(shiro_rs_bytes_release(&extension) == 0);
    assert(shiro_rs_batch_options_clone(options, NULL) == 1 && shiro_rs_extractor_clone(extractor, NULL) == 1 && shiro_rs_batch_outputs_clone(output, NULL) == 1);
    assert(shiro_rs_batch_outputs_get_path(output, SIZE_MAX, &stem_owner) == 2);
    assert(shiro_rs_batch_outputs_release(&output) == 0 && shiro_rs_batch_options_release(&options) == 0 && shiro_rs_extractor_release(&extractor) == 0 && shiro_rs_path_release(&stem_owner) == 0);
    assert(shiro_rs_batch_outputs_release(NULL) == 1 && shiro_rs_batch_options_release(NULL) == 1 && shiro_rs_extractor_release(NULL) == 1);
    const char *cleanup[] = {".wav", ".raw", ".param", ".mfcc"}; for (size_t i = 0; i < 4; ++i) { assert(snprintf(filename, sizeof(filename), "%s%s", stem, cleanup[i]) > 0); assert(remove(filename) == 0); }
    assert(remove(script) == 0 && remove_directory(directory) == 0);
    puts("SHIRO batch C: all22 exports, all3 presets/full fields/actual Lua/SPTK protocol/RNG/ownership/failures passed"); return 0;
}
