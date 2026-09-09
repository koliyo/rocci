---
type: Research Report
title: Expose parse and compile on rocci-platform via hosted glue
description: "Roc can call crates/rocci-template through rocci-platform hosted functions. Luke's rust platform template is the glue/build layout rocci-platform already uses. Parse and compile are Str APIs; render is not Html.Node. A second stdio host does not help apps that already pin rocci."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/tooling, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-09-09T19:40:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: glue-research
    resource: template-parser-roc-glue.md
    title: Earlier glue research recommended a stdio template-host first
    author: process:cursor
    last_modified: 2026-09-02
  - id: glue-plan
    resource: ../../plans/rocci/template-parser-roc-glue.md
    title: Hosted glue implementation plan
    author: process:cursor
    last_modified: 2026-09-02
  - id: as-platform
    resource: rocci-as-roc-platform.md
    title: Rocci should be a Roc platform
    author: process:cursor
    last_modified: 2026-09-03
  - id: postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: First-cut platform cutover
    author: process:cursor
    last_modified: 2026-09-03
  - id: native-pm
    resource: roc-native-template-compiler-postmortem.md
    title: Roc-native rewrite is a POC; Rust stays the product compiler
    author: process:cursor
    last_modified: 2026-09-03
  - id: native-research
    resource: roc-native-template-compiler.md
    title: Consume-in-Roc vision without the rocci CLI
    author: process:cursor
    last_modified: 2026-09-02
  - id: template
    resource: https://github.com/lukewilliamboswell/roc-platform-template-rust
    title: Roc platform template for Rust
    author: human:luke-boswell
    last_modified: 2026-09-09
  - id: platform-main
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: platform rocci requires, exposes, hosted, glue size note
    author: process:git
    last_modified: 2026-09-03
  - id: platform-readme
    resource: ../../../crates/rocci-platform/README.md
    title: Pin, build.sh, bundle.sh, roc glue regen
    author: process:git
    last_modified: 2026-09-09
  - id: host-roc
    resource: ../../../crates/rocci-platform/platform/Host.roc
    title: Hosted effect signatures; no Rocci parser today
    author: process:git
    last_modified: 2026-09-03
  - id: stdout-roc
    resource: ../../../crates/rocci-platform/platform/Stdout.roc
    title: Public module wrapping Host.stdout_*
    author: process:git
    last_modified: 2026-09-03
  - id: path-roc
    resource: ../../../crates/rocci-platform/platform/Path.roc
    title: Path.read_utf8! already exists
    author: process:git
    last_modified: 2026-09-03
  - id: file-rs
    resource: ../../../crates/rocci-platform/src/file.rs
    title: hosted_file_read_utf8 C ABI pattern
    author: process:git
    last_modified: 2026-09-03
  - id: abi-mod
    resource: ../../../crates/rocci-platform/src/abi/mod.rs
    title: Semantic aliases over numbered glue Try types
    author: process:git
    last_modified: 2026-09-03
  - id: glue-rs
    resource: ../../../crates/rocci-platform/src/roc_platform_abi.rs
    title: Generated hosted symbol signatures
    author: process:git
    last_modified: 2026-09-03
  - id: platform-lib
    resource: ../../../crates/rocci-platform/src/lib.rs
    title: Host crate; no rocci-template link
    author: process:git
    last_modified: 2026-09-03
  - id: platform-cargo
    resource: ../../../crates/rocci-platform/Cargo.toml
    title: Host dependencies; no rocci-template
    author: process:git
    last_modified: 2026-09-03
  - id: hello-web
    resource: ../../../crates/rocci-platform/examples/hello-web.roc
    title: Minimal app pin of in-tree platform
    author: process:git
    last_modified: 2026-09-03
  - id: html-roc
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: pf.Html including render_fragment
    author: process:git
    last_modified: 2026-09-03
  - id: template-lib
    resource: ../../../crates/rocci-template/src/lib.rs
    title: parse, lower, compile; crate does not invoke roc
    author: process:git
    last_modified: 2026-08-31
  - id: template-cargo
    resource: ../../../crates/rocci-template/Cargo.toml
    title: clap is a package dependency of the lib
    author: process:git
    last_modified: 2026-08-21
  - id: template-main
    resource: ../../../crates/rocci-template/src/main.rs
    title: clap is used only by the crate binary
    author: process:git
    last_modified: 2026-08-25
  - id: lower-opts
    resource: ../../../crates/rocci-template/src/lower/mod.rs
    title: LowerOptions default html_module Html; emit import Html
    author: process:git
    last_modified: 2026-09-09
  - id: pprint
    resource: ../../../crates/rocci-template/src/pprint.rs
    title: format_ast S-expression printer
    author: process:git
    last_modified: 2026-08-25
  - id: codes
    resource: ../../../crates/rocci-template/src/diagnostic.rs
    title: Diagnostic code, message, span
    author: process:git
    last_modified: 2026-08-31
  - id: source-file
    resource: ../../../crates/rocci-template/src/span.rs
    title: SourceFile name plus src
    author: process:git
    last_modified: 2026-08-17
  - id: roc-host
    resource: ../../../crates/rocci-roc-host/README.md
    title: Apply cache; native host pins basic-cli
    author: process:git
    last_modified: 2026-09-01
  - id: hosting-abi
    resource: roc-hosting-lazy-abi.md
    title: hosted_* is Roc calling the host over the C ABI
    author: process:cursor
    last_modified: 2026-08-30
  - id: gen-research
    resource: rocci-components-in-generation.md
    title: Glue embeds compiled Roc; prefer Str across the host boundary
    author: process:cursor
    last_modified: 2026-08-31
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Components lower to pure Html functions
    author: human:nils
    last_modified: 2026-08-31
  - id: workspace-deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: rocci-platform and rocci-template are both base-rocci
    author: process:git
    last_modified: 2026-09-03
