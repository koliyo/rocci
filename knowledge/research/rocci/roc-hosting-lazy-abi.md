---
type: Research Report
title: Roc hosting API versus Component Model lazy ABI
description: "Roc's native hosting API already gets the outcomes the planned Component Model lazy ABI is chasing, via shared-address-space seamless slices and opaque host handles. Adopting canon value.lower inside Roc glue would add copies Roc already avoids. The real lazy ABI matters only if Roc becomes a WIT component with its own linear memory — not the current --http-module object-link path — and that ABI is not shipped."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-30T10:26:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: ba-roadmap
    resource: https://bytecodealliance.org/articles/the-road-to-component-model-1-0
    title: "The Road to Component Model 1.0 (2026-06-08): lazy ABI as 0.3.x opt-in, default at 1.0"
    author: organization:bytecode-alliance
  - id: cm-383
    resource: https://github.com/WebAssembly/component-model/issues/383
    title: "Lazy value lowering: opaque validx plus canon value.lower; first slice still a proposal"
    author: organization:bytecode-alliance
  - id: meld-watch
    resource: https://github.com/pulseengine/meld/issues/274
    title: Roadmap is announcement not spec; eager Canonical ABI stays until 0.3.x opt-in lands
    author: organization:pulseengine
  - id: wasi-async-cm
    resource: https://component-model.bytecodealliance.org/design/async.html
    title: Canonical ABI async func, stream, and future; distinct from lazy value lowering
    author: organization:bytecode-alliance
  - id: wasi-research
    resource: basic-webserver-wasi.md
    title: Blocking 0.16 C-ABI is CPU occupancy; option 1 is Rust component plus Roc object
    author: process:cursor
    last_modified: 2026-08-30
  - id: app-plan
    resource: ../../plans/rocci/wasi-http-03-app.md
    title: Keep 0.16 C-ABI; a new Roc ABI is out of bound
    author: process:cursor
    last_modified: 2026-08-30
  - id: adapter-abi
    resource: ../../../crates/rocci-wasi-http/src/abi.rs
    title: WASI adapter buffers the body into Vec instead of a streaming handle
    author: process:git
    last_modified: 2026-08-30
  - id: adapter-handle
    resource: ../../../crates/rocci-wasi-http/src/handle.rs
    title: handle buffers the body, then calls Roc
    author: process:git
    last_modified: 2026-08-30
  - id: apply-host
    resource: ../../../crates/rocci-roc-host/platform/host.c
    title: Apply wasm host.o is malloc plus roc_main, not HTTP
    author: process:git
    last_modified: 2026-08-18
  - id: bws-platform
    resource: ../../../../roc-basic-webserver/platform/main.roc
    title: provides roc_*_for_host; hosted table is linker symbols, not WIT
    author: process:git
    last_modified: 2026-08-30
  - id: bws-design
    resource: ../../../../roc-basic-webserver/design.md
    title: Seamless slices, host resource handles, native plans; copies are a fallback
    author: process:git
    last_modified: 2026-08-30
  - id: bws-glue
    resource: ../../../../roc-basic-webserver/src/roc_platform_abi.rs
    title: Generated C ABI; RocStr seamless-slice tag; hosted ownership transfer
    author: process:git
    last_modified: 2026-08-30
  - id: bws-parts
    resource: ../../../../roc-basic-webserver/src/request_parts.rs
    title: Request strings borrow Hyper URI and header storage without copying payload
    author: process:git
    last_modified: 2026-08-30
  - id: bws-body
    resource: ../../../../roc-basic-webserver/src/request_body.rs
    title: Body is an opaque handle; hosted reads return seamless chunks
    author: process:git
    last_modified: 2026-08-30
  - id: bws-heap
    resource: ../../../../roc-basic-webserver/src/host_resource.rs
    title: Finite host heap whose allocation prefix is ABI-compatible with Box(U64)
    author: process:git
    last_modified: 2026-08-30
---

# Roc hosting API versus Component Model lazy ABI

## Claim

