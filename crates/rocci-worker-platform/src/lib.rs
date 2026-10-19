//! Core-Wasm hosting is separate from the native TCP/platform implementation.

pub const ABI_VERSION: u32 = 1;
pub const COMPILER_VERSION: &str = "nightly-2026-10-09-258ab27";
pub mod sources;

#[cfg(target_arch = "wasm32")]
#[path = "../../rocci-platform/src/roc_platform_abi.rs"]
#[allow(unsafe_op_in_unsafe_fn, clippy::enum_variant_names)]
mod roc_platform_abi;

#[cfg(target_arch = "wasm32")]
mod host;