---

# Expose parse and compile on rocci-platform via hosted glue

Exploratory. Not shipped. Placement update to
[hosted glue for the Rust template parser](template-parser-roc-glue.md).
Paired plan:
[hosted glue plan](/plans/rocci/template-parser-roc-glue.md).
The parser stays `crates/rocci-template`. Roc calls it through **this
repository's product platform**, not a new stdio host and not a Roc
package.[^glue-research][^glue-plan][^as-platform]

## For a later agent

- **Authority:** exploratory. Do not start phases unless asked.
- Keep this distinct from the [Roc-native rewrite](roc-native-template-compiler.md)
  (Rust stays the product compiler).[^native-pm][^native-research]
- `roc glue` regenerates `src/roc_platform_abi.rs`. It does **not** wrap
  `rocci-template` as a package.[^template][^platform-readme][^hosting-abi]
- **Parse is not render.** `compile!` returns generated Roc `Str`.
  Returning `pf.Html` from parse would interpret `{expr}` in Rust, which
  is a [pure-render](/decisions/pure-render-components.md) break.
  [^template-lib][^pure-render][^gen-research]
- A second stdio platform cannot be called from an app that already pins
  `rocci-platform` (one platform per app).[^postmortem][^as-platform]

## What already exists

`rocci-platform` is the shipped app pin. Generated `rocci run` / `rocci
build` apps and the custom gallery/snake examples use it.
`platform/main.roc` already has `requires` / `exposes` / `hosted` /
`targets`, `build.sh` writes native `libhost.a`, `bundle.sh` makes
`.tar.zst`, and the crate README documents the same `roc glue` command
as Luke's [Rust platform template](https://github.com/lukewilliamboswell/roc-platform-template-rust).
The template's product API is stdio `main!`; Rocci's is HTTP
`{ init!, respond!, shutdown! }`. Use the template for **layout and
glue regen**, not as the host to start from.[^platform-readme][^platform-main][^template][^as-platform][^hello-web]

Public Roc modules wrap `Host.*` effects (`Stdout.line!` calls
`Host.stdout_line!`). `Path.read_utf8!` is already hosted. There is no
`Rocci` / `Template` module and no `hosted_rocci_*` symbol. Datastar
and the compiler helpers on `Html` are **Roc-only**; they did not add
Rust hosted functions. Parse/compile would be the first
Rocci-original `hosted_*` on the vendored basic-webserver host.
[^stdout-roc][^path-roc][^host-roc][^html-roc][^postmortem][^platform-lib]

