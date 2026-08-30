//! Call the linked `roc_*_for_host` object (wasm32 C ABI).

use std::ffi::c_void;

use crate::abi::{OrdinaryResponse, OutcomeToHost, ServerHeader, ServerRequest};
use crate::guest::RocGuest;

const INIT_SIZE: usize = 224;
const INIT_TAG: usize = 216;
const INIT_CONTEXT: usize = 212;
const OUTCOME_SIZE: usize = 48;
const OUTCOME_TAG: usize = 44;
const ORDINARY: u8 = 1;
const BODY_PTR: usize = 8;
const BODY_LEN: usize = 12;
const STATUS: usize = 32;
const REQUEST_SIZE: usize = 104;

#[unsafe(no_mangle)]
pub extern "C" fn roc_alloc(size: usize, alignment: usize) -> *mut u8 {
    let align = alignment.max(1);
    let layout = std::alloc::Layout::from_size_align(size.max(1), align)
        .unwrap_or_else(|_| std::alloc::Layout::from_size_align(1, 1).unwrap());
    unsafe { std::alloc::alloc(layout) }
}

#[unsafe(no_mangle)]
pub extern "C" fn roc_realloc(ptr: *mut u8, new_size: usize, alignment: usize) -> *mut u8 {
    if ptr.is_null() {
        return roc_alloc(new_size, alignment);
    }
    let next = roc_alloc(new_size, alignment);
    if !next.is_null() && new_size > 0 {
        unsafe { std::ptr::copy_nonoverlapping(ptr, next, new_size) };
    }
    next
}

#[unsafe(no_mangle)]
pub extern "C" fn roc_dealloc(ptr: *mut u8, _alignment: usize) {
    let _ = ptr;
}

#[unsafe(no_mangle)]
pub extern "C" fn roc_crashed(bytes: *const u8, len: usize) {
    let msg = if bytes.is_null() || len == 0 {
        "roc_crashed"
    } else {
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(bytes, len) })
            .unwrap_or("roc_crashed")
    };
    panic!("{msg}");
}

unsafe extern "C" {
    fn roc_init_for_host(out: *mut u8);
    fn roc_respond_for_host(out: *mut u8, request: *const u8, context: *mut c_void);
    fn roc_shutdown_for_host(out: *mut u8, reason: *const u8, context: *mut c_void);
}

fn small_roc_str(text: &str) -> [u8; 12] {
    let mut raw = [0u8; 12];
    let bytes = text.as_bytes();
    assert!(bytes.len() < 12, "path too long for small RocStr");
    raw[..bytes.len()].copy_from_slice(bytes);
    raw[11] = (bytes.len() as u8) ^ 0x80;
    raw
}

fn empty_roc_str() -> [u8; 12] {
    small_roc_str("")
}

fn write_request(buf: &mut [u8; REQUEST_SIZE], request: &ServerRequest) {
    buf.fill(0);
    buf[0..8].copy_from_slice(&request.body_limit_bytes.to_le_bytes());
    buf[8..16].copy_from_slice(&request.content_length.to_le_bytes());
    buf[16..28].copy_from_slice(&empty_roc_str());
    buf[40..52].copy_from_slice(&empty_roc_str());
    buf[52..64].copy_from_slice(&empty_roc_str());
    buf[64..76].copy_from_slice(&small_roc_str(&request.target_path));
    buf[76..88].copy_from_slice(&empty_roc_str());
    buf[100] = request.method;
    buf[103] = request.target_tag;
}

pub struct LinkedHelloWebGuest {
    context: usize,
}

impl LinkedHelloWebGuest {
    pub fn new() -> Self {
        Self { context: 0 }
    }

    fn context_ptr(&self) -> *mut c_void {
        self.context as *mut c_void
    }
}

impl Default for LinkedHelloWebGuest {
    fn default() -> Self {
        Self::new()
    }
}

impl RocGuest for LinkedHelloWebGuest {
    fn init(&mut self) {
        let mut out = [0u8; INIT_SIZE];
        unsafe { roc_init_for_host(out.as_mut_ptr()) };
        assert_eq!(out[INIT_TAG], 1, "roc_init_for_host");
        self.context =
            u32::from_le_bytes(out[INIT_CONTEXT..INIT_CONTEXT + 4].try_into().unwrap()) as usize;
    }

    fn respond(&mut self, request: &ServerRequest) -> OutcomeToHost {
        let mut raw = [0u8; REQUEST_SIZE];
        write_request(&mut raw, request);
        let mut out = [0u8; OUTCOME_SIZE];
        unsafe { roc_respond_for_host(out.as_mut_ptr(), raw.as_ptr(), self.context_ptr()) };
        assert_eq!(out[OUTCOME_TAG], ORDINARY, "ordinary outcome");
        let status = u16::from_le_bytes(out[STATUS..STATUS + 2].try_into().unwrap());
        let body_ptr =
            u32::from_le_bytes(out[BODY_PTR..BODY_PTR + 4].try_into().unwrap()) as *const u8;
        let body_len = u32::from_le_bytes(out[BODY_LEN..BODY_LEN + 4].try_into().unwrap()) as usize;
        let body = if body_ptr.is_null() || body_len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(body_ptr, body_len) }.to_vec()
        };
        OutcomeToHost::Ordinary(OrdinaryResponse {
            exit_code: 0,
            body,
            headers: vec![ServerHeader {
                name: "content-type".into(),
                value: "text/html; charset=utf-8".into(),
            }],
            status,
            stop: false,
        })
    }

    fn shutdown(&mut self) {
        let mut out = [0u8; 16];
        let reason = [0u8; 8];
        unsafe { roc_shutdown_for_host(out.as_mut_ptr(), reason.as_ptr(), self.context_ptr()) };
        self.context = 0;
    }
}
