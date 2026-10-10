/* Actual C complete index fields, original rows and lossless native paths. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ShiroRsBytes *owned(const void *data, size_t count) {
    ShiroRsBytes *value = NULL; assert(shiro_rs_bytes_create(data, count, &value) == 0); return value;
}
static void equal_bytes(ShiroRsBytes *owner, const void *expected, size_t count) {
    size_t length = 99; assert(shiro_rs_bytes_length(owner, &length) == 0 && length == count);
    unsigned char *copy = malloc(count + 1); assert(copy);
    assert(shiro_rs_bytes_copy(owner, 0, copy, count) == 0 && memcmp(copy, expected, count) == 0); free(copy);
}
static void equal_string(ShiroRsStrings *owner, size_t index, const void *expected, size_t count) {
    ShiroRsBytes *value = NULL; assert(shiro_rs_strings_get(owner, index, &value) == 0);
    equal_bytes(value, expected, count); assert(shiro_rs_bytes_release(&value) == 0);
}
static ShiroRsStrings *one(const void *data, size_t count) {
    ShiroRsBytes *value = owned(data, count); const ShiroRsBytes *pointers[] = {value}; ShiroRsStrings *result = NULL;
    assert(shiro_rs_strings_create(pointers, 1, &result) == 0 && shiro_rs_bytes_release(&value) == 0); return result;
}
static void equal_path(ShiroRsPath *owner, const unsigned char *expected, size_t count) {
    ShiroRsBytes *value = NULL; assert(shiro_rs_path_native_bytes(owner, &value) == 0);
    equal_bytes(value, expected, count); assert(shiro_rs_bytes_release(&value) == 0);
}
static unsigned char *native_ascii(const char *text, size_t count, uint32_t kind, size_t *length) {
    *length = count * (kind == 2 ? 2 : 1); unsigned char *value = calloc(*length + 1, 1); assert(value);
    for (size_t i = 0; i < count; ++i) value[i * (kind == 2 ? 2 : 1)] = (unsigned char)text[i]; return value;
}
int main(void) {
    uint32_t kind = shiro_rs_path_native_encoding(); assert(kind == 1 || kind == 2);
    ShiroRsBytes *source = owned("data", 4); ShiroRsPath *directory = NULL;
    assert(shiro_rs_path_from_utf8(source, &directory) == 0 && shiro_rs_bytes_release(&source) == 0);
    ShiroRsStrings *left = one("left", 4), *right = one("right", 5);
    FILE *file = fopen("tests/fixtures/index-original.txt", "rb"); assert(file);
    assert(fseek(file, 0, SEEK_END) == 0); long size = ftell(file); assert(size >= 0); rewind(file);
    unsigned char *contents = malloc((size_t)size + 1); assert(contents);
    assert(fread(contents, 1, (size_t)size, file) == (size_t)size && fclose(file) == 0);
    source = owned(contents, (size_t)size); free(contents); ShiroRsIndexEntries *entries = NULL;
    assert(shiro_rs_index_read_bytes(source, directory, left, right, &entries) == 0);
    size_t length = 99; assert(shiro_rs_index_entries_length(entries, &length) == 0 && length == 3);
    const char *names[] = {"clip one", "sub/clip.two", "silent"};
    const char *phones0[] = {"left", "aa", "bb", "right"};
    const char *phones1[] = {"left", "cc", "", "dd", "right"};
    const char *phones2[] = {"left", "right"};
    const char **phones[] = {phones0, phones1, phones2}; const size_t counts[] = {4, 5, 2};
    for (size_t i = 0; i < 3; ++i) {
        ShiroRsPath *stem = NULL; ShiroRsStrings *values = NULL;
        assert(shiro_rs_index_entries_get_stem(entries, i, &stem) == 0);
        assert(shiro_rs_index_entries_get_phonemes(entries, i, &values) == 0);
        char path[64]; int n = snprintf(path, sizeof(path), "data%c%s", kind == 2 ? '\\' : '/', names[i]); assert(n > 0);
        size_t wire_length; unsigned char *wire = native_ascii(path, (size_t)n, kind, &wire_length);
        equal_path(stem, wire, wire_length); free(wire);
        assert(shiro_rs_strings_length(values, &length) == 0 && length == counts[i]);
        for (size_t j = 0; j < counts[i]; ++j) equal_string(values, j, phones[i][j], strlen(phones[i][j]));
        assert(shiro_rs_path_release(&stem) == 0 && shiro_rs_strings_release(&values) == 0);
    }
    assert(shiro_rs_index_entries_release(&entries) == 0 && shiro_rs_bytes_release(&source) == 0);
    assert(shiro_rs_strings_release(&left) == 0 && shiro_rs_strings_release(&right) == 0);
    /* Full whitespace/empty/NUL padding, skipped rows and CRLF. */
    const char padding[] = "pad\0end"; left = one(padding, sizeof(padding) - 1); right = one(" right ", 7);
    const char rows[] = "\r\nclip, aa  bb \r\n\nsilent,\n"; source = owned(rows, sizeof(rows) - 1);
    assert(shiro_rs_index_read_bytes(source, directory, left, right, &entries) == 0);
    ShiroRsStrings *values = NULL; assert(shiro_rs_index_entries_get_phonemes(entries, 0, &values) == 0);
    assert(shiro_rs_strings_length(values, &length) == 0 && length == 7);
    equal_string(values, 0, padding, sizeof(padding) - 1);
    equal_string(values, 1, "", 0); equal_string(values, 2, "aa", 2); equal_string(values, 3, "", 0);
    equal_string(values, 4, "bb", 2); equal_string(values, 5, "", 0); equal_string(values, 6, " right ", 7);
    assert(shiro_rs_strings_release(&values) == 0 && shiro_rs_index_entries_release(&entries) == 0 && shiro_rs_bytes_release(&source) == 0);
    const char *bad[] = {"\n\r\nbad\n", "ok,a\n\n,b\n", "ok,a\n\nclip,a,b\n", "clip,\xff\n"};
    for (size_t i = 0; i < 4; ++i) {
        source = owned(bad[i], strlen(bad[i])); entries = NULL;
        assert(shiro_rs_index_read_bytes(source, directory, left, right, &entries) == 3 && entries == NULL);
        assert(shiro_rs_bytes_release(&source) == 0);
    }
    assert(shiro_rs_path_release(&directory) == 0 && shiro_rs_strings_release(&left) == 0 && shiro_rs_strings_release(&right) == 0);
    /* Every native unit, repeated owners and independent full-field snapshots. */
    const unsigned char windows[] = {0x66,0,0,0xd8,0,0,0,0xdc}, unix_units[] = {0x66,0xff,0,0x80};
    const unsigned char *expected = kind == 2 ? windows : unix_units; size_t count = kind == 2 ? sizeof(windows) : sizeof(unix_units);
    source = owned(expected, count); assert(shiro_rs_path_from_native_bytes(source, &directory) == 0 && shiro_rs_bytes_release(&source) == 0);
    const char text[] = "repeat\0 "; source = owned(text, sizeof(text) - 1); ShiroRsBytes *empty = owned("", 0);
    const ShiroRsBytes *pointers[] = {source, empty, source}; ShiroRsStrings *clone = NULL;
    assert(shiro_rs_strings_create(pointers, 3, &values) == 0 && shiro_rs_strings_clone(values, &clone) == 0);
    assert(shiro_rs_bytes_release(&source) == 0 && shiro_rs_bytes_release(&empty) == 0);
    _Alignas(8) ShiroRsIndexEntryInput input[] = {{directory, values}, {directory, values}};
    assert(shiro_rs_index_entries_create(input, 2, &entries) == 0);
    assert(shiro_rs_path_release(&directory) == 0 && shiro_rs_strings_release(&values) == 0);
    ShiroRsIndexEntries *copied = NULL; assert(shiro_rs_index_entries_clone(entries, &copied) == 0 && shiro_rs_index_entries_release(&entries) == 0);
    assert(shiro_rs_index_entries_length(copied, &length) == 0 && length == 2);
    assert(shiro_rs_index_entries_get_stem(copied, 1, &directory) == 0 && shiro_rs_index_entries_get_phonemes(copied, 1, &values) == 0);
    assert(shiro_rs_index_entries_release(&copied) == 0); equal_path(directory, expected, count);
    equal_string(values, 0, text, sizeof(text) - 1); equal_string(values, 1, "", 0); equal_string(values, 2, text, sizeof(text) - 1);
    equal_string(clone, 2, text, sizeof(text) - 1);
    ShiroRsStrings *no_padding = NULL; assert(shiro_rs_strings_create(NULL, 0, &no_padding) == 0);
    source = owned("child,\n", 7); assert(shiro_rs_index_read_bytes(source, directory, no_padding, no_padding, &entries) == 0);
    ShiroRsPath *joined = NULL; assert(shiro_rs_index_entries_get_stem(entries, 0, &joined) == 0);
    size_t suffix_length; unsigned char *suffix = native_ascii(kind == 2 ? "\\child" : "/child", 6, kind, &suffix_length);
    unsigned char *joined_wire = malloc(count + suffix_length); assert(joined_wire);
    memcpy(joined_wire, expected, count); memcpy(joined_wire + count, suffix, suffix_length); equal_path(joined, joined_wire, count + suffix_length);
    free(suffix); free(joined_wire); assert(shiro_rs_path_release(&joined) == 0 && shiro_rs_bytes_release(&source) == 0);
    assert(shiro_rs_index_entries_get_stem(entries, SIZE_MAX, &directory) == 2 && shiro_rs_index_entries_get_phonemes(entries, SIZE_MAX, &values) == 2);
    ShiroRsStrings *retained = values; const unsigned char invalid[] = {0xff}; source = owned("valid", 5); ShiroRsBytes *bad_bytes = owned(invalid, 1);
    const ShiroRsBytes *bad_strings[] = {source, bad_bytes};
    assert(shiro_rs_strings_create(bad_strings, 2, &retained) == 3 && retained == values);
    assert(shiro_rs_strings_create(NULL, SIZE_MAX, &retained) == 2 && retained == values);
    const ShiroRsBytes *null_owner[] = {NULL}; assert(shiro_rs_strings_create(null_owner, 1, &retained) == 1 && retained == values);
    assert(shiro_rs_strings_get(values, SIZE_MAX, &source) == 2); equal_bytes(source, "valid", 5);
    ShiroRsIndexEntries *retained_entries = entries;
    _Alignas(8) ShiroRsIndexEntryInput bad_entries[] = {{directory, values}, {NULL, values}};
    assert(shiro_rs_index_entries_create(bad_entries, 2, &retained_entries) == 1 && retained_entries == entries);
    assert(shiro_rs_index_entries_create(NULL, SIZE_MAX, &retained_entries) == 2 && retained_entries == entries);
    assert(shiro_rs_index_entries_release(&entries) == 0 && shiro_rs_index_entries_create(NULL, 0, &entries) == 0);
    assert(shiro_rs_index_entries_length(entries, &length) == 0 && length == 0);
    assert(shiro_rs_index_entries_release(&entries) == 0 && shiro_rs_index_entries_release(&entries) == 0 && shiro_rs_index_entries_release(NULL) == 1);
    assert(shiro_rs_strings_release(NULL) == 1);
    assert(shiro_rs_bytes_release(&source) == 0 && shiro_rs_bytes_release(&bad_bytes) == 0);
    assert(shiro_rs_path_release(&directory) == 0 && shiro_rs_strings_release(&values) == 0 && shiro_rs_strings_release(&clone) == 0 && shiro_rs_strings_release(&no_padding) == 0);
    puts("SHIRO index C: all12 exports, original index/full fields/native units/padding/ownership/failures passed");
    return 0;
}
