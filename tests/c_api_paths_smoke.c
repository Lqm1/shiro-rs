/* Actual C lossless native path units, suffixes and independent owner snapshots. */
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
    ShiroRsBytes *output = NULL; assert(shiro_rs_bytes_create(data, count, &output) == 0); return output;
}
static void equal(ShiroRsBytes *owner, const uint8_t *expected, size_t count) {
    size_t n = 99; assert(shiro_rs_bytes_length(owner, &n) == 0 && n == count);
    uint8_t *values = malloc(count + 1); assert(values);
    assert(shiro_rs_bytes_copy(owner, 0, values, count) == 0 && memcmp(values, expected, count) == 0); free(values);
}
static uint8_t *native_ascii(const char *text, size_t count, uint32_t kind, size_t *length) {
    *length = count * (kind == 2 ? 2 : 1); uint8_t *values = calloc(*length + 1, 1); assert(values);
    for (size_t i = 0; i < count; ++i) values[i * (kind == 2 ? 2 : 1)] = (uint8_t)text[i]; return values;
}
int main(void) {
    uint32_t kind = shiro_rs_path_native_encoding(); assert(kind == 1 || kind == 2);
    const uint8_t windows[] = {0x66,0,0,0xd8,0,0,0,0xdc,0x2f,0,0x6f,0};
    const uint8_t unix_bytes[] = {0x66,0xff,0x80,0,0xfe,0x2f,0x6f};
    const uint8_t *expected = kind == 2 ? windows : unix_bytes;
    size_t count = kind == 2 ? sizeof(windows) : sizeof(unix_bytes);
    ShiroRsBytes *source = owned(expected, count); ShiroRsPath *path = NULL, *clone = NULL;
    assert(shiro_rs_path_from_native_bytes(source, &path) == 0 && shiro_rs_bytes_release(&source) == 0);
    assert(shiro_rs_path_clone(path, &clone) == 0 && shiro_rs_path_release(&path) == 0);
    const char suffix[] = ".param\0end"; source = owned(suffix, sizeof(suffix) - 1);
    assert(shiro_rs_path_append_suffix(clone, source, &path) == 0 && shiro_rs_bytes_release(&source) == 0);
    ShiroRsBytes *original = NULL, *appended = NULL;
    assert(shiro_rs_path_native_bytes(clone, &original) == 0 && shiro_rs_path_native_bytes(path, &appended) == 0);
    assert(shiro_rs_path_release(&clone) == 0 && shiro_rs_path_release(&path) == 0); equal(original, expected, count);
    size_t suffix_length = 0; uint8_t *suffix_bytes = native_ascii(suffix, sizeof(suffix) - 1, kind, &suffix_length);
    uint8_t *complete = malloc(count + suffix_length); assert(complete);
    memcpy(complete, expected, count); memcpy(complete + count, suffix_bytes, suffix_length); equal(appended, complete, count + suffix_length);
    free(complete); free(suffix_bytes); assert(shiro_rs_bytes_release(&original) == 0 && shiro_rs_bytes_release(&appended) == 0);
    const char *texts[] = {"", "data/sub/voice", "C:\\folder\\clip.wav"};
    for (size_t i = 0; i < 3; ++i) {
        source = owned(texts[i], strlen(texts[i]));
        assert(shiro_rs_path_from_utf8(source, &path) == 0 && shiro_rs_bytes_release(&source) == 0);
        assert(shiro_rs_path_native_bytes(path, &original) == 0);
        size_t length = 0; uint8_t *encoded = native_ascii(texts[i], strlen(texts[i]), kind, &length); equal(original, encoded, length); free(encoded);
        assert(shiro_rs_bytes_release(&original) == 0);
        const uint8_t invalid_utf8[] = {0xff}; ShiroRsBytes *invalid = owned(invalid_utf8, 1); ShiroRsPath *retained = path;
        assert(shiro_rs_path_from_utf8(invalid, &retained) == 3 && retained == path);
        assert(shiro_rs_path_append_suffix(path, invalid, &retained) == 3 && retained == path);
        if (kind == 2) assert(shiro_rs_path_from_native_bytes(invalid, &retained) == 3 && retained == path);
        assert(shiro_rs_path_from_native_bytes(NULL, &retained) == 1 && retained == path);
        assert(shiro_rs_path_from_utf8(NULL, &retained) == 1 && retained == path);
        assert(shiro_rs_path_clone(path, NULL) == 1 && shiro_rs_path_native_bytes(path, NULL) == 1 && shiro_rs_path_release(NULL) == 1);
        assert(shiro_rs_bytes_release(&invalid) == 0 && shiro_rs_path_release(&path) == 0 && shiro_rs_path_release(&path) == 0);
    }
    /* Non-BMP Unicode and an embedded NUL remain length-delimited. */
    const uint8_t unicode[] = {'v',0,0xf0,0x9f,0x99,0x82};
    const uint8_t unicode_wide[] = {'v',0,0,0,0x3d,0xd8,0x42,0xde};
    source = owned(unicode, sizeof(unicode)); assert(shiro_rs_path_from_utf8(source, &path) == 0 && shiro_rs_bytes_release(&source) == 0);
    assert(shiro_rs_path_native_bytes(path, &original) == 0 && shiro_rs_path_release(&path) == 0);
    equal(original, kind == 2 ? unicode_wide : unicode, kind == 2 ? sizeof(unicode_wide) : sizeof(unicode));
    assert(shiro_rs_bytes_release(&original) == 0);
    puts("SHIRO paths C: all7 exports, full native units/Unicode/empty/suffixes and ownership passed"); return 0;
}
