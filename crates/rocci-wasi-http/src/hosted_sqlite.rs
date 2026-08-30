//! Sync hosted sqlite for the wasm component. Nested queries serialize `handle`s.

use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_int};
use std::sync::{Mutex, OnceLock};

use crate::roc_object::{read_roc_str, read_u32, roc_alloc, write_roc_str, write_u32};

const SQLITE_OK: c_int = 0;
const SQLITE_ROW: c_int = 100;
const SQLITE_DONE: c_int = 101;
const SQLITE_OPEN_READWRITE: c_int = 0x0000_0002;
const SQLITE_OPEN_CREATE: c_int = 0x0000_0004;
const SQLITE_OPEN_URI: c_int = 0x0000_0040;
const SQLITE_INTEGER: c_int = 1;
const SQLITE_FLOAT: c_int = 2;
const SQLITE_TEXT: c_int = 3;
const SQLITE_BLOB: c_int = 4;

const HANDLE_OK: u8 = 1;
const HANDLE_ERR: u8 = 0;
const HANDLE_RESULT_SIZE: usize = 32;
const HANDLE_RESULT_TAG: usize = 24;
const COLUMNS_RESULT_SIZE: usize = 32;
const COLUMNS_RESULT_TAG: usize = 24;
const NEXT_RESULT_SIZE: usize = 40;
const NEXT_RESULT_TAG: usize = 32;
const NEXT_OK: u8 = 1;
const NEXT_DONE: u8 = 0;
const NEXT_ROW: u8 = 2;
const NEXT_ROW_LIMIT: u8 = 3;
const VALUE_INTEGER: u8 = 1;
const VALUE_STRING: u8 = 4;
const VALUE_NULL: u8 = 2;
const ROC_STR_SIZE: usize = 12;
const ROC_LIST_SIZE: usize = 12;
const PATH_SIZE: usize = 28;
const BINDING_SIZE: usize = 32;
const BINDING_VALUE_TAG: usize = 12;
const BINDING_NAME: usize = 16;
const VALUE_BYTES: u8 = 0;
const VALUE_REAL: u8 = 3;
const SQLITE_TRANSIENT: *const u8 = (-1isize) as *const u8;

#[repr(C)]
struct Sqlite3 {
    _private: [u8; 0],
}

