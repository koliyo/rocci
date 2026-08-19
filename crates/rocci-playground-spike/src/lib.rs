use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpikeDiagnostic {
    pub severity: String,
    pub message: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpikeCompileResult {
    pub language: String,
    pub roc: String,
    pub ast: String,
    pub diagnostics: Vec<SpikeDiagnostic>,
    pub has_errors: bool,
}

pub fn compile_rocci(source: &str) -> SpikeCompileResult {
    let source_file = rocci_template::SourceFile::new("Spike.rocci", source);
    let output = rocci_template::compile(source_file, &rocci_template::LowerOptions::default());
    let ast = rocci_template::format_ast(source, &output.document);
    let has_errors = output.has_errors();
    let spike_diagnostics = output
        .diagnostics
        .into_iter()
        .map(|d| SpikeDiagnostic {
            severity: format!("{:?}", d.severity).to_lowercase(),
            message: d.message,
            start: d.span.start as usize,
            end: d.span.end as usize,
        })
        .collect();

    SpikeCompileResult {
        language: "rocci".to_string(),
        roc: output.roc,
        ast,
        diagnostics: spike_diagnostics,
        has_errors,
    }
}

pub fn compile_rocdown(source: &str) -> SpikeCompileResult {
    let source_file = rocci_rocdown::SourceFile::new("Spike.rocdown", source);
    let options = rocci_rocdown::CompileOptions {
        resolve_links: false,
        resolve_includes: false,
        check_assets: false,
        ..rocci_rocdown::CompileOptions::default()
    };
    let output = rocci_rocdown::compile(source_file, &options);
    let ast = rocci_rocdown::format_ast(source, &output.document);
    let has_errors = output.has_errors();
    let spike_diagnostics = output
        .diagnostics
        .into_iter()
        .map(|d| SpikeDiagnostic {
            severity: format!("{:?}", d.severity).to_lowercase(),
            message: d.message,
            start: d.span.start as usize,
            end: d.span.end as usize,
        })
        .collect();

    SpikeCompileResult {
        language: "rocdown".to_string(),
        roc: output.roc,
        ast,
        diagnostics: spike_diagnostics,
        has_errors,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spike_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// # Safety
/// `ptr` must be null or come from `spike_alloc` with the same `len`, and no
/// other alias may use the allocation after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spike_dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, len, len);
        }
    }
}

unsafe fn run_raw_compile(
    ptr: *const u8,
    len: usize,
    compiler: fn(&str) -> SpikeCompileResult,
) -> u64 {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let source = std::str::from_utf8(slice).unwrap_or("");
    let result = compiler(source);
    let json = serde_json::to_vec(&result).unwrap_or_default();
    let out_len = json.len();
    let out_ptr = spike_alloc(out_len);
    unsafe {
        std::ptr::copy_nonoverlapping(json.as_ptr(), out_ptr, out_len);
    }
    ((out_ptr as u64) << 32) | (out_len as u64)
}

/// # Safety
/// `ptr` must point to `len` readable bytes that stay valid for the duration
/// of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn compile_rocci_raw(ptr: *const u8, len: usize) -> u64 {
    unsafe { run_raw_compile(ptr, len, compile_rocci) }
}

/// # Safety
/// `ptr` must point to `len` readable bytes that stay valid for the duration
/// of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn compile_rocdown_raw(ptr: *const u8, len: usize) -> u64 {
    unsafe { run_raw_compile(ptr, len, compile_rocdown) }
}

#[wasm_bindgen]
pub fn compile_rocci_wasm(source: &str) -> String {
    let result = compile_rocci(source);
    serde_json::to_string(&result).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

#[wasm_bindgen]
pub fn compile_rocdown_wasm(source: &str) -> String {
    let result = compile_rocdown(source);
    serde_json::to_string(&result).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_rocci_spike() {
        let source = "@component Counter = |{ count }| { <button>{count}</button> }";
        let res = compile_rocci(source);
        assert_eq!(res.language, "rocci");
        assert!(!res.has_errors, "diagnostics: {:?}", res.diagnostics);
        assert!(
            res.roc.contains("counter = |{ count }|"),
            "roc: {}",
            res.roc
        );
        assert!(res.ast.contains("component Counter"), "ast: {}", res.ast);
    }

    #[test]
    fn test_compile_rocdown_spike() {
        let source = "# Welcome\n\nThis is **Rocdown**.";
        let res = compile_rocdown(source);
        assert_eq!(res.language, "rocdown");
        assert!(!res.has_errors, "diagnostics: {:?}", res.diagnostics);
        assert!(res.roc.contains("Welcome"), "roc: {}", res.roc);
        assert!(
            res.ast.contains("(block h1 line id welcome"),
            "ast: {}",
            res.ast
        );
    }
}
