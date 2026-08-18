/**
 * JSON-serializable Playground Protocol TypeScript Definitions.
 * Aligned with Rust crate `rocci-playground`.
 */

export const PROTOCOL_VERSION = 1;
export const HTML_UNAVAILABLE_REASON =
  "HTML preview is not available yet. Rocci can parse and lower this file in Rust/WASM, but rendering the generated Roc also requires a Roc runtime in WebAssembly.";

export type Language = "rocci" | "rocdown";
export type DiagnosticSeverity = "error" | "warning" | "info" | "hint";

export interface VirtualFile {
  path: string;
  content: string;
}

export interface VirtualWorkspace {
  files: VirtualFile[];
}

export interface CompileRequest {
  protocol_version?: number;
  revision: number;
  filename: string;
  language?: Language;
  source: string;
  workspace?: VirtualWorkspace;
}

export interface PlaygroundDiagnostic {
  severity: DiagnosticSeverity;
  message: string;
  start_byte: number;
  end_byte: number;
  from: number;
  to: number;
}

export interface PlaygroundHighlightSpan {
  from: number;
  to: number;
  kind: string;
  modifiers?: number;
}

export interface PlaygroundHighlights {
  source: PlaygroundHighlightSpan[];
  roc: PlaygroundHighlightSpan[];
  ast: PlaygroundHighlightSpan[];
}

export interface Capability {
  available: boolean;
}

export interface HtmlCapability {
  available: boolean;
  reason: string;
}

export interface PlaygroundCapabilities {
  roc: Capability;
  ast: Capability;
  html: HtmlCapability;
}

export interface CompileResponse {
  protocol_version: number;
  revision: number;
  language: Language;
  roc: string;
  ast: string;
  diagnostics: PlaygroundDiagnostic[];
  highlights: PlaygroundHighlights;
  capabilities: PlaygroundCapabilities;
  has_errors: boolean;
  error?: string;
}

export interface PlaygroundBootstrapDocument {
  id: string;
  filename: string;
  language: Language;
  source: string;
}

export interface PlaygroundBootstrap {
  protocol_version: number;
  documents: PlaygroundBootstrapDocument[];
  selected_document: string;
  compiler_wasm_url: string;
  worker_url: string;
  html_runtime: HtmlCapability;
}

// Worker message protocol

export type WorkerRequest =
  | { type: "init"; wasmUrl: string }
  | { type: "compile"; request: CompileRequest };

export type WorkerResponse =
  | { type: "init_ok"; metadata: Record<string, unknown> }
  | { type: "init_error"; error: string }
  | { type: "compile_ok"; response: CompileResponse; durationMs: number }
  | { type: "compile_error"; revision: number; error: string };
