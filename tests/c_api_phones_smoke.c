/* Actual C complete phone map transformations against original Lua fixtures. */
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shiro_rs.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "fixtures/phones_c_cases.h"

static ShiroRsBytes *owned(const char *text) {
    ShiroRsBytes *output = NULL;
    assert(shiro_rs_bytes_create((const uint8_t *)text, strlen(text), &output) == 0);
    return output;
}
static void equal_bytes(ShiroRsBytes *a, ShiroRsBytes *b) {
    size_t n = 0, m = 0;
    assert(shiro_rs_bytes_length(a, &n) == 0 && shiro_rs_bytes_length(b, &m) == 0 && n == m);
    unsigned char *left = malloc(n + 1), *right = malloc(n + 1); assert(left && right);
    assert(shiro_rs_bytes_copy(a, 0, left, n) == 0 && shiro_rs_bytes_copy(b, 0, right, n) == 0);
    assert(memcmp(left, right, n) == 0); free(left); free(right);
}
static void equal_models(ShiroRsBytes *actual, const char *expected_json) {
    ShiroRsBytes *expected = owned(expected_json), *a = NULL, *b = NULL;
    ShiroRsModel *left = NULL, *right = NULL;
    assert(shiro_rs_model_from_definition(actual, &left) == 0 && shiro_rs_model_from_definition(expected, &right) == 0);
    assert(shiro_rs_model_write_bytes(left, 0, &a) == 0 && shiro_rs_model_write_bytes(right, 0, &b) == 0);
    equal_bytes(a, b);
    assert(shiro_rs_bytes_release(&a) == 0 && shiro_rs_bytes_release(&b) == 0 && shiro_rs_bytes_release(&expected) == 0);
    assert(shiro_rs_model_release(&left) == 0 && shiro_rs_model_release(&right) == 0);
}
static char *copied(ShiroRsBytes *bytes) {
    size_t count = 0; assert(shiro_rs_bytes_length(bytes, &count) == 0);
    char *text = malloc(count + 1); assert(text);
    assert(shiro_rs_bytes_copy(bytes, 0, (uint8_t *)text, count) == 0); text[count] = 0; return text;
}
/* Both values use the same typed JSON serializer; compare every token, allowing
   only the established binary64 Lua numerical tolerance outside strings. */
