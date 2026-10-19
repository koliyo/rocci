// Invocation arena: no allocation survives invoke(). No imports or WASI.
#include <stddef.h>
#include <stdint.h>
#ifndef __wasm__
#include <stdio.h>
#endif

_Alignas(16) static unsigned char arena[4 * 1024 * 1024];
static unsigned char input[256 * 1024];
static size_t used;
typedef struct { unsigned char *bytes; size_t capacity; size_t length; } RocStr;
static RocStr result;

void *roc_alloc(size_t size, unsigned int alignment) {
    if (alignment < sizeof(size_t)) alignment = sizeof(size_t);
    if (alignment > 16 || (alignment & (alignment - 1))) __builtin_trap();
    size_t pos = (used + sizeof(size_t) + alignment - 1) & ~(alignment - 1);
    if (pos > sizeof arena || size > sizeof arena - pos) __builtin_trap();
    used = pos + size;
    *((size_t *)(arena + pos) - 1) = size;
    return arena + pos;
}
void roc_dealloc(void *ptr, unsigned int alignment) { (void)ptr; (void)alignment; }
void *memcpy(void *dst, const void *src, size_t size) {
    unsigned char *a = dst; const unsigned char *b = src;
    for (size_t i = 0; i < size; i++) a[i] = b[i];
    return dst;
}
void *memset(void *dst, int value, size_t size) {
    unsigned char *a = dst;
    for (size_t i = 0; i < size; i++) a[i] = value;
    return dst;
}
void *memmove(void *dst, const void *src, size_t size) {
    unsigned char *a = dst; const unsigned char *b = src;
    if (a < b) return memcpy(dst, src, size);
    for (size_t i = size; i > 0; i--) a[i - 1] = b[i - 1];
    return dst;
}
void *roc_realloc(void *ptr, size_t size, unsigned int alignment) {
    void *result = roc_alloc(size, alignment);
    size_t old = ptr ? *((size_t *)ptr - 1) : 0;
    if (old) memcpy(result, ptr, size < old ? size : old);
    return result;
}
void roc_dbg(void *bytes, size_t length) { (void)bytes; (void)length; }
void roc_expect_failed(void *bytes, size_t length) { (void)bytes; (void)length; __builtin_trap(); }
void roc_crashed(void *bytes, size_t length) { (void)bytes; (void)length; __builtin_trap(); }
void *roc_memcpy(void *dst, const void *src, size_t size) { return memcpy(dst, src, size); }
void *roc_memset(void *dst, int value, size_t size) { return memset(dst, value, size); }
extern RocStr roc_call(RocStr);
unsigned char *input_ptr(void) { return input; }
unsigned char *output_ptr(void) { return (intptr_t)result.length < 0 ? (unsigned char *)&result : result.bytes; }
size_t output_len(void) { return (intptr_t)result.length < 0 ? ((unsigned char *)&result)[sizeof(RocStr) - 1] ^ 128 : result.length; }
size_t arena_used(void) { return used; }
void invoke(size_t length) {
    if (length > sizeof input) __builtin_trap();
    used = 0;
    unsigned char *ptr = roc_alloc(length + sizeof(size_t), sizeof(size_t));
    *(intptr_t *)ptr = 0; // Roc's static-data refcount; arena owns the whole invocation.
    memcpy(ptr + sizeof(size_t), input, length);
    // Capacity is shifted left: its low bit tags a seamless slice allocation.
    result = roc_call((RocStr){ptr + sizeof(size_t), length << 1, length});
}
#ifndef __wasm__
int main(void) {
    size_t length = fread(input, 1, sizeof input, stdin);
    if (!feof(stdin)) return 2;
    invoke(length);
    return fwrite(output_ptr(), 1, output_len(), stdout) == output_len() ? 0 : 3;
}
#endif
