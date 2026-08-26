# rocci-core

Shared configuration models and runtime contracts for the Rocci ecosystem.

## Responsibilities

- **Configuration schema (`rocci.toml`)**:
  - `[window]`: Title, width, height, resizability, devtools settings.
  - `[http]`: Host, port, redirect trailing slash behavior.
  - `[security]`: Content Security Policy (CSP), allowed origins, script evaluation policy.
  - `[assets]`: Pinned Datastar JS version and asset discovery paths.
  - `[bundle]`: App name, identifier, entry module, and output configuration for desktop packaging.
  - `[dev]`: Live reload, watch paths, and debug flags.
- **Validation**: Strict deserialization and constraint checks preventing unsafe runtime options and invalid network/file configurations.
- **Zero reverse dependencies**: Pure data contracts with no dependencies on `rocci-rocdown`, `okf`, or GUI backends.