#[repr(C)]
struct Sqlite3Stmt {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn sqlite3_open_v2(
        filename: *const c_char,
        pp_db: *mut *mut Sqlite3,
        flags: c_int,
        vfs: *const c_char,
    ) -> c_int;
    fn sqlite3_close(db: *mut Sqlite3) -> c_int;
    fn sqlite3_exec(
        db: *mut Sqlite3,
        sql: *const c_char,
        callback: *const u8,
        arg: *mut u8,
        errmsg: *mut *mut c_char,
    ) -> c_int;
    fn sqlite3_prepare_v2(
        db: *mut Sqlite3,
        sql: *const c_char,
        n_byte: c_int,
        stmt: *mut *mut Sqlite3Stmt,
        tail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_step(stmt: *mut Sqlite3Stmt) -> c_int;
    fn sqlite3_finalize(stmt: *mut Sqlite3Stmt) -> c_int;
    fn sqlite3_column_count(stmt: *mut Sqlite3Stmt) -> c_int;
    fn sqlite3_column_name(stmt: *mut Sqlite3Stmt, n: c_int) -> *const c_char;
    fn sqlite3_column_type(stmt: *mut Sqlite3Stmt, i: c_int) -> c_int;
    fn sqlite3_column_int64(stmt: *mut Sqlite3Stmt, i: c_int) -> i64;
    fn sqlite3_column_text(stmt: *mut Sqlite3Stmt, i: c_int) -> *const u8;
    fn sqlite3_column_bytes(stmt: *mut Sqlite3Stmt, i: c_int) -> c_int;
    fn sqlite3_errmsg(db: *mut Sqlite3) -> *const c_char;
    fn sqlite3_busy_timeout(db: *mut Sqlite3, ms: c_int) -> c_int;
    fn sqlite3_bind_int64(stmt: *mut Sqlite3Stmt, index: c_int, value: i64) -> c_int;
    fn sqlite3_bind_text(
        stmt: *mut Sqlite3Stmt,
        index: c_int,
        value: *const c_char,
        n: c_int,
        destructor: *const u8,
    ) -> c_int;
    fn sqlite3_bind_null(stmt: *mut Sqlite3Stmt, index: c_int) -> c_int;
    fn sqlite3_bind_parameter_index(stmt: *mut Sqlite3Stmt, name: *const c_char) -> c_int;
}

enum Resource {
    Db(*mut Sqlite3),
    Stmt {
        db: u64,
        sql: String,
        columns: Vec<String>,
    },
    Exec {
        stmt: *mut Sqlite3Stmt,
    },
}

unsafe impl Send for Resource {}

struct Store {
    next: u64,
    items: HashMap<u64, Resource>,
}

fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(Store {
            next: 1,
            items: HashMap::new(),
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
    assert!(!base.is_null(), "roc_alloc sqlite handle");
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

fn write_err(out: *mut u8, size: usize, tag_off: usize, code: i64, message: &str) {
    unsafe { std::ptr::write_bytes(out, 0, size) };
    unsafe { std::ptr::copy_nonoverlapping(code.to_le_bytes().as_ptr(), out, 8) };
    write_roc_str(unsafe { out.add(8) }, message);
    unsafe { *out.add(tag_off) = HANDLE_ERR };
}

fn write_handle_ok(out: *mut u8, handle: *mut u64) {
    unsafe { std::ptr::write_bytes(out, 0, HANDLE_RESULT_SIZE) };
    write_u32(out, 0, handle as u32);
    unsafe { *out.add(HANDLE_RESULT_TAG) = HANDLE_OK };
}

fn write_roc_list_strs(dest: *mut u8, items: &[String]) {
    unsafe { std::ptr::write_bytes(dest, 0, ROC_LIST_SIZE) };
    if items.is_empty() {
        return;
    }
    let header = 8;
    let total = header + items.len() * ROC_STR_SIZE;
    let base = roc_alloc(total, 4);
    assert!(!base.is_null(), "roc_alloc column list");
    let elems = unsafe { base.add(header) };
    unsafe {
        (base as *mut u32).write(items.len() as u32);
        (base.add(4) as *mut i32).write(1);
    }
    for (index, name) in items.iter().enumerate() {
        write_roc_str(unsafe { elems.add(index * ROC_STR_SIZE) }, name);
    }
    write_u32(dest, 0, elems as u32);
    write_u32(dest, 4, items.len() as u32);
    write_u32(dest, 8, (items.len() as u32) << 1);
}

fn write_value_list(dest: *mut u8, values: &[SqliteValue]) {
    unsafe { std::ptr::write_bytes(dest, 0, ROC_LIST_SIZE) };
    if values.is_empty() {
        return;
    }
    let elem = 16;
    let header = 8;
    let total = header + values.len() * elem;
    let base = roc_alloc(total, 8);
    assert!(!base.is_null(), "roc_alloc row values");
    let elems = unsafe { base.add(header) };
    unsafe {
        (base as *mut u32).write(values.len() as u32);
        (base.add(4) as *mut i32).write(1);
    }
    for (index, value) in values.iter().enumerate() {
        let slot = unsafe { elems.add(index * elem) };
        unsafe { std::ptr::write_bytes(slot, 0, elem) };
        match value {
            SqliteValue::Null => unsafe { *slot.add(12) = VALUE_NULL },
            SqliteValue::Integer(n) => {
                unsafe { std::ptr::copy_nonoverlapping(n.to_le_bytes().as_ptr(), slot, 8) };
                unsafe { *slot.add(12) = VALUE_INTEGER };
            }
            SqliteValue::Text(text) => {
                write_roc_str(slot, text);
                unsafe { *slot.add(12) = VALUE_STRING };
            }
        }
    }
    write_u32(dest, 0, elems as u32);
    write_u32(dest, 4, values.len() as u32);
    write_u32(dest, 8, (values.len() as u32) << 1);
}

enum SqliteValue {
    Null,
    Integer(i64),
    Text(String),
}

fn read_path(path: *const u8) -> String {
    if path.is_null() {
        return String::new();
    }
    let is_windows = unsafe { *path.add(24) } != 0;
    if is_windows {
        return String::new();
    }
    let elements = read_u32(path, 0) as *const u8;
    let len = read_u32(path, 4) as usize;
    if elements.is_null() || len == 0 {
        return String::new();
    }
    String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(elements, len) }).into_owned()
}

fn c_msg(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return "sqlite".into();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

fn exec_sql(db: *mut Sqlite3, sql: &str) -> Result<(), (i64, String)> {
    let c_sql = CString::new(sql).map_err(|_| (1_i64, "nul in sql".into()))?;
    let rc = unsafe {
        sqlite3_exec(
            db,
            c_sql.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if rc == SQLITE_OK {
        Ok(())
    } else {
        Err((rc as i64, c_msg(unsafe { sqlite3_errmsg(db) })))
    }
}

fn bind_params(stmt: *mut Sqlite3Stmt, bindings: *const u8) -> Result<(), (i64, String)> {
    if bindings.is_null() {
        return Ok(());
    }
    let elems = read_u32(bindings, 0) as *const u8;
    let len = read_u32(bindings, 4) as usize;
    if elems.is_null() || len == 0 {
        return Ok(());
    }
    for index in 0..len {
        let elem = unsafe { elems.add(index * BINDING_SIZE) };
        let name = read_roc_str(unsafe { elem.add(BINDING_NAME) });
        let c_name = CString::new(name).map_err(|_| (1_i64, "nul in binding name".into()))?;
        let param = unsafe { sqlite3_bind_parameter_index(stmt, c_name.as_ptr()) };
        if param == 0 {
            continue;
        }
        let tag = unsafe { *elem.add(BINDING_VALUE_TAG) };
        let rc = match tag {
            VALUE_INTEGER => {
                let value = i64::from_le_bytes(
                    unsafe { std::slice::from_raw_parts(elem, 8) }
                        .try_into()
                        .expect("i64"),
                );
                unsafe { sqlite3_bind_int64(stmt, param, value) }
            }
            VALUE_STRING => {
                let text = read_roc_str(elem);
                let c_text = CString::new(text).map_err(|_| (1_i64, "nul in binding".into()))?;
                let rc = unsafe {
                    sqlite3_bind_text(stmt, param, c_text.as_ptr(), -1, SQLITE_TRANSIENT)
                };
                let _ = c_text;
                rc
            }
            VALUE_NULL | VALUE_BYTES | VALUE_REAL => unsafe { sqlite3_bind_null(stmt, param) },
            _ => unsafe { sqlite3_bind_null(stmt, param) },
        };
        if rc != SQLITE_OK {
            return Err((rc as i64, "bind failed".into()));
        }
    }
    Ok(())
}

fn prepare_stmt(db: *mut Sqlite3, sql: &str) -> Result<*mut Sqlite3Stmt, (i64, String)> {
    let c_sql = CString::new(sql).map_err(|_| (1_i64, "nul in sql".into()))?;
    let mut stmt = std::ptr::null_mut();
    let rc = unsafe { sqlite3_prepare_v2(db, c_sql.as_ptr(), -1, &mut stmt, std::ptr::null_mut()) };
    if rc == SQLITE_OK && !stmt.is_null() {
        Ok(stmt)
    } else {
        if !stmt.is_null() {
            unsafe { sqlite3_finalize(stmt) };
        }
        Err((rc as i64, c_msg(unsafe { sqlite3_errmsg(db) })))
    }
}

fn column_names(stmt: *mut Sqlite3Stmt) -> Vec<String> {
    let count = unsafe { sqlite3_column_count(stmt) }.max(0) as usize;
    (0..count)
        .map(|index| {
            let name = unsafe { sqlite3_column_name(stmt, index as c_int) };
            c_msg(name)
        })
        .collect()
}

fn read_row(stmt: *mut Sqlite3Stmt) -> Vec<SqliteValue> {
    let count = unsafe { sqlite3_column_count(stmt) }.max(0) as usize;
    (0..count)
        .map(|index| {
            let i = index as c_int;
            match unsafe { sqlite3_column_type(stmt, i) } {
                SQLITE_INTEGER => SqliteValue::Integer(unsafe { sqlite3_column_int64(stmt, i) }),
                SQLITE_FLOAT | SQLITE_BLOB => SqliteValue::Null,
                SQLITE_TEXT => {
                    let ptr = unsafe { sqlite3_column_text(stmt, i) };
                    let len = unsafe { sqlite3_column_bytes(stmt, i) }.max(0) as usize;
                    if ptr.is_null() {
                        SqliteValue::Text(String::new())
                    } else {
                        SqliteValue::Text(
                            String::from_utf8_lossy(unsafe {
                                std::slice::from_raw_parts(ptr, len)
                            })
                            .into_owned(),
                        )
                    }
                }
                _ => SqliteValue::Null,
            }
        })
        .collect()
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_sqlite_open(
    out: *mut u8,
    path: *const u8,
    _max_connections: i64,
    _acquire_timeout_ms: i64,
    _busy_timeout_ms: i64,
    _max_cached: i64,
    journal_mode: i64,
    _synchronous: i64,
) {
    let filename = if path.is_null() || PATH_SIZE == 0 {
        ":memory:".into()
    } else {
        let raw = read_path(path);
        if raw.is_empty() {
            ":memory:".into()
        } else {
            raw
        }
    };
    let memory = filename == ":memory:";
    let open_name = if memory {
        filename
    } else {
        format!("file:{filename}?mode=rwc&nolock=1")
    };
    let c_path = match CString::new(open_name) {
        Ok(value) => value,
        Err(_) => {
            write_err(out, HANDLE_RESULT_SIZE, HANDLE_RESULT_TAG, 14, "bad path");
            return;
        }
    };
    let mut db = std::ptr::null_mut();
    let flags = if memory {
        SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE
    } else {
        SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI
    };
    let rc = unsafe { sqlite3_open_v2(c_path.as_ptr(), &mut db, flags, std::ptr::null()) };
    if rc != SQLITE_OK || db.is_null() {
        let message = if db.is_null() {
            "open failed".into()
        } else {
            let message = c_msg(unsafe { sqlite3_errmsg(db) });
            unsafe { sqlite3_close(db) };
            message
        };
        write_err(
            out,
            HANDLE_RESULT_SIZE,
            HANDLE_RESULT_TAG,
            rc as i64,
            &message,
        );
        return;
    }
    let _ = journal_mode;
    unsafe { sqlite3_busy_timeout(db, 5_000) };
    let pragma = "PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;";
    if let Err((code, message)) = exec_sql(db, pragma) {
        unsafe { sqlite3_close(db) };
        write_err(out, HANDLE_RESULT_SIZE, HANDLE_RESULT_TAG, code, &message);
        return;
    }
    let mut store = lock_store();
    let id = store.next;
    store.next += 1;
    store.items.insert(id, Resource::Db(db));
    drop(store);
    write_handle_ok(out, alloc_handle(id));
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_sqlite_prepare(out: *mut u8, database: *mut u64, query: *const u8) {
    let sql = if query.is_null() {
        String::new()
    } else {
        read_roc_str(query)
    };
    let Some(db_id) = handle_id(database) else {
        write_err(out, HANDLE_RESULT_SIZE, HANDLE_RESULT_TAG, 1, "no database");
        return;
    };
    let db = {
        let store = lock_store();
        match store.items.get(&db_id) {
            Some(Resource::Db(db)) => *db,
            _ => {
                write_err(out, HANDLE_RESULT_SIZE, HANDLE_RESULT_TAG, 1, "no database");
                return;
            }
        }
    };
    let stmt = match prepare_stmt(db, &sql) {
        Ok(stmt) => stmt,
        Err((code, message)) => {
            write_err(out, HANDLE_RESULT_SIZE, HANDLE_RESULT_TAG, code, &message);
            return;
        }
    };
    let columns = column_names(stmt);
    unsafe { sqlite3_finalize(stmt) };
    let mut store = lock_store();
    let id = store.next;
    store.next += 1;
    store.items.insert(
        id,
        Resource::Stmt {
            db: db_id,
            sql,
            columns,
        },
    );
    drop(store);
    write_handle_ok(out, alloc_handle(id));
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_sqlite_columns(out: *mut u8, statement: *mut u64) {
    let Some(id) = handle_id(statement) else {
        write_err(
            out,
            COLUMNS_RESULT_SIZE,
            COLUMNS_RESULT_TAG,
            1,
            "no statement",
        );
        return;
    };
    let columns = {
        let store = lock_store();
        match store.items.get(&id) {
            Some(Resource::Stmt { columns, .. }) => columns.clone(),
            _ => {
                write_err(
                    out,
                    COLUMNS_RESULT_SIZE,
                    COLUMNS_RESULT_TAG,
                    1,
                    "no statement",
                );
                return;
            }
        }
    };
    unsafe { std::ptr::write_bytes(out, 0, COLUMNS_RESULT_SIZE) };
    write_roc_list_strs(out, &columns);
    unsafe { *out.add(COLUMNS_RESULT_TAG) = HANDLE_OK };
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_sqlite_start(
    out: *mut u8,
    statement: *mut u64,
    bindings: *const u8,
    _timeout_ms: i64,
) {
    let Some(id) = handle_id(statement) else {
        write_err(
            out,
            HANDLE_RESULT_SIZE,
            HANDLE_RESULT_TAG,
            1,
            "no statement",
        );
        return;
    };
    let (db_id, sql) = {
        let store = lock_store();
        match store.items.get(&id) {
            Some(Resource::Stmt { db, sql, .. }) => (*db, sql.clone()),
            _ => {
                write_err(
                    out,
                    HANDLE_RESULT_SIZE,
                    HANDLE_RESULT_TAG,
                    1,
                    "no statement",
                );
                return;
            }
        }
    };
    let db = {
        let store = lock_store();
        match store.items.get(&db_id) {
            Some(Resource::Db(db)) => *db,
            _ => {
                write_err(out, HANDLE_RESULT_SIZE, HANDLE_RESULT_TAG, 1, "no database");
                return;
            }
        }
    };
    let stmt = match prepare_stmt(db, &sql) {
        Ok(stmt) => stmt,
        Err((code, message)) => {
            write_err(out, HANDLE_RESULT_SIZE, HANDLE_RESULT_TAG, code, &message);
            return;
        }
    };
    if let Err((code, message)) = bind_params(stmt, bindings) {
        unsafe { sqlite3_finalize(stmt) };
        write_err(out, HANDLE_RESULT_SIZE, HANDLE_RESULT_TAG, code, &message);
        return;
    }
    let mut store = lock_store();
    let exec_id = store.next;
    store.next += 1;
    store.items.insert(exec_id, Resource::Exec { stmt });
    drop(store);
    write_handle_ok(out, alloc_handle(exec_id));
}

#[unsafe(no_mangle)]
pub extern "C" fn hosted_sqlite_next_row(
    out: *mut u8,
    execution: *mut u64,
    _max_bytes: i64,
    allow_row: i32,
) {
    unsafe { std::ptr::write_bytes(out, 0, NEXT_RESULT_SIZE) };
    let Some(id) = handle_id(execution) else {
        write_err(out, NEXT_RESULT_SIZE, NEXT_RESULT_TAG, 1, "no execution");
        return;
    };
    let stmt = {
        let store = lock_store();
        match store.items.get(&id) {
            Some(Resource::Exec { stmt }) => *stmt,
            _ => {
                write_err(out, NEXT_RESULT_SIZE, NEXT_RESULT_TAG, 1, "no execution");
                return;
            }
        }
    };
    let step = unsafe { sqlite3_step(stmt) };
    if step == SQLITE_DONE {
        unsafe { *out.add(24) = NEXT_DONE };
        unsafe { *out.add(NEXT_RESULT_TAG) = NEXT_OK };
        return;
    }
    if step != SQLITE_ROW {
        write_err(out, NEXT_RESULT_SIZE, NEXT_RESULT_TAG, step as i64, "step");
        return;
    }
    if allow_row == 0 {
        unsafe { *out.add(24) = NEXT_ROW_LIMIT };
        unsafe { *out.add(NEXT_RESULT_TAG) = NEXT_OK };
        return;
    }
    let values = read_row(stmt);
    let bytes = values
        .iter()
        .map(|value| match value {
            SqliteValue::Text(text) => text.len() as u64,
            SqliteValue::Integer(_) => 8,
            SqliteValue::Null => 0,
        })
        .sum::<u64>();
    unsafe { std::ptr::copy_nonoverlapping(bytes.to_le_bytes().as_ptr(), out, 8) };
    write_value_list(unsafe { out.add(8) }, &values);
    unsafe { *out.add(24) = NEXT_ROW };
    unsafe { *out.add(NEXT_RESULT_TAG) = NEXT_OK };
}
