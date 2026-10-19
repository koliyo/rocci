# Rocci Worker platform

`rocci-worker-platform` hosts ordinary effectful Roc programs as core Wasm.
The CLI embeds its compatible platform sources and JavaScript adapter. It
reuses the native platform's generated Wasm32 layouts; JavaScript exchanges
versioned, domain independent envelopes and resource IDs. No application
record codec or handwritten route table is required.

The qualified compiler is `nightly-2026-10-09-258ab27`, with specialization
**enabled**, in `dev`, `speed` and `size` modes. The native `.roc-version` is
independent. Unspecialized builds are unsupported. Provider inputs are Node 26.12.0,
Wrangler 4.151.1 / workerd 1.20261012.1 and compatibility date 2026-10-19.
A bounded hosted preview passed the actual Roc JSPI fixture in all three modes;
disposable hosted SQLite/transaction/live acceptance also passed separately.

The shared platform modules originate from basic-webserver at
`50e064cdd1c4562c293598c61f6ce7a895d99bcf`. Copyright © 2023 Richard
Feldman and subsequent Roc authors; the UPL text is in `LICENSE-UPL`.
Rocci original code is Apache-2.0.

## Application workflow

```sh
rocci worker init path/to/app --provider cloudflare
cd path/to/app
npm install
npm run dev
npm run deploy
```

Wrangler calls `rocci worker build`. Generated Roc, Wasm, adapter, assets,
source maps and a hash manifest live in `.rocci/worker/`. `rocci-worker.lock`
records compiler, ABI, host and runtime hashes. Commit that file; CI can use
`rocci worker build --locked`. A failed compilation preserves the previous
output tree. Cached output is accepted only if every output hash still matches.
The default singleton identity and Wrangler class/migration are stable across
builds. Secrets remain in normal Wrangler configuration and secret bindings.

Automatic toolchain downloads support Apple Silicon macOS and x64 Linux.
Normal app builds need neither Cargo nor clang. Prebuilt releases supply
`rocci-worker-host.a` and its checksum. Maintainers can build it with
`cargo build --release -p rocci-worker-platform --target wasm32-unknown-unknown`
and pass `--host-archive`. `--roc` / `ROCCI_WORKER_ROC` select the exact compiler;
`ROCCI_WORKER_HOST` selects an archive. `ROCCI_CACHE` relocates the cache.
`--offline` requires preseeded compiler/host, Datastar and format-3 leaf Roc
package caches; missing or transitive package manifests fail before compilation.
The archive download becomes available when this implementation is released.

## Capability and ownership profile

| Capability | Supported contract and limits |
| --- | --- |
| HTTP documents/fragments/commands | Ordinary generated dispatch; finite fragments update the acting tab; commands return empty SSE or ordinary 204. |
| Context/closures/resources | Persistent ARC; one admitted invocation per instance, 32 queued, 1,024 resources, 32 MiB live Roc allocations. A trap, cancellation or timeout discards the generation. Unsettled continuations prevent replacement. |
| Env/logging/time | Scalar environment strings, UTF-8 names, bounded log lines, Unix time and timers up to 30 seconds. |
| Object SQLite | Explicit path aliases select one object database. Default open options only. Named parameters; one SELECT/INSERT/UPDATE/DELETE/CREATE/DROP/ALTER statement. No ATTACH, raw transaction SQL or multiple databases. |
| SQL values | Exact full-range I64 through text casts; REAL, NULL, UTF-8 and owned BLOB values remain distinct. Materialized results: 10,000 rows and 1 MiB. Unique column names required. Use scalar `Sqlite.Blob`; native compiler limits on mixed Blob/nullable records still apply. Metadata is checked at execution, not native prepare time. Default timeout maps to provider execution limits; custom query timeouts fail. |
| Transactions | Deferred/Immediate callback rendezvous; commit waits for callback completion; explicit rollback and final-reference drop roll back. No nesting or Exclusive mode. Body/fetch/timer effects inside an open transaction fail before starting. |
| Inbound body | Request-scoped `read_all!`, at most 1 MiB; retained handles are invalid after the response. Incremental reads/file sinks are unsupported. |
| Outbound HTTP | `Http.send_with!`, explicit limit at most 256 KiB and timeout at most 30 seconds. Native 8 MiB defaults are rejected. Redirects are returned, never automatically followed. Typed timeout/cancellation/transport/size errors. |
| SSE | At most 128 Roc sources; one buffered event at most 1 MiB; 30-second stalled reader deadline. Cancel drops the closure. Polling remains the compatibility strategy. Declared live GET actions receive generated retry-always unless overridden; POST commands are never automatically retried. |
| Native OS/server facilities | Processes, arbitrary files, sockets, streaming file responses, native file mounts and shutdown guarantees are unsupported. Reachable missing hosted symbols fail linking. Provider owns the listener/lifecycle; native Server config does not tune provider sockets, pools or deadlines. |

