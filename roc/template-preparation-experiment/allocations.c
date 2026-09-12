// macOS-only optional measurement of intercepted libc allocation calls.
// Counts requested bytes, including realloc requests; not live/peak bytes.
// These are process totals, not exhaustive Roc allocator instrumentation.
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static _Atomic uint64_t allocations, reallocations, frees, bytes;

static void *count_malloc(size_t size) {
    atomic_fetch_add(&allocations, 1);
    atomic_fetch_add(&bytes, size);
    return malloc(size);
}

static void *count_calloc(size_t count, size_t size) {
    atomic_fetch_add(&allocations, 1);
    atomic_fetch_add(&bytes, count * size);
    return calloc(count, size);
}

static void *count_realloc(void *ptr, size_t size) {
    atomic_fetch_add(&reallocations, 1);
    atomic_fetch_add(&bytes, size);
    return realloc(ptr, size);
}

static void count_free(void *ptr) {
    atomic_fetch_add(&frees, 1);
    free(ptr);
}

#define INTERPOSE(replacement, original)                                        \
    __attribute__((used)) static struct {                                      \
        const void *new_fn;                                                    \
        const void *old_fn;                                                    \
    } interpose_##original __attribute__((section("__DATA,__interpose"))) = {   \
        (const void *)&replacement, (const void *)&original                    \
    }

INTERPOSE(count_malloc, malloc);
INTERPOSE(count_calloc, calloc);
INTERPOSE(count_realloc, realloc);
INTERPOSE(count_free, free);

__attribute__((destructor)) static void report(void) {
    fprintf(stderr,
            "ALLOCATIONS {\"allocations\":%llu,\"reallocations\":%llu,"
            "\"frees\":%llu,\"requested_bytes\":%llu}\n",
            (unsigned long long)atomic_load(&allocations),
            (unsigned long long)atomic_load(&reallocations),
            (unsigned long long)atomic_load(&frees),
            (unsigned long long)atomic_load(&bytes));
}
