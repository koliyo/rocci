# rocci-datastar

Datastar specification metadata, typed Server-Sent Events (SSE) framing, request signals, and asset management for Rocci.

## Overview

`rocci-datastar` provides domain-neutral Datastar protocol and tooling definitions across the Rocci ecosystem:

1. **Spec & Tooling Schema (`rocci_datastar::spec`)**: Directives (`data-bind`, `data-on`, `data-signals`, etc.), event modifiers (`__debounce.500ms`, `__throttle`, `__window`), and actions (`@get`, `@post`). Used by language servers and template linters for completions and diagnostics.
2. **Wire Protocol & SSE Framing (`rocci_datastar::sse`)**: Strongly typed SSE event builders for `datastar-patch-elements`, `datastar-patch-signals`, `datastar-remove-fragments`, and `datastar-execute-script`.
3. **Signals Extractor (`rocci_datastar::signals`)**: Deserialization helpers for reading client signals from query parameters (`?datastar=...`) and request bodies.
4. **Asset Management (`rocci_datastar::assets`)**: Version resolution, CDN downloads (jsDelivr / GitHub), integrity hashing, caching, and staging.
5. **Roc Runtime Generator (`rocci_datastar::codegen`)**: Standard typed Roc runtime module definitions (`Datastar.roc`).

## Feature Flags

* `fetch` (enabled by default): Enables `ureq` and `sha2` for downloading and verifying Datastar JS bundles from CDNs. Disable for pure metadata/protocol builds (e.g., in language servers or lightweight tools).
