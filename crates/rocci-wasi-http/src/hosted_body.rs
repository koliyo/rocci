//! Buffered `hosted_request_body_read_all` for the wasm component.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::roc_object::{roc_alloc, write_u32};

const READ_ALL_SIZE: usize = 32;
const READ_ALL_TAG: usize = 24;
const READ_ALL_OK: u8 = 1;
const ROC_LIST_SIZE: usize = 12;

struct Store {
    next: u64,
    bodies: HashMap<u64, Vec<u8>>,
}

fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(Store {
            next: 1,
            bodies: HashMap::new(),
        })
    })
}

fn lock_store() -> std::sync::MutexGuard<'static, Store> {
    store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn alloc_handle(id: u64) -> *mut u64 {
    let base = roc_alloc(12, 8);
    assert!(!base.is_null(), "roc_alloc body handle");
    unsafe {
        (base as *mut i32).write(1);
        (base.add(4) as *mut u64).write_unaligned(id);
        base.add(4) as *mut u64
    }
}

fn handle_id(handle: *mut u64) -> Option<u64> {
    if handle.is_null() {
        return None;
    }
    Some(unsafe { handle.read_unaligned() })
}

fn write_roc_list_u8(dest: *mut u8, bytes: &[u8]) {
    unsafe { std::ptr::write_bytes(dest, 0, ROC_LIST_SIZE) };
    if bytes.is_empty() {
        return;
    }
    let header = 8;
    let total = header + bytes.len();
    let base = roc_alloc(total, 4);
    assert!(!base.is_null(), "roc_alloc body list");
    let elems = unsafe { base.add(header) };
    unsafe {
        (base as *mut u32).write(bytes.len() as u32);
        (base.add(4) as *mut i32).write(1);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), elems, bytes.len());
    }
    write_u32(dest, 0, elems as u32);
    write_u32(dest, 4, bytes.len() as u32);
    write_u32(dest, 8, (bytes.len() as u32) << 1);
}

pub fn register(bytes: &[u8]) -> *mut u64 {
    let mut store = lock_store();
    let id = store.next;
    store.next += 1;
    store.bodies.insert(id, bytes.to_vec());
    drop(store);
    alloc_handle(id)
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_request_body_read_all(out: *mut u8, handle: *mut u64, limit: u64) {
    unsafe { std::ptr::write_bytes(out, 0, READ_ALL_SIZE) };
    let bytes = handle_id(handle).and_then(|id| lock_store().bodies.get(&id).cloned());
    let Some(bytes) = bytes else {
        write_roc_list_u8(out, &[]);
        unsafe { *out.add(READ_ALL_TAG) = READ_ALL_OK };
        return;
    };
    let take = if limit == 0 {
        bytes.len()
    } else {
        bytes.len().min(limit as usize)
    };
    write_roc_list_u8(out, &bytes[..take]);
    unsafe { *out.add(READ_ALL_TAG) = READ_ALL_OK };
}