Roc does not need a Component-Model-style lazy lowering *inside* its hosting
API. The native platform already keeps large bytes out of the Roc heap until
the app asks, and shares them without a payload copy when it does. The planned
lazy Canonical ABI is the right tool for a different geometry: two components
with **separate linear memories**, where today's `cabi_realloc` forces the
adapter to allocate and copy before the callee runs. Rocci's
`--http-module` path links Roc as a C-ABI object into one Rust component, so
that geometry is not the product ABI. The lazy ABI is also not shipped. Do not
block the [app link](/plans/rocci/wasi-http-03-app.md) on it, and do not treat
it as a reason to make Roc async.[^ba-roadmap][^cm-383][^wasi-research][^app-plan]

## What the lazy ABI is

The Component Model's current Canonical ABI is eager. A callee that returns a
`list` or `string` (or more values than fit in flat registers) causes the
adapter to call the caller's exported `cabi_realloc`, then copy every element
into that linear memory before the caller resumes. That works. It also
fragments heaps, makes large allocation failure awkward, costs one
host-to-guest call per list element, and fights custom allocators.[^ba-roadmap][^cm-383]

The planned replacement inverts control:

1. The adapter stores an opaque `i32` **value index** (`validx`) plus the
   eager length.
2. The callee chooses the destination address (batch, stack, inline, or a
   custom heap).
3. The callee calls a static builtin `canon value.lower [validx, dstp] -> []`
   to materialize the value there.
4. Unused indices are dropped at end-of-call. A later extension would let an
   intermediate component **forward** a `validx` without lowering it at
   all.[^cm-383]

The Bytecode Alliance 1.0 roadmap (2026-06-08) treats this as the headline ABI
change: opt-in `lazy` canon option in a 0.3.x release, default when 1.0 lands,
with an adapter from today's eager components. Implementation projects named
are wasm-tools, Wasmtime, and wit-bindgen. As of 2026-08-30 the spec issue is
still open; a first-slice PR (lazy lift of `list<list<u8>>` only; strings,
streams, and forwarding deferred) was proposed in May 2026 and has not
landed. Treat the article as a roadmap, not a normative ABI.[^ba-roadmap][^cm-383][^meld-watch]

Lazy value lowering is **not** WASI 0.3 async. Async adds `async func`,
`stream<T>`, and `future<T>` so a call can suspend. Lazy ABI is about *when
bytes hit linear memory* on a call that is already happening. Confusing the
two would redo the [WASI HTTP](basic-webserver-wasi.md) stall analysis under
the wrong name.[^wasi-async-cm][^wasi-research]

## What Roc's hosting API is

A Roc platform is a pair of linker tables, not a WIT world.[^bws-platform]

| Direction | Names | Shape |
| --- | --- | --- |
| Host → Roc | `roc_init_for_host`, `roc_respond_for_host`, `roc_sse_advance_for_host`, `roc_shutdown_for_host` | Natural C ABI; glue-generated structs |
| Roc → host | `hosted_*` (Env, Path, File, Sqlite, body read, …) | Same C ABI; Roc transfers ownership of refcounted arguments; the host must release them |
| Alloc | `roc_alloc` / `roc_realloc` / `roc_dealloc` | Host-supplied; Roc and the host share one heap |

`roc glue` emits the exact layouts (`RocStr`, `RocList`, `RocBox`). It does
not emit Canonical ABI lifts. Apply wasm's `host.c` is the same alloc story
without HTTP.[^bws-glue][^apply-host][^wasi-research]

The sibling `basic-webserver` design already states the performance
constraint the lazy ABI is chasing: the common request path should avoid
unnecessary copies, allocations, and Roc/host transitions. It then picks a
stronger primitive than "lower later into guest memory," because the host and
Roc share an address space.[^bws-design]

### Four host→Roc strategies (already designed)

