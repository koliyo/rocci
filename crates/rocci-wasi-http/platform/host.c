#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

void *roc_alloc(size_t size, unsigned int alignment) {
    (void)alignment;
    return malloc(size);
}

void *roc_realloc(void *ptr, size_t new_size, size_t old_size, unsigned int alignment) {
    (void)old_size;
    (void)alignment;
    return realloc(ptr, new_size);
}

void roc_dealloc(void *ptr, unsigned int alignment) {
    (void)alignment;
    free(ptr);
}

void roc_panic(void *msg, unsigned int tag_id) {
    (void)msg;
    (void)tag_id;
    __builtin_trap();
}

void *roc_memcpy(void *dest, const void *src, size_t n) {
    return memcpy(dest, src, n);
}

void *roc_memset(void *s, int c, size_t n) {
    return memset(s, s ? c : c, n);
}

void roc_dbg(void *loc, void *src, void *extra) {
    (void)loc;
    (void)src;
    (void)extra;
}
