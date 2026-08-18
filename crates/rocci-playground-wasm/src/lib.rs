use rocci_playground::{CompileRequest, PROTOCOL_VERSION, compile};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaygroundInitMetadata {
    pub protocol_version: u32,
    pub version: String,
    pub languages: Vec<String>,
}

#[wasm_bindgen]
pub fn init_playground() -> String {
    let meta = PlaygroundInitMetadata {
        protocol_version: PROTOCOL_VERSION,
        version: env!("CARGO_PKG_VERSION").to_string(),
        languages: vec!["rocci".to_string(), "rocdown".to_string()],
    };
    serde_json::to_string(&meta).unwrap_or_else(|_| "{}".to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaygroundErrorResponse {
    pub protocol_version: u32,
    pub error: String,
    pub has_errors: bool,
}

#[wasm_bindgen]
pub fn compile_json(request_json: &str) -> String {
    let req: Result<CompileRequest, _> = serde_json::from_str(request_json);
    match req {
        Ok(request) => {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| compile(&request)));
            match res {
                Ok(response) => serde_json::to_string(&response).unwrap_or_else(|e| {
                    serde_json::to_string(&PlaygroundErrorResponse {
                        protocol_version: PROTOCOL_VERSION,
                        error: format!("serialization error: {e}"),
                        has_errors: true,
                    })
                    .unwrap_or_default()
                }),
                Err(panic_payload) => {
                    let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown compiler panic".to_string()
                    };
                    serde_json::to_string(&PlaygroundErrorResponse {
                        protocol_version: PROTOCOL_VERSION,
                        error: format!("panic: {msg}"),
                        has_errors: true,
                    })
                    .unwrap_or_default()
                }
            }
        }
        Err(e) => serde_json::to_string(&PlaygroundErrorResponse {
            protocol_version: PROTOCOL_VERSION,
            error: format!("invalid JSON request: {e}"),
            has_errors: true,
        })
        .unwrap_or_default(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn playground_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn playground_dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, len, len);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn playground_compile_raw(ptr: *const u8, len: usize) -> u64 {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let json_req = std::str::from_utf8(slice).unwrap_or("{}");
    let json_resp = compile_json(json_req);
    let bytes = json_resp.into_bytes();
    let out_len = bytes.len();
    let out_ptr = playground_alloc(out_len);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_ptr, out_len);
    }
    ((out_ptr as u64) << 32) | (out_len as u64)
}