| Strategy | What it does | Lazy-ABI analogue |
| --- | --- | --- |
| Reference-counted seamless slice | Roc `Str` / `List` is a view (pointer, length, ARC) into host backing. No payload copy. | Better than `value.lower`: there is nothing to lower |
| Ownership transfer | Move a compatible allocation across the boundary | Eager copy that the host already paid for |
| Native host plan | File / upload / download never enters Roc | Drop the unused `validx` |
| Bounded copy | Fallback when representation or lifetime cannot be shared | Today's `cabi_realloc` path |

A change that adds a payload copy to the common path must justify why the
first three cannot work.[^bws-design]

The shipped fork host follows that table:

- Request URI and header bytes are seamless `RocStr` views of Hyper
  `Parts`. The allocation pointer is a host-heap token; final ARC release
  drops the Hyper storage.[^bws-parts][^bws-heap]
- The request body is an opaque `Box(U64)` capability. `hosted_request_body_read`
  / `read_all` / `write_file` pull chunks; delivered bytes are seamless views
  of the original `Bytes` owner. The handler that never reads the body never
  materializes it as a Roc `List`.[^bws-body][^bws-platform]
- SQLite, TCP, file readers, and readiness are the same handle heap: Roc
  copies only the box pointer; the native value stays in a finite host
  slot.[^bws-heap][^bws-design]

Glue already knows the seamless-slice tag on `RocStr` / `RocList`. That is
Roc's native "lazy" bit: a small descriptor, payload allocated only when an
operation needs independently owned storage.[^bws-glue]

## Side-by-side

| Friction | Component Model today | Planned lazy ABI | Roc native hosting |
| --- | --- | --- | --- |
| Who allocates the destination | Adapter calls `cabi_realloc` | Callee chooses address, then `value.lower` | Host owns `roc_alloc`; seamless slices skip it |
| Unused large value | Already copied | Drop `validx` | Body handle / native plan; unused fields are cheap descriptors |
| Forward through a wrapper | Copy in, copy out | Forward the index | Seamless ARC or pass the handle |
| Separate linear memories | Required (components) | Required | Not the native case; not option 1 either |
| Custom allocator | Fights host-driven realloc | Callee-controlled | Host already *is* the allocator |
| Failure on huge list | Mid-adapter realloc | Callee can refuse before lower | Host bounds before Roc sees network lengths |

The problems the lazy ABI names are real **across a component boundary**.
They are the wrong diagnosis for a same-process Roc platform.

## Three surfaces

### 1. Native `rocci run` / basic-webserver

Do not replace seamless slices or host heaps with opaque `validx` plus a copy
into Roc memory. That would undo the design's preferred boundary and add the
copy the platform is written to avoid.[^bws-design][^bws-parts][^bws-body]

A hosted function that *returns* a large `List(Str)` to Roc (directory listing,
`env_dict`) is the one native case that looks like eager `cabi_realloc`: the
host must allocate Roc cells before return. A lazy "here is N strings, lower
them when you want" protocol would help only if Roc often dropped the result.
That is not the request path. Leave it.

### 2. Rocci option 1: Rust is the WASI component; Roc is a linked object

This is the [app-link](/plans/rocci/wasi-http-03-app.md) geometry. Roc and the
adapter share **one** wasm linear memory. Seamless slices and resource handles
still work if the backing lives in that memory. A new Roc ABI is out of
bound.[^app-plan][^wasi-research]

The copies that remain are **adapter policy**, not a missing Roc lazy ABI:

- `handle` buffers the WASI `stream<u8>` body into a `Vec<u8>` before
  `roc_respond_for_host`.[^adapter-handle][^adapter-abi]
- `ServerRequest.body` is that owned vector, not a streaming handle.[^adapter-abi]

WASI 0.3 already has the right lazy-over-time primitive for the *wire*:
`stream<u8>`. The adapter chooses to drain it so the 0.16 C-ABI stays
synchronous. Yield stays around Roc. That decision is recorded in the WASI
research; lazy Canonical ABI does not change it.[^wasi-research][^wasi-async-cm]

If Wasmtime later offers the `lazy` canon option, the **Rust**
`rocci-wasi-http-component` crate can opt in on its WIT exports and imports.
The linked Roc object would not see it.

