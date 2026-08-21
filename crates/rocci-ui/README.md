# rocci-ui

Domain-neutral view records and presentation primitives for Rocci.

## Architecture and Boundary

`rocci-ui` lives in the base/shared tier of the Rocci ecosystem. It defines:
1. **Domain-Neutral View Records** (`rocci_ui::view`): Typed Rust data structures (`PageView`, `SiteView`, `LaneView`, `NavGroupView`, `NavItemView`, `OutlineView`, `BreadcrumbView`, `ResourceView`) representing normalized layout inputs without any file-system, routing, graph, or review metadata dependencies.
2. **Deterministic HTML Helpers** (`rocci_ui::html`): Pure Rust utility functions like `escape` for environments that emit HTML directly.
3. **Shared Base Chrome Templates** (`templates/chrome/*.rocci`): Shared `.rocci` markup components for domain-neutral chrome (`PageOutline.rocci`, `NavList.rocci`, `Breadcrumbs.rocci`). Product shells (`SiteShell`, `RocdownTheme`) remain product-owned. Markdown rendering, catalog data, and OKF governance remain outside `rocci-ui`.
4. **Shared client chrome** (`assets/toc.js`, `assets/goto.js`): small page-owned scripts. `goto.js` is the Cmd/Ctrl-K fuzzy page palette and same-origin HTML swap used by Rocdown, rocci.dev, OKF review, and desktop preview.

## Dependency Rules

`rocci-ui` must have **zero dependencies** on `rocci-rocdown`, `okf`, or `rocci-okf`.
Consumers such as `rocci-rocdown` and `rocci-okf` may consume `rocci-ui` for structural presentation while keeping all domain logic (routing, catalog resolution, concept graphs, backlinks, review workflows) within their own packages.