`rocci-template` already exposes `parse` / `compile` as a Rust library
and as a crate binary (`build` / `ast`). The library does not invoke
`roc`. Interpolations stay opaque Roc source. Default lowering emits
`import Html`, which matches `pf.Html` on this platform.
[^template-lib][^lower-opts][^html-roc]

`rocci-platform` does not depend on `rocci-template` today. Both crates
are `base-rocci`. The template crate must not depend on the platform.
[^platform-cargo][^workspace-deps]

## Two directions of glue

| Direction | What it is | Role here |
| --- | --- | --- |
| **Hosted** (`hosted_*`) | Roc calls the host. The host is Rust. | How Roc invokes `parse` / `compile`. |
| **`roc glue`** | Generates Rust ABI types so the host can exchange Roc values. | Required when `platform/main.roc` `hosted` / `provides` change. Overwrites `src/roc_platform_abi.rs`. |

Roc packages have no FFI. Only the platform can run Rust. Exposing the
parser therefore means new `hosted` functions **on a platform the app
pins**, implemented by linking `rocci-template` into that host.
[^hosting-abi][^glue-research][^template-lib]

Luke's template is the mechanical checklist rocci-platform already
followed: `platform/main.roc`, `roc glue …/RustGlue.roc ./src/
platform/main.roc`, `./build.sh` → `platform/targets/<triple>/libhost.a`,
`./bundle.sh` → `.tar.zst`. Adding parse/compile is another hosted
export on that path, not a new packaging invention.[^template][^platform-readme][^glue-rs]

`roc glue` mis-sizes aggregates that contain an unresolved type
variable (`Exit(I64)` workaround in `main.roc`). New result types must
be fully sized records and closed unions. After regen, add a semantic
alias in `src/abi/mod.rs` so the effect module does not chase numbered
`TryTypeN` names, which shift when modules are added.
[^platform-main][^abi-mod]

## Why the product platform, not a new stdio host

The earlier glue record recommended a **stdio template-host**
(`requires { main! }`) so a basic-cli-shaped app could consume templates
without pinning HTTP Rocci, and treated copying onto `rocci-platform`
as a follow-on.[^glue-research][^glue-plan]

That split is the right answer only if the first caller is a **CLI
that is not a Rocci app**. For "parse and render `.rocci` from Roc"
on the product runtime, a second platform does not help: an app that
pins `rocci-platform` cannot call another platform's `hosted_*`.
[^postmortem][^as-platform]

| Job | New stdio host | `rocci-platform` extras |
| --- | --- | --- |
| Call from a generated or custom Rocci app | No (wrong `pf`) | Yes |
| File IO for `*.rocci` paths | Must re-expose File/Path | `Path.read_utf8!` exists |
| Emitted Roc `import Html` | Needs an Html module on that host | Matches `pf.Html` |
| New workspace crate | Yes (`rocci-template-host`) | No |
| First product `hosted_*` on the vendored HTTP host | Avoided | Yes; next vendor snapshot must keep them |
| basic-cli / playground snapshot eval | Yes | No |

Recommend **`rocci-platform` first**. Keep a stdio host as a follow-on
only if a non-HTTP Roc tool needs the same ABI. Do not invent a second
HTTP engine.[^as-platform][^path-roc][^html-roc][^hello-web]

## Three payloads (do not mix them)

`compile` does not type-check interpolations and does not run `roc`.
A hosted entry cannot return a finished `Html.Node` from parse alone.
[^template-lib][^pure-render][^gen-research]

| Payload | Host does | Roc sees | First cut? |
| --- | --- | --- | --- |
| **Parse** | `parse` + `format_ast` | `{ ast : Str, diagnostics : List(Diagnostic) }` | Yes, after compile |
| **Compile** | `compile` with default `LowerOptions` | `{ roc : Str, diagnostics : List(Diagnostic) }` | Yes, first hosted proof |
| **Apply / render** | `compile` + `roc` via rocci-roc-host | HTML `Str` | Follow-on |

