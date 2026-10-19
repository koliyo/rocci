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

## Worker deployment profile

`Config::from_toml_worker` validates a strict `[worker]` profile independently
of desktop windows and native listen addresses. `provider = "cloudflare"` and
`execution = "stateless" | "durable-object"` are required. Object execution
requires stable `binding` and `object_key`; optional `[worker.sqlite]` maps
`path` and `aliases` to `storage = "object"`. Unmapped paths and `:memory:` fail.
Stateless execution rejects object/storage identity fields. Optional
`custom_entry` names an authored relative provider entry under the app root.
Native `Config::from_toml` retains its existing validation. Worker assets must
remain within the application root.
