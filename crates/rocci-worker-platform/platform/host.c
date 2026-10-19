// Direct-symbol ABI for the pinned Roc compiler. No OS, WASI or arena reset.
#include <stddef.h>
#include <stdint.h>

#define HEAP_SIZE (8 * 1024 * 1024)
#define INPUT_SIZE (256 * 1024)
#define MAGIC 0x524f4343u
typedef struct Block {
    size_t span;
    size_t requested;
    struct Block *next;
    uint32_t magic;
    uint32_t resource;
    uint32_t alignment;
    uint32_t allocated;
} Block;
_Alignas(16) static unsigned char heap[HEAP_SIZE];
static Block *head;
static size_t allocated_bytes, allocated_count, high_water;
static unsigned char input[INPUT_SIZE];
typedef struct { unsigned char *bytes; size_t capacity; size_t length; } RocStr;
static RocStr result;
static void *context;
static int active, initialized;

__attribute__((import_module("rocci"), import_name("delay")))
extern int64_t host_delay(uint64_t value);
__attribute__((import_module("rocci"), import_name("resource_drop")))
extern void host_resource_drop(uint64_t id);

void *memcpy(void *dst, const void *src, size_t size) {
    unsigned char *a = dst; const unsigned char *b = src;
    for (size_t i = 0; i < size; i++) a[i] = b[i];
    return dst;
}
void *memset(void *dst, int value, size_t size) {
    unsigned char *a = dst;
    for (size_t i = 0; i < size; i++) a[i] = (unsigned char)value;
    return dst;
}
void *memmove(void *dst, const void *src, size_t size) {
    unsigned char *a = dst; const unsigned char *b = src;
    if ((uintptr_t)a < (uintptr_t)b) return memcpy(dst, src, size);
    for (size_t i = size; i > 0; i--) a[i - 1] = b[i - 1];
    return dst;
}

void *roc_alloc(size_t size, unsigned int alignment) {
    if (alignment < sizeof(size_t)) alignment = sizeof(size_t);
    if (alignment > 16 || (alignment & (alignment - 1))) __builtin_trap();
    if (!head) {
        head = (Block *)heap;
        *head = (Block){.span = HEAP_SIZE, .magic = MAGIC};
    }
    // Every payload is 16-byte aligned. Keep one offset immediately before it
    // so neither Roc's refcount prefix nor a boxed payload overlaps metadata.
    const size_t prefix = (sizeof(Block) + sizeof(size_t) + 15) & ~(size_t)15;
    if (size > HEAP_SIZE - prefix - 15) __builtin_trap();
    const size_t needed = (prefix + size + 15) & ~(size_t)15;
    for (Block *block = head; block; block = block->next) {
        if (block->magic != MAGIC) __builtin_trap();
        if (block->allocated || block->span < needed) continue;
        if (block->span - needed >= prefix + 16) {
            Block *next = (Block *)((unsigned char *)block + needed);
            *next = (Block){.span = block->span - needed, .next = block->next, .magic = MAGIC};
            block->next = next;
            block->span = needed;
        }
        block->requested = size;
        block->allocated = 1;
        block->alignment = alignment;
        block->resource = 0;
        unsigned char *ptr = (unsigned char *)block + prefix;
        ((size_t *)ptr)[-1] = prefix;
        allocated_bytes += size;
        allocated_count++;
        if (allocated_bytes > high_water) high_water = allocated_bytes;
        return ptr;
    }
    __builtin_trap();
}
static Block *block_for(void *ptr) {
    uintptr_t address = (uintptr_t)ptr, start = (uintptr_t)heap;
    if (address < start + sizeof(Block) + sizeof(size_t) || address >= start + HEAP_SIZE) __builtin_trap();
    size_t prefix = ((size_t *)ptr)[-1];
    if (prefix != ((sizeof(Block) + sizeof(size_t) + 15) & ~(size_t)15)) __builtin_trap();
    Block *block = (Block *)((unsigned char *)ptr - prefix);
    if (block->magic != MAGIC || !block->allocated) __builtin_trap();
    return block;
}
void roc_dealloc(void *ptr, unsigned int alignment) {
    if (!ptr) return;
    Block *block = block_for(ptr);
    if (alignment < sizeof(size_t)) alignment = sizeof(size_t);
    if (block->alignment != alignment) __builtin_trap();
    if (block->resource) host_resource_drop(*(uint64_t *)((unsigned char *)ptr + 8));
    allocated_bytes -= block->requested;
    allocated_count--;
    block->allocated = 0;
    for (Block *current = head; current && current->next;) {
        if (!current->allocated && !current->next->allocated) {
            current->span += current->next->span;
            current->next = current->next->next;
        } else current = current->next;
    }
}
void *roc_realloc(void *ptr, size_t size, unsigned int alignment) {
    if (!ptr) return roc_alloc(size, alignment);
    Block *old = block_for(ptr);
    if (old->resource) __builtin_trap();
    size_t old_size = old->requested;
    void *next = roc_alloc(size, alignment);
    memcpy(next, ptr, old_size < size ? old_size : size);
    roc_dealloc(ptr, alignment);
    return next;
}
void roc_dbg(void *bytes, size_t length) { (void)bytes; (void)length; }
void roc_expect_failed(void *bytes, size_t length) { (void)bytes; (void)length; __builtin_trap(); }
void roc_crashed(void *bytes, size_t length) { (void)bytes; (void)length; __builtin_trap(); }
void *roc_memcpy(void *dst, const void *src, size_t size) { return memcpy(dst, src, size); }
void *roc_memset(void *dst, int value, size_t size) { return memset(dst, value, size); }