### 3. Hypothetical: Roc emits a WIT component

Then yes. A Roc guest with its own linear memory, importing `wasi:http` and
exporting `handle`, would hit `cabi_realloc` on every list and string that
crosses the Canonical ABI. The lazy option (or the 1.0 default) would be the
correct compiler/glue target: callee-controlled alloc, drop unused headers,
forward a request body through a thin wrapper without a copy.

That is a Roc **compiler** project (WIT lowering, resources vs Roc ARC,
async `handle` vs sync `respond!`). It is not a glue-file tweak, and it is
explicitly out of the current WASI plans. Do not start it to "align with"
an unshipped 0.3.x option.[^app-plan][^ba-roadmap][^wasi-research]

## Could Roc benefit from a similar setup?

**The pattern, yes — and it already did.** Opaque handles for host-owned
resources, field accessors that materialize on use (`hosted_request_body_read`),
and native plans that never enter Roc *are* the similar setup. They are
stronger than `value.lower` where memories are shared, because the destination
can be the original bytes.

**The Component Model builtin, no — not for today's hosting API.** Importing
`canon value.lower` semantics into `roc glue` would teach the host to copy
into Roc-chosen addresses. Native hosts can already hand Roc a pointer.
Wasm object-link can too.

**A request-as-handle platform API** (pass one capability into `respond!`,
read path/headers/body through hosted getters) would look even more like lazy
lowering. The fork is already halfway there for the body. Doing that for
headers and target would be a `Server.Request` contract change, not an ABI
change, and it would cost every handler an extra hosted call per field. The
current seamless descriptors make unused header *payloads* cheap without that
tax. Do not take it unless a measurement shows header-list construction
dominates.

## Recommendation

1. Keep the 0.16 C-ABI and the option-1 link. Do not open a Roc lazy-hosting
   ABI project.[^app-plan]
2. Do not wait for the Component Model `lazy` canon option. It is a 0.3.x
   opt-in on a roadmap; the first spec slice has not landed.[^ba-roadmap][^cm-383][^meld-watch]
3. When the opt-in exists in Wasmtime / wit-bindgen, consider it on the Rust
   component crate only.
4. If the adapter later stops buffering bodies, copy the fork's body-handle
   plus seamless chunk pattern into wasm linear memory — not `value.lower`.
5. Keep lazy ABI out of the blocking-handler story. It does not add an await
   point inside `roc_respond_for_host`.[^wasi-research][^wasi-async-cm]

## What this does not change

- [App-link](/plans/rocci/wasi-http-03-app.md) phases, `--host wasm`, or
  `rocci run`.
- The measured C-ABI stall (CPU occupancy; hosted-sleep C does not park).
- Making Roc language-async.

[^ba-roadmap]: Roadmap: invert `cabi_realloc`; opt-in 0.3.x; default at 1.0.
[^cm-383]: `validx` plus `value.lower`; drop unused; forwarding later; first slice proposed, not merged.
[^meld-watch]: Article is not a normative ABI; eager path stays until the opt-in ships.
[^wasi-async-cm]: Async primitives are `async func` / `stream` / `future`.
[^wasi-research]: Option 1; yield around Roc; C-ABI has no Canonical ABI await point.
[^app-plan]: Keep 0.16 C-ABI; new Roc ABI out of bound.
[^adapter-abi]: `ServerRequest.body` is an owned `Vec<u8>`.
[^adapter-handle]: Buffer then call Roc.
[^apply-host]: Apply `host.c` is alloc plus `roc_main`.
[^bws-platform]: `provides` / `hosted` linker tables.
[^bws-design]: Seamless slice preferred; copy is a fallback; native plans for transport.
[^bws-glue]: Generated C ABI and seamless-slice tag.
[^bws-parts]: Hyper `Parts` borrowed as seamless `RocStr`.
[^bws-body]: Body capability; hosted read returns seamless chunks.
[^bws-heap]: `Box(U64)`-compatible host resource heap.
