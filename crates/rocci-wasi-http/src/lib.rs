//! WASI 0.3 `wasi:http/service` adapter for basic-webserver 0.16 Roc apps.
//!
//! Phase 0 ships a probe that measures whether concurrent `handle` waits overlap
//! for adapter-await, CPU-only C, and hosted-sleep C. Later phases map real
//! WASI requests onto the 0.16 Roc ABI.

pub mod probe;

pub use probe::{
    OverlapReport, ProbeMode, ProbeRequest, ProbeResponse, handle_probe, overlap_native,
    overlap_wasmtime,
};
