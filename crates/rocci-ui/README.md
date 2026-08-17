# rocci-ui

Domain-neutral presentation components, view records, and shell layout primitives for Rocci.

## Architecture and Boundary

`rocci-ui` lives in the base/shared tier of the Rocci ecosystem. It defines:
1. **Domain-Neutral View Records** (`rocci_ui::view`): Typed Rust data structures (`PageView`, `SiteView`, `LaneView`, `NavItemView`, `OutlineView`, `BreadcrumbView`, `StatCardView`, `BadgeView`, `AlertView`, `ResourceView`) representing normalized layout inputs without any file-system, routing, graph, or review metadata dependencies.
2. **Reusable Presentation Components** (`templates/RocciUi.rocci`): PascalCase `.rocci` components (`Breadcrumbs`, `Outline`, `Journey`, `StatCard`, `AlertBanner`) that compile down to standard Roc functions.
3. **Deterministic HTML Helpers** (`rocci_ui::html`): Pure Rust rendering functions for environments that emit HTML directly.
4. **Base Design Tokens & CSS** (`themes/base.css`): Styling variables, layout grids, responsive breakpoints, accessible focus outlines, and print styles.

## Dependency Rules

`rocci-ui` must have **zero dependencies** on `rocci-rocdown`, `okf`, or `rocci-okf`.
Both `rocci-rocdown` and `rocci-okf` may consume `rocci-ui` for structural presentation while keeping all domain logic (routing, catalog resolution, concept graphs, backlinks, review workflows) within their own packages.
