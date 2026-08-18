//! Target-neutral playground compiler facade and protocol types for Rocci and Rocdown.

pub mod compiler;
pub mod protocol;
pub mod utf16;

pub use compiler::compile;
pub use protocol::{
    Capability, CompileRequest, CompileResponse, DiagnosticSeverity, HTML_NO_TARGET_REASON,
    HTML_UNAVAILABLE_REASON, HtmlCapability, Language, PROTOCOL_VERSION, PlaygroundBootstrap,
    PlaygroundBootstrapDocument, PlaygroundCapabilities, PlaygroundDiagnostic,
    PlaygroundHighlightSpan, PlaygroundHighlights, PlaygroundMode, VirtualFile, VirtualWorkspace,
};
pub use utf16::{byte_range_to_utf16, byte_to_utf16_offset};
