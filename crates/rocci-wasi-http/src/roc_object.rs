//! Call the linked `roc_*_for_host` object (wasm32 C ABI).

use std::alloc::Layout;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use crate::abi::{OrdinaryResponse, OutcomeToHost, ServerHeader, ServerRequest};
use crate::guest::RocGuest;

fn allocs() -> &'static Mutex<HashMap<usize, Layout>> {
    static ALLOCS: OnceLock<Mutex<HashMap<usize, Layout>>> = OnceLock::new();
    ALLOCS.get_or_init(|| Mutex::new(HashMap::new()))
}

const INIT_SIZE: usize = 224;
const INIT_TAG: usize = 216;
const INIT_CONTEXT: usize = 208;
const OUTCOME_SIZE: usize = 48;
const OUTCOME_TAG: usize = 44;
const ORDINARY: u8 = 1;
const STREAM: u8 = 2;
const BODY_PTR: usize = 8;
const STATUS: usize = 32;
const STEP_SIZE: usize = 32;
const STEP_TAG: usize = 24;
const STEP_EMIT: u8 = 0;
const STEP_END: u8 = 1;
const STEP_ERR: u8 = 2;
const STEP_WAIT: u8 = 3;
const REQUEST_SIZE: usize = 104;
const ROC_STR_SIZE: usize = 12;
const RAW_OS_STR_SIZE: usize = 16;
const RAW_OS_STR_TAG: usize = 12;
const OS_UNIX_BYTES: u8 = 0;
const OS_UTF8: u8 = 1;
const ENV_RESULT_SIZE: usize = 24;
const ENV_RESULT_TAG: usize = 20;
const ENV_ERR: u8 = 0;
const ENV_OK: u8 = 1;
const ENV_ERR_INNER_TAG: usize = 16;
const VAR_NOT_FOUND: u8 = 1;
const STDERR_RESULT_SIZE: usize = 20;
const STDERR_RESULT_TAG: usize = 16;
const STDERR_OK: u8 = 1;

#[unsafe(no_mangle)]
pub extern "C" fn roc_alloc(size: usize, alignment: usize) -> *mut u8 {
    let align = alignment.max(1);
    let layout = Layout::from_size_align(size.max(1), align)
        .unwrap_or_else(|_| Layout::from_size_align(1, 1).unwrap());
    let ptr = unsafe { std::alloc::alloc(layout) };
    if !ptr.is_null() {
        allocs()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(ptr as usize, layout);
    }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn roc_realloc(ptr: *mut u8, new_size: usize, alignment: usize) -> *mut u8 {
    if ptr.is_null() {
        return roc_alloc(new_size, alignment);
    }
    let old = allocs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&(ptr as usize));
    let next = roc_alloc(new_size, alignment);
    if let Some(old) = old {
        if !next.is_null() {
            let n = old.size().min(new_size);
            if n > 0 {
                unsafe { std::ptr::copy_nonoverlapping(ptr, next, n) };
            }
        }
        unsafe { std::alloc::dealloc(ptr, old) };
    }
    next
}

#[unsafe(no_mangle)]
pub extern "C" fn roc_dealloc(ptr: *mut u8, _alignment: usize) {
    if ptr.is_null() {
        return;
    }
    if let Some(layout) = allocs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&(ptr as usize))
    {
        unsafe { std::alloc::dealloc(ptr, layout) };
    }
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
    fn roc_sse_advance_for_host(out: *mut u8, source: *mut u8, wake: u64);
}

pub(crate) fn read_u32(ptr: *const u8, offset: usize) -> u32 {
    u32::from_le_bytes(
        unsafe { std::slice::from_raw_parts(ptr.add(offset), 4) }
            .try_into()
            .expect("u32"),
    )
}

pub(crate) fn write_u32(ptr: *mut u8, offset: usize, value: u32) {
    unsafe {
        ptr.add(offset)
            .copy_from_nonoverlapping(value.to_le_bytes().as_ptr(), 4);
    }
}

