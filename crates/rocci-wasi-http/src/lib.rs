//! WASI 0.3 `wasi:http/service` adapter for basic-webserver 0.16 Roc apps.
//!
//! Yield around Roc: buffer the request, call `respond`, map an ordinary body
//! or SSE `stream<u8>`. Wait is adapter clocks. Nested sqlite/file inside
//! `respond!` serializes other `handle`s.

pub mod abi;
pub mod files;
pub mod guest;
pub mod handle;
#[cfg(not(target_family = "wasm"))]
pub mod linked;
#[cfg(feature = "embedder")]
pub mod probe;
#[cfg(target_family = "wasm")]
pub mod roc_object;
#[cfg(feature = "embedder")]
pub mod roc_wasm;
#[cfg(feature = "embedder")]
pub mod sqlite;

pub use abi::{
    IncomingRequest, OrdinaryResponse, OutcomeToHost, OutgoingResponse, ServerRequest,
    SseStepToHost,
};
pub use guest::{EchoGuest, EmptySseGuest, RocGuest, StubGuest, WaitEmitGuest};
pub use handle::Adapter;
#[cfg(not(target_family = "wasm"))]
pub use linked::LinkedHelloWebGuest;
#[cfg(feature = "embedder")]
pub use probe::{
    OverlapReport, ProbeMode, ProbeRequest, ProbeResponse, handle_probe, overlap_native,
    overlap_wasmtime,
};
#[cfg(target_family = "wasm")]
pub use roc_object::LinkedHelloWebGuest;
#[cfg(feature = "embedder")]
pub use roc_wasm::{WasmRocGuest, hello_web_component_bytes};
#[cfg(feature = "embedder")]
pub use sqlite::{SqliteGuest, SqliteStore};
