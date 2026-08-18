# rocci-ui

Domain-neutral view records and presentation primitives for Rocci.

## Architecture and Boundary

`rocci-ui` lives in the base/shared tier of the Rocci ecosystem. It defines:
1. **Domain-Neutral View Records** (`rocci_ui::view`): Typed Rust data structures (`PageView`, `SiteView`, `LaneView`, `NavItemView`, `OutlineView`, `BreadcrumbView`, `ResourceView`) representing normalized layout inputs without any file-system, routing, graph, or review metadata dependencies.
2. **Deterministic HTML Helpers** (`rocci_ui::html`): Pure Rust utility functions like `escape` for environments that emit HTML directly.

## Dependency Rules

`rocci-ui` must have **zero dependencies** on `rocci-rocdown`, `okf`, or `rocci-okf`.
Consumers such as `rocci-rocdown` and `rocci-okf` may consume `rocci-ui` for structural presentation while keeping all domain logic (routing, catalog resolution, concept graphs, backlinks, review workflows) within their own packages.