pub(crate) fn read_roc_str(ptr: *const u8) -> String {
    let last = unsafe { *ptr.add(ROC_STR_SIZE - 1) };
    if last & 0x80 != 0 {
        let len = (last ^ 0x80) as usize;
        return String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
            .into_owned();
    }
    let bytes = read_u32(ptr, 0) as *const u8;
    let len = read_u32(ptr, 8) as usize;
    if bytes.is_null() || len == 0 {
        return String::new();
    }
    String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(bytes, len) }).into_owned()
}

pub(crate) fn write_roc_str(ptr: *mut u8, text: &str) {
    let bytes = text.as_bytes();
    unsafe { std::ptr::write_bytes(ptr, 0, ROC_STR_SIZE) };
    if bytes.len() < ROC_STR_SIZE {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len()) };
        unsafe { *ptr.add(ROC_STR_SIZE - 1) = (bytes.len() as u8) ^ 0x80 };
        return;
    }
    let header = std::mem::size_of::<usize>();
    let total = header + bytes.len();
    let base = roc_alloc(total, header);
    assert!(!base.is_null(), "roc_alloc for RocStr");
    unsafe {
        (base as *mut i32).write(1);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), base.add(header), bytes.len());
    }
    write_u32(ptr, 0, base as u32 + header as u32);
    write_u32(ptr, 4, (bytes.len() as u32) << 1);
    write_u32(ptr, 8, bytes.len() as u32);
}

fn read_raw_os_str(ptr: *const u8) -> Option<String> {
    let tag = unsafe { *ptr.add(RAW_OS_STR_TAG) };
    match tag {
        OS_UTF8 => Some(read_roc_str(ptr)),
        OS_UNIX_BYTES => {
            let elements = read_u32(ptr, 0) as *const u8;
            let len = read_u32(ptr, 4) as usize;
            if elements.is_null() || len == 0 {
                return Some(String::new());
            }
            Some(
                String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(elements, len) })
                    .into_owned(),
            )
        }
        _ => None,
    }
}

