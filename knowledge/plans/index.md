# Plans

* [Hybrid Rocdown islands for CDN-static sites](hybrid-rocdown-islands.md) - Phased delivery of CDN-static HTML with dynamic Rocci components backed by a rocci/rocdown island service. Article widgets out of scope. Exploratory; not shipped.
* [Generalized Rocdown block model](generalized-rocdown-block-model.md) - Phased delivery of uniform article `BlockCall` nodes, `:name[params]` spelling, a closed builtin registry, and per-kind Rocci renderers. Exploratory; not shipped.
* [Full Rocci and Rocdown language tooling](language-server.md) - Proposed region-aware editor tooling with shared token spans and product-owned server composition under the boundary refactor.
* [Public-preview branding and community](public-preview-community.md) - Reversible launch gate, Roc and Datastar feedback sequence, and evidence-based naming and identity decisions.
* [rocci.dev site architecture and Rocdown evolution](rocci-dev-site.md) - Proposed site structure, Rocdown/Rocci authoring split, named layouts, collections, and the decision boundary for a possible `rocci-site` profile.
* [Standalone Rocci OKF review and query application](rocci-okf-app.md) - Portable `okf` engine extraction and a `rocci-okf` application for evidence review and authenticated retrieval.
* [Rocdown product-boundary refactor](rocdown-boundary-refactor.md) - Phased consolidation of the Rocdown format and static generator, removal from base Rocci, Rocs retirement, and OKF separation.
* [CLI entry points for Rocci, Rocdown, and OKF preview](cli-entry-points.md) - Keep the three product CLIs, reject a plugin host, and make `rocci-okf run` the file-aware OKF viewer.
* [First-party Rocci chrome library and generation host](rocci-component-generation.md) - Extract demonstrated outline/nav/breadcrumb chrome into base Rocci, host Roc through a cached native subprocess and Wasmtime, and persist both generated Roc and compiled artifacts.

Other implementation-plan concepts have not yet been migrated. Priority-2 plan migration remains scheduled after the Phase 2 priority-1 corpus.