`compile` in Rust already returns a `CompileOutput` even when
diagnostics contain errors (`has_errors`). Prefer a **record**, not
`Try`, so the ABI stays a closed struct and the caller inspects
`diagnostics`. Use `RCxxxx` in `code` when present; empty `Str`
otherwise. Spans are UTF-16 offsets in `SourceFile` by default; expose
them as `U64` `start` / `end` and do not claim they are line/column.
[^template-lib][^codes][^source-file][^pprint]

Apply already exists **from Rust** (Rocdown's native host: `roc build`
plus an apply binary, cached). Glue would let a Roc `respond!` ask for
that path. The nested compile is **seconds** on a cache miss, blocks
the hosted C-ABI (CPU occupancy on the Tokio worker), and the native
apply host pins **basic-cli**, not rocci-platform. That is a different
product from parse/compile. Do not ship apply as the first hosted API.
Do not return `pf.Html`: constructing `Html.Node` in Rust would either
interpret interpolations or smuggle a pre-rendered string through
`dangerously_include_unescaped_html`. Prefer HTML `Str` if apply lands
later.[^roc-host][^html-roc][^gen-research][^hosting-abi]

File-path helpers do not need extra hosted functions:

```roc
compile_file! : Path => Try(CompileResult, [PathErr(IOErr), ..])
compile_file! = |path|
    source = Path.read_utf8!(path)?
    Ok(compile!({ name: Path.display(path), source: source }))
```

`Path.read_utf8!` already exists. `compile!` itself should take
`{ name : Str, source : Str }` because `SourceFile` wants a name for
diagnostics.[^path-roc][^source-file]

## How to add it (mechanics)

Follow the Stdout / `hosted_file_read_utf8` pattern, not a new crate.

1. **Stop pulling clap into `libhost.a`.** `clap` lives in
   `rocci-template` `[dependencies]` but is used only by `src/main.rs`.
   Feature-gate it (`cli` on the binary) **before** the platform crate
   depends on `rocci_template`. The parser itself is a small Rust lib
   next to Hyper/SQLite; clap is the avoidable bloat.
   [^template-cargo][^template-main][^platform-cargo]
2. Add `rocci-template` to `crates/rocci-platform/Cargo.toml`. Do not
   add a workspace member. Both crates are already `base-rocci`.
   [^workspace-deps][^platform-cargo]
3. In `Host.roc`, declare fully sized hosted functions, for example:

   ```roc
   Diagnostic : { code : Str, message : Str, start : U64, end : U64 }
   Input : { name : Str, source : Str }
   CompileResult : { roc : Str, diagnostics : List(Diagnostic) }
   ParseResult : { ast : Str, diagnostics : List(Diagnostic) }

   rocci_compile! : Input => CompileResult
   rocci_parse! : Input => ParseResult
   ```

4. Map them in `platform/main.roc` `hosted` (`"hosted_rocci_compile":
   Host.rocci_compile!`, …) and add `Rocci` to `exposes`.
   [^platform-main][^host-roc]
5. Regenerate glue:

   ```text
   roc glue /path/to/roc/crates/compiler/glue/src/RustGlue.roc ./src/ platform/main.roc
   ```

   Alias the new generated Try/record types in `src/abi/mod.rs`.
   Implement `#[no_mangle] pub extern "C" fn hosted_rocci_compile`
   next to `file.rs`: take `RocStr`s, call
   `rocci_template::compile(SourceFile::new(name, src),
   &LowerOptions::default())`, write `RocStr` / `RocList` results,
   release owned arguments per the generated glue comments.
   [^platform-readme][^file-rs][^glue-rs][^abi-mod][^template-lib][^lower-opts]
6. Add `platform/Rocci.roc` that wraps `Host.rocci_*` the way
   `Stdout.roc` wraps stdio. File helpers use `Path.read_utf8!`.
   [^stdout-roc][^path-roc]
7. Rebuild `libhost.a` with `crates/rocci-platform/build.sh`. Prove
   with a sibling of `examples/hello-web.roc`: `respond!` compiles an
   inline `@component` fixture and returns the generated Roc as
   `text/plain`. A known-bad fixture returns diagnostics, not a panic.
   [^hello-web][^platform-readme]