fn write_raw_os_str_utf8(ptr: *mut u8, text: &str) {
    unsafe { std::ptr::write_bytes(ptr, 0, RAW_OS_STR_SIZE) };
    write_roc_str(ptr, text);
    unsafe { *ptr.add(RAW_OS_STR_TAG) = OS_UTF8 };
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_stderr_line(out: *mut u8, message: *const u8) {
    let text = if message.is_null() {
        String::new()
    } else {
        read_roc_str(message)
    };
    eprintln!("{text}");
    unsafe { std::ptr::write_bytes(out, 0, STDERR_RESULT_SIZE) };
    unsafe { *out.add(STDERR_RESULT_TAG) = STDERR_OK };
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_env_is_windows(_dummy: *const u8) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_env_var(out: *mut u8, name: *const u8) {
    unsafe { std::ptr::write_bytes(out, 0, ENV_RESULT_SIZE) };
    let key = if name.is_null() {
        String::new()
    } else {
        read_raw_os_str(name).unwrap_or_default()
    };
    match std::env::var(&key) {
        Ok(value) => {
            write_raw_os_str_utf8(out, &value);
            unsafe { *out.add(ENV_RESULT_TAG) = ENV_OK };
        }
        Err(_) => {
            write_raw_os_str_utf8(out, &key);
            unsafe { *out.add(ENV_ERR_INNER_TAG) = VAR_NOT_FOUND };
            unsafe { *out.add(ENV_RESULT_TAG) = ENV_ERR };
        }
    }
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

fn incref_roc_box(ptr: usize) {
    if ptr == 0 {
        return;
    }
    let rc = (ptr - std::mem::size_of::<isize>()) as *mut isize;
    let n = unsafe { rc.read() };
    if n == 0 {
        return;
    }
    unsafe { rc.write(n + 1) };
}

fn read_list_u8(ptr: *const u8, offset: usize) -> Vec<u8> {
    let bytes = read_u32(ptr, offset) as *const u8;
    let len = read_u32(ptr, offset + 4) as usize;
    if bytes.is_null() || len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(bytes, len) }.to_vec()
    }
}

fn drain_sse_source(mut source: usize) -> Vec<u8> {
    let mut body = Vec::new();
    let mut wake = 0u64;
    loop {
        let mut step = [0u8; STEP_SIZE];
        unsafe { roc_sse_advance_for_host(step.as_mut_ptr(), source as *mut u8, wake) };
        match step[STEP_TAG] {
            STEP_EMIT => {
                body.extend_from_slice(&read_list_u8(step.as_ptr(), 8));
                source = read_u32(step.as_ptr(), 20) as usize;
                let wait = u64::from_le_bytes(step[0..8].try_into().unwrap());
                if wait > 0 {
                    wake = wake.wrapping_add(1);
                }
            }
            STEP_WAIT => {
                source = read_u32(step.as_ptr(), 8) as usize;
                wake = wake.wrapping_add(1);
            }
            STEP_END => break,
            STEP_ERR => panic!("roc_sse_advance_for_host error"),
            tag => panic!("roc_sse_advance_for_host tag={tag}"),
        }
    }
    body
}

fn write_request(buf: &mut [u8; REQUEST_SIZE], request: &ServerRequest) {
    buf.fill(0);
    buf[0..8].copy_from_slice(&request.body_limit_bytes.to_le_bytes());
    buf[8..16].copy_from_slice(&request.content_length.to_le_bytes());
    buf[16..28].copy_from_slice(&empty_roc_str());
    buf[44..56].copy_from_slice(&empty_roc_str());
    buf[56..68].copy_from_slice(&empty_roc_str());
    write_roc_str(buf[68..80].as_mut_ptr(), &request.target_path);
    buf[80..92].copy_from_slice(&empty_roc_str());
    buf[98] = 1;
    buf[99] = request.method;
    buf[102] = request.target_tag;
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
        if out[INIT_TAG] != 1 {
            let code = i64::from_le_bytes(out[0..8].try_into().unwrap());
            panic!("roc_init_for_host err={code}");
        }
        self.context =
            u32::from_le_bytes(out[INIT_CONTEXT..INIT_CONTEXT + 4].try_into().unwrap()) as usize;
    }

    fn respond(&mut self, request: &ServerRequest) -> OutcomeToHost {
        let mut raw = [0u8; REQUEST_SIZE];
        write_request(&mut raw, request);
        let mut out = [0u8; OUTCOME_SIZE];
        incref_roc_box(self.context);
        unsafe { roc_respond_for_host(out.as_mut_ptr(), raw.as_ptr(), self.context_ptr()) };
        match out[OUTCOME_TAG] {
            ORDINARY => {
                let status = u16::from_le_bytes(out[STATUS..STATUS + 2].try_into().unwrap());
                OutcomeToHost::Ordinary(OrdinaryResponse {
                    exit_code: 0,
                    body: read_list_u8(out.as_ptr(), BODY_PTR),
                    headers: vec![ServerHeader {
                        name: "content-type".into(),
                        value: "text/html; charset=utf-8".into(),
                    }],
                    status,
                    stop: false,
                })
            }
            STREAM => OutcomeToHost::Ordinary(OrdinaryResponse {
                exit_code: 0,
                body: drain_sse_source(read_u32(out.as_ptr(), 0) as usize),
                headers: vec![ServerHeader {
                    name: "content-type".into(),
                    value: "text/event-stream".into(),
                }],
                status: 200,
                stop: false,
            }),
            tag => panic!("roc_respond_for_host tag={tag}"),
        }
    }

    fn shutdown(&mut self) {
        let mut out = [0u8; 16];
        let reason = [0u8; 8];
        unsafe { roc_shutdown_for_host(out.as_mut_ptr(), reason.as_ptr(), self.context_ptr()) };
        self.context = 0;
    }
}
