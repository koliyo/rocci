//! 0.16 `roc_*_for_host` symbols, statically linked (same names as `hello_web.wat`).

use std::sync::Mutex;

use crate::abi::{OrdinaryResponse, OutcomeToHost, ServerHeader, ServerRequest};
use crate::guest::RocGuest;

struct EmitState {
    status: u16,
    body: Vec<u8>,
}

static EMIT: Mutex<EmitState> = Mutex::new(EmitState {
    status: 500,
    body: Vec::new(),
});

const HELLO_WEB_HTML: &[u8] = b"<!doctype html><html><body>hello-web</body></html>";

/// # Safety
/// `ptr` must be null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hosted_emit_ordinary(status: i32, ptr: *const u8, len: i32) {
    let bytes = if ptr.is_null() || len <= 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len as usize) }.to_vec()
    };
    let mut emit = EMIT.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    emit.status = status.max(0) as u16;
    emit.body = bytes;
}

#[unsafe(no_mangle)]
pub extern "C" fn roc_init_for_host() -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn roc_respond_for_host() -> i32 {
    unsafe { hosted_emit_ordinary(200, HELLO_WEB_HTML.as_ptr(), HELLO_WEB_HTML.len() as i32) };
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn roc_shutdown_for_host() -> i32 {
    0
}

pub struct LinkedHelloWebGuest {
    initialized: bool,
}

impl LinkedHelloWebGuest {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Default for LinkedHelloWebGuest {
    fn default() -> Self {
        Self::new()
    }
}

impl RocGuest for LinkedHelloWebGuest {
    fn init(&mut self) {
        let code = roc_init_for_host();
        assert_eq!(code, 0, "roc_init_for_host");
        self.initialized = true;
    }

    fn respond(&mut self, _request: &ServerRequest) -> OutcomeToHost {
        let code = roc_respond_for_host();
        assert_eq!(code, 0, "roc_respond_for_host");
        let emit = EMIT.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        OutcomeToHost::Ordinary(OrdinaryResponse {
            exit_code: 0,
            body: emit.body.clone(),
            headers: vec![ServerHeader {
                name: "content-type".into(),
                value: "text/html; charset=utf-8".into(),
            }],
            status: emit.status,
            stop: false,
        })
    }

    fn shutdown(&mut self) {
        let _ = roc_shutdown_for_host();
        self.initialized = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_hello_web_uses_export_names() {
        let mut guest = LinkedHelloWebGuest::new();
        guest.init();
        let OutcomeToHost::Ordinary(out) = guest.respond(&ServerRequest {
            authority_host: String::new(),
            authority_port: 0,
            authority_port_present: false,
            authority_present: false,
            body: Vec::new(),
            body_limit_bytes: 4096,
            content_length: 0,
            content_length_known: true,
            headers: vec![],
            method: ServerRequest::METHOD_GET,
            method_ext: String::new(),
            target_authority_host: String::new(),
            target_authority_port: 0,
            target_authority_port_present: false,
            target_path: "/".into(),
            target_query: String::new(),
            target_query_present: false,
            target_tag: ServerRequest::TARGET_RESOURCE,
        }) else {
            panic!("ordinary");
        };
        assert_eq!(out.status, 200);
        assert_eq!(out.body, HELLO_WEB_HTML);
    }
}