static void equal_json(ShiroRsBytes *a, ShiroRsBytes *b) {
    char *left = copied(a), *right = copied(b), *x = left, *y = right;
    int quoted = 0, escaped = 0;
    while (*x && *y) {
        if (!quoted && (*x == '-' || (*x >= '0' && *x <= '9'))) {
            char *end_x, *end_y; double number_x = strtod(x, &end_x), number_y = strtod(y, &end_y);
            assert(end_x != x && end_y != y && fabs(number_x - number_y) <= 1e-14);
            x = end_x; y = end_y; continue;
        }
        assert(*x == *y);
        if (quoted) { if (escaped) escaped = 0; else if (*x == '\\') escaped = 1; else if (*x == '"') quoted = 0; }
        else if (*x == '"') quoted = 1;
        ++x; ++y;
    }
    assert(!*x && !*y && !quoted); free(left); free(right);
}
int main(void) {
    FILE *file = fopen("tests/fixtures/phones-input.txt", "rb"); assert(file);
    assert(fseek(file, 0, SEEK_END) == 0); long count = ftell(file); assert(count >= 0);
    assert(fseek(file, 0, SEEK_SET) == 0); char *text = malloc((size_t)count + 1); assert(text);
    assert(fread(text, 1, (size_t)count, file) == (size_t)count && fclose(file) == 0); text[count] = 0;
    ShiroRsBytes *input = owned(text), *names = owned("[\"bb\",\"aa\",\"bb\",\"cc\",\"aa\"]"); free(text);
    ShiroRsPhoneOptions config;
    assert(shiro_rs_phone_options_default(&config) == 0 && config.states_per_phone == 3 && config.streams == 3 && config.weak_skips == 0);
    for (size_t i = 0; i < sizeof(phone_cases)/sizeof(phone_cases[0]); ++i) {
        PhoneCase test = phone_cases[i]; config.states_per_phone = test.count; config.streams = 2; config.weak_skips = 1;
        ShiroRsBytes *topology = owned(test.topology), *a = NULL, *b = NULL;
        ShiroRsPhoneMap *map = NULL, *clone = NULL, *expected = NULL;
        assert(shiro_rs_phone_map_create(input, &config, topology, &map) == 0 && shiro_rs_bytes_release(&topology) == 0);
        assert(shiro_rs_phone_map_clone(map, &clone) == 0 && shiro_rs_phone_map_release(&map) == 0);
        ShiroRsBytes *expected_json = owned(test.map);
        assert(shiro_rs_phone_map_read_json(expected_json, &expected) == 0 && shiro_rs_bytes_release(&expected_json) == 0);
        assert(shiro_rs_phone_map_write_json(clone, &a) == 0 && shiro_rs_phone_map_write_json(expected, &b) == 0);
        equal_json(a, b); assert(shiro_rs_bytes_release(&a) == 0 && shiro_rs_bytes_release(&b) == 0);
        assert(shiro_rs_phone_map_release(&expected) == 0);
        ShiroRsStates *states = NULL, *expected_states = NULL;
        assert(shiro_rs_segmentation_initial(names, clone, 41, &states) == 0);
        expected_json = owned(test.states);
        assert(shiro_rs_states_read_json(expected_json, &expected_states) == 0 && shiro_rs_bytes_release(&expected_json) == 0);
        assert(shiro_rs_states_write_json(states, &a) == 0 && shiro_rs_states_write_json(expected_states, &b) == 0);
        equal_json(a, b);
        assert(shiro_rs_states_release(&states) == 0 && shiro_rs_states_release(&expected_states) == 0);
        assert(shiro_rs_bytes_release(&a) == 0 && shiro_rs_bytes_release(&b) == 0);
        if (test.definition) { assert(shiro_rs_phone_map_to_definition(clone, 12, 0.01, &a) == 0); equal_models(a, test.definition); assert(shiro_rs_bytes_release(&a) == 0); }
        ShiroRsPhoneMap *retained = clone; config.weak_skips = 2;
        assert(shiro_rs_phone_map_create(input, &config, NULL, &retained) == 2 && retained == clone);
        assert(shiro_rs_phone_map_to_definition(clone, 0, 0.01, &a) == 3 && !a);
        assert(shiro_rs_phone_map_release(&clone) == 0);
    }
    const char *metadata = "{\"phone_map\":{\"aa\":{\"states\":[{\"dur\":0,\"out\":[0],\"extra\":{\"x\":[1,null]}}],\"extra\":true}},\"extra\":{\"nested\":\"retained\"}}";
    ShiroRsBytes *encoded = owned(metadata), *wire = NULL; ShiroRsPhoneMap *map = NULL, *clone = NULL;
    assert(shiro_rs_phone_map_read_json(encoded, &map) == 0 && shiro_rs_phone_map_clone(map, &clone) == 0);
    assert(shiro_rs_phone_map_release(&map) == 0 && shiro_rs_phone_map_write_json(clone, &wire) == 0);
    /* JSON member order differs; round-trip the complete typed map then compare. */
    assert(shiro_rs_phone_map_read_json(wire, &map) == 0); ShiroRsBytes *second = NULL;
    assert(shiro_rs_phone_map_write_json(map, &second) == 0); equal_bytes(wire, second);
    assert(shiro_rs_bytes_release(&wire) == 0 && shiro_rs_bytes_release(&second) == 0 && shiro_rs_bytes_release(&encoded) == 0);
    assert(shiro_rs_phone_options_default(NULL) == 1 && shiro_rs_phone_map_release(NULL) == 1);
    assert(shiro_rs_phone_map_clone(clone, NULL) == 1 && shiro_rs_phone_map_release(&clone) == 0 && shiro_rs_phone_map_release(&clone) == 0);
    assert(shiro_rs_phone_map_release(&map) == 0 && shiro_rs_bytes_release(&input) == 0 && shiro_rs_bytes_release(&names) == 0);
    puts("SHIRO phones C: all8 exports, original Lua maps/definitions/states, ownership and failures passed");
    return 0;
}