A response and each published SSE event await `storage.sync()`. This does not
promise exactly once delivery, retry-safe external effects, cross-object
transactions, or a shutdown callback on eviction. Durable state belongs in SQL;
context must be reconstructible. A disconnected browser reconnects for a fresh
snapshot. No client domain store is added.

## Plain Roc and provider extensions

A plain `main.roc` can implement the native `Context`/`program` contract with
`pf: platform "platform/main.roc"`, independently of Rocci or Rocdown. See
[examples](examples/) for stateless HTTP, bounded fetch and SQL programs.
Run `rocci worker init examples/hello --stateless`, then build normally.
The default scaffold uses an object; select `--stateless` for SQL-free programs.
Build-time import inspection rejects SQL reachable in a stateless profile.

For service bindings or keyed identity, set `worker.custom_entry = "worker.mjs"`
and Wrangler `main = "worker.mjs"`. Import `.rocci/worker/app.wasm` and the
maintained `.rocci/worker/runtime/cloudflare.mjs`. Its accompanying
`cloudflare.d.mts` defines `WorkerProfile`, `WorkerObject`, `objectWorker` and
`statelessWorker`. A profile may supply `outboundFetch: request =>
env.SERVICE.fetch(request)`; the usual Roc HTTP types and byte/time/transaction
limits still apply. Provider types stay in the adapter. A custom entry owns
explicit authenticated identity selection; never derive tenancy from an
untrusted header. This escape hatch is opt-in; ordinary apps need no JS host.

## Conformance

```sh
npm ci --prefix crates/rocci-worker-platform/tooling
node --test crates/rocci-worker-platform/tests/*.test.mjs
ROCCI_WORKER_ROC=/path/to/pinned/roc node crates/rocci-worker-platform/tools/applications.mjs
ROCCI_WORKER_ROC=/path/to/pinned/roc node crates/rocci-worker-platform/tools/profiles.mjs
```

The application suite compiles the existing standalone counter and live-counter
examples, the plain Roc SQL example and a focused async regression program,
then runs those exact modules on local workerd. The examples are the single
source for application behavior. Outputs, hashes and logs are under `target/worker-*`.
The opt-in `tools/hosted.mjs` lane requires explicit test credentials and
creates and deletes only disposable guarded test resources. It never runs
in the offline suite. CI retains the verified application graph
and conformance results. Broader providers, whole-isolate cancellation stress,
idle residency economics and broader provider profiles require independent
evidence before promoting this preview.

The scaffold watches existing root source files and authored directories,
including an assets directory. Add newly created root source files to
Wrangler `build.watch_dir` or restart after a manual build; files inside
watched directories are discovered automatically. Keep generated directories
and provider logs outside that watch list.

The release also exports a content-addressed standalone Roc platform bundle
and `rocci-worker-runtime.tar.gz`, identified by `rocci-worker-platform.json`.
Plain Roc authors can use that platform URL directly, build with the qualified
`roc build --target=wasm32 --specialize=yes --opt=speed --wasm-stack-size=1048576`
and install the runtime tarball with npm. A minimal provider entry imports the
Wasm module and `statelessWorker` from `@rocci/worker-runtime`. This route does
not load Rocci templates, its compiler frontend, or Rocdown. The CLI remains
the automatic build/scaffold path for either ordinary Roc or Rocci sources.
Maintainers create the independent archives with `tools/package.mjs`; it reuses
the same platform staging implementation.

Finite responses are bounded to 1 MiB. Request targets are bounded to 8 KiB,
headers to 32 KiB and 100 fields. Smaller native Server body/header/target/SSE
limits are honored; incompatible larger limits fail initialization. Provider
listener and pool settings retain the deployment semantics described above.

The preview validates the default Wrangler environment. Named `env` blocks
are rejected until environment-specific binding/migration selection is
qualified; separate application configurations can select deployment profiles.

The lock retains compiler fingerprints per development host, so macOS and
Linux builds do not overwrite each other. Review one unlocked build on each
new host architecture before requiring `--locked` there. The compiler pin,
Wasm host and runtime remain common to the application.

Generated hosting attributes logs and traps to the authored init/route/live
declaration (`file:line`). Plain Roc falls back to its entry location. The
source-map artifact handles compile diagnostics; optimized Wasm instruction
addresses are not reconstructed as individual Roc expressions.
