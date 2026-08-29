//! WASI 0.3 `wasi:http/service` adapter for basic-webserver 0.16 Roc apps.
//!
//! Yield around Roc: buffer the request, call `respond`, map an ordinary body.
//! SSE Wait (later) is adapter clocks, not async Roc.

pub mod abi;
pub mod guest;
pub mod handle;
pub mod probe;
pub mod roc_wasm;

pub use abi::{IncomingRequest, OrdinaryResponse, OutcomeToHost, OutgoingResponse, ServerRequest};
pub use guest::{RocGuest, StubGuest};
pub use handle::Adapter;
pub use probe::{
    OverlapReport, ProbeMode, ProbeRequest, ProbeResponse, handle_probe, overlap_native,
    overlap_wasmtime,
};
pub use roc_wasm::WasmRocGuest;