extern void *roc_worker_init(void);
extern RocStr roc_worker_call(RocStr, void *);
extern uint64_t roc_worker_drop_string(RocStr);
extern uint64_t roc_worker_drop_context(void *);

uint32_t worker_abi_version(void) { return 1; }
unsigned char *input_ptr(void) { return input; }
size_t input_capacity(void) { return INPUT_SIZE; }
unsigned char *output_ptr(void) { return (intptr_t)result.length < 0 ? (unsigned char *)&result : result.bytes; }
size_t output_len(void) { return (intptr_t)result.length < 0 ? ((unsigned char *)&result)[sizeof(RocStr) - 1] ^ 128 : result.length; }
size_t live_bytes(void) { return allocated_bytes; }
size_t live_allocations(void) { return allocated_count; }
size_t heap_high_water(void) { return high_water; }
void worker_init(void) {
    if (active || initialized) __builtin_trap();
    active = 1;
    context = roc_worker_init();
    initialized = 1;
    active = 0;
}
void worker_invoke(size_t length) {
    if (active || !initialized || length > INPUT_SIZE) __builtin_trap();
    active = 1;
    roc_worker_drop_string(result);
    result = (RocStr){0};
    unsigned char *ptr = roc_alloc(length + sizeof(size_t), sizeof(size_t));
    *(intptr_t *)ptr = 1;
    memcpy(ptr + sizeof(size_t), input, length);
    // Context is a Box(Context); preserve the host's reference across calls.
    intptr_t *rc = (intptr_t *)context - 1;
    if (*rc > 0) (*rc)++;
    result = roc_worker_call((RocStr){ptr + sizeof(size_t), length << 1, length}, context);
    active = 0;
}
void worker_close(void) {
    if (active || !initialized) __builtin_trap();
    active = 1;
    roc_worker_drop_string(result);
    result = (RocStr){0};
    roc_worker_drop_context(context);
    context = 0;
    initialized = 0;
    active = 0;
}

RocStr hosted_worker_echo(RocStr value) { return value; }
typedef struct { uint64_t value; uint8_t tag; } DelayResult;
DelayResult hosted_worker_delay(uint64_t value) {
    int64_t result = host_delay(value);
    // Roc Try tags are alphabetical: Err=0, Ok=1.
    return (DelayResult){.value = result < 0 ? 0 : (uint64_t)result, .tag = result < 0 ? 0 : 1};
}
uint64_t *hosted_worker_resource(uint64_t id) {
    unsigned char *base = roc_alloc(16, 8);
    *(intptr_t *)(base + 8 - sizeof(intptr_t)) = 1;
    *(uint64_t *)(base + 8) = id;
    block_for(base)->resource = 1;
    return (uint64_t *)(base + 8);
}
uint64_t hosted_worker_resource_value(uint64_t *ptr) {
    uint64_t value = *ptr;
    intptr_t *rc = (intptr_t *)ptr - 1;
    if (*rc > 0 && --*rc == 0) roc_dealloc((unsigned char *)ptr - 8, 8);
    return value;
}