A later vendor snapshot of basic-webserver must **keep**
`hosted_rocci_*`, `platform/Rocci.roc`, and the `rocci-template`
Cargo dep. Those are Rocci-original, like `Datastar.roc`.
[^postmortem][^platform-readme]

## Roc surface (sketch)

```roc
import pf.Rocci
import pf.Path

result = Rocci.compile!({ name: "Hello.rocci", source: fixture })
# result.roc starts with `import Html`
# result.diagnostics is empty on success

tree = Rocci.parse!({ name: "Hello.rocci", source: fixture })
# tree.ast is a format_ast S-expression
```

Do not add a typed Roc AST matching the ungram in v1. Do not add
handler/`@init` apply. Do not claim this replaces `rocci run`.
[^pprint][^glue-research][^native-research]

## Recommendation

1. Treat the API as **hosted effects on `rocci-platform`**, exposed as
   `pf.Rocci`.
2. Feature-gate `clap` on `rocci-template` first.
3. First hosted function: **compile!** (record of Roc `Str` plus
   diagnostics). Then **parse!** as `format_ast`.
4. File-path wrappers in Roc via `Path.read_utf8!`.
5. Leave **apply!** / HTML render as a follow-on that uses
   rocci-roc-host and returns `Str`. Do not interpret `{expr}`. Do
   not start the native-compiler phases from this record.
6. Do not add `crates/rocci-template-host` unless a non-HTTP caller
   appears.

Implementation: retarget
[hosted glue for the Rust template parser](/plans/rocci/template-parser-roc-glue.md).
[^glue-plan]

[^glue-research]: Stdio template-host was the v1 placement; copy onto rocci-platform was a follow-on.
[^glue-plan]: Phased Bound originally named `crates/rocci-template-host`.
[^as-platform]: Domain platform owns hosted I/O; one app pin.
[^postmortem]: First-cut payoff is `pf` ownership; one platform per app.
[^native-pm]: POC only; Rust stays the product compiler.
[^native-research]: Vision is consume-in-Roc without the CLI; rewrite is not this path.
[^template]: Template: `main.roc`, `roc glue`, `build.sh`, `bundle.sh`.
[^platform-main]: `hosted { }` list; `Exit(I64)` glue size workaround; no `hosted_rocci_*`.
[^platform-readme]: Glue regen command; `build.sh` writes `libhost.a`.
[^host-roc]: Hosted signatures today are file/env/sqlite/http/stdio/tcp.
[^stdout-roc]: Public modules wrap `Host.*` and map errors.
[^path-roc]: `read_utf8!` is already a hosted file read.
[^file-rs]: `extern "C"` hosted functions take glue types, copy into Rust, return `RocStr`.
[^abi-mod]: Numbered glue names shift; alias by semantic host name.
[^glue-rs]: Generated file header: hosted argument ownership and exact C signatures.
[^platform-lib]: Staticlib entry; modules implement hosted symbols declared in `main.roc`.
[^platform-cargo]: No `rocci-template` dependency.
[^hello-web]: In-tree example pins `../platform/main.roc` and implements `respond!`.
[^html-roc]: `Html` is the runtime module compiled output imports.
[^template-lib]: `parse` / `compile`; no `roc`; interpolations not evaluated.
[^template-cargo]: `clap` is a package `[dependencies]` entry.
[^template-main]: Only the binary imports clap.
[^lower-opts]: Default `html_module` is `"Html"`; emitter prepends `import Html`.
[^pprint]: S-expression printer over `Document`.
[^codes]: `code` is `Option<&'static str>`; spans ride on `Diagnostic`.
[^source-file]: `SourceFile { name, src }`; default position encoding UTF-16.
[^roc-host]: Native apply is `roc build` against basic-cli plus a cached binary.
[^hosting-abi]: Roc → host is `hosted_*` C ABI; blocking from `respond!`.
[^gen-research]: No compiler-as-library; prefer `Str` across the host boundary.
[^pure-render]: `@component` is a pure function to Html, not a runtime interpreter.
[^workspace-deps]: `rocci-platform` and `rocci-template` are both `BASE_ROCCI`.
