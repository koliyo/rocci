//! The guest owns Roc layouts. JavaScript sees a versioned, domain-neutral
//! envelope and generation-scoped resource IDs, never Roc records or pointers.
#![allow(improper_ctypes_definitions)]

use crate::roc_platform_abi::*;
use core::{ffi::c_void, mem, ptr};
use serde_json::{Value, json};
use std::alloc::{Layout, alloc, dealloc};
use std::collections::BTreeMap;

#[link(wasm_import_module = "rocci")]
unsafe extern "C" {
    fn env(op: u32, input: *const u8, len: usize, output: *mut u8, capacity: usize) -> usize;
    fn log(op: u32, input: *const u8, len: usize, output: *mut u8, capacity: usize) -> usize;
    fn sql(op: u32, input: *const u8, len: usize, output: *mut u8, capacity: usize) -> usize;
    fn transaction(
        op: u32,
        input: *const u8,
        len: usize,
        output: *mut u8,
        capacity: usize,
    ) -> usize;
    fn body(op: u32, input: *const u8, len: usize, output: *mut u8, capacity: usize) -> usize;
    fn http(op: u32, input: *const u8, len: usize, output: *mut u8, capacity: usize) -> usize;
    fn clock(op: u32, input: *const u8, len: usize, output: *mut u8, capacity: usize) -> usize;
    fn sleep(op: u32, input: *const u8, len: usize, output: *mut u8, capacity: usize) -> usize;
    fn resource_drop(id: u64);
}

const LIMIT: usize = 8 * 1024 * 1024;
static mut INPUT: [u8; LIMIT] = [0; LIMIT];
static mut OUTPUT: Vec<u8> = Vec::new();
static mut CONTEXT: RocBox = ptr::null_mut();
static mut INITIALIZED: bool = false;
static mut BODY_LIMIT: u64 = 1024 * 1024;
static mut STREAM_LIMIT: usize = 128;
static mut STREAMS: BTreeMap<u32, RocErasedCallable> = BTreeMap::new();
static mut NEXT_STREAM: u32 = 1;
static mut ACTIVE: bool = false;
static mut LIVE_BYTES: usize = 0;
static mut LIVE_COUNT: usize = 0;

#[repr(C, align(16))]
struct Allocation {
    size: usize,
    alignment: usize,
    resource: u64,
}

extern "C" fn allocate(_: *mut RocHost, size: usize, alignment: usize) -> *mut c_void {
    let alignment = alignment.max(16);
    let layout = Layout::from_size_align(
        size.checked_add(16).expect("allocation overflow"),
        alignment,
    )
    .expect("allocation alignment");
    // The header is directly before the returned pointer, even for larger alignments.
    let prefix = alignment;
    let layout = Layout::from_size_align(layout.size() + prefix - 16, alignment).unwrap();
    unsafe {
        assert!(
            LIVE_BYTES
                .checked_add(size)
                .is_some_and(|bytes| bytes <= 32 * 1024 * 1024),
            "Worker guest allocation capacity exhausted"
        );
        let base = alloc(layout);
        if base.is_null() {
            core::arch::wasm32::unreachable();
        }
        let data = base.add(prefix);
        data.sub(16).cast::<Allocation>().write(Allocation {
            size,
            alignment,
            resource: 0,
        });
        LIVE_BYTES += size;
        LIVE_COUNT += 1;
        data.cast()
    }
}
extern "C" fn release(_: *mut RocHost, data: *mut c_void, _: usize) {
    if data.is_null() {
        return;
    }
    unsafe {
        let header = data.cast::<u8>().sub(16).cast::<Allocation>().read();
        if header.resource != 0 {
            resource_drop(header.resource);
        }
        LIVE_BYTES -= header.size;
        LIVE_COUNT -= 1;
        let layout =
            Layout::from_size_align(header.size + header.alignment, header.alignment).unwrap();
        dealloc(data.cast::<u8>().sub(header.alignment), layout);
    }
}
extern "C" fn reallocate(
    host: *mut RocHost,
    data: *mut c_void,
    size: usize,
    alignment: usize,
) -> *mut c_void {
    if data.is_null() {
        return allocate(host, size, alignment);
    }
    unsafe {
        let old = &*data.cast::<u8>().sub(16).cast::<Allocation>();
        assert_eq!(old.resource, 0, "cannot resize an opaque resource");
        let next = allocate(host, size, alignment);
        ptr::copy_nonoverlapping(data.cast::<u8>(), next.cast::<u8>(), old.size.min(size));
        release(host, data, alignment);
        next
    }
}
fn host() -> RocHost {
    let mut host = make_roc_host(ptr::null_mut());
    host.roc_alloc = allocate;
    host.roc_dealloc = release;
    host.roc_realloc = reallocate;
    host
}
#[unsafe(no_mangle)]
extern "C" fn roc_alloc(size: usize, alignment: usize) -> *mut c_void {
    allocate(ptr::null_mut(), size, alignment)
}
#[unsafe(no_mangle)]
extern "C" fn roc_dealloc(data: *mut c_void, alignment: usize) {
    release(ptr::null_mut(), data, alignment)
}
#[unsafe(no_mangle)]
extern "C" fn roc_realloc(data: *mut c_void, size: usize, alignment: usize) -> *mut c_void {
    reallocate(ptr::null_mut(), data, size, alignment)
}
#[unsafe(no_mangle)]
extern "C" fn roc_dbg(message: *const u8, length: usize) {
    let message = unsafe { std::slice::from_raw_parts(message, length.min(8192)) };
    rpc::<2>(json!({"message":String::from_utf8_lossy(message)}));
}
#[unsafe(no_mangle)]
extern "C" fn roc_crashed(message: *const u8, length: usize) {
    roc_dbg(message, length);
    core::arch::wasm32::unreachable()
}
#[unsafe(no_mangle)]
extern "C" fn roc_expect_failed(message: *const u8, length: usize) {
    roc_dbg(message, length);
    core::arch::wasm32::unreachable()
}

fn text(value: &str) -> RocStr {
    RocStr::from_str(value, &host())
}
fn bytes(value: &[u8]) -> RocListWith<u8, false> {
    unsafe { RocListWith::from_slice(value, &host()) }
}
fn list<T: Copy>(value: &[T]) -> RocList<T> {
    unsafe { RocList::from_slice(value, &host()) }
}

#[inline(always)]
fn rpc<const OP: u32>(value: Value) -> Value {
    let input = serde_json::to_vec(&value).unwrap();
    assert!(
        input.len() <= LIMIT,
        "Worker effect envelope exceeds ABI capacity"
    );
    let mut output = vec![0u8; LIMIT];
    let length = unsafe {
        let effect: unsafe extern "C" fn(u32, *const u8, usize, *mut u8, usize) -> usize = match OP
        {
            1 => env,
            2 => log,
            3 => sleep,
            4 => clock,
            10..=14 => sql,
            20..=22 => transaction,
            30 => body,
            31 => http,
            _ => core::arch::wasm32::unreachable(),
        };
        effect(
            OP,
            input.as_ptr(),
            input.len(),
            output.as_mut_ptr(),
            output.len(),
        )
    };
    assert!(length <= output.len(), "host reply exceeds ABI buffer");
    serde_json::from_slice(&output[..length]).expect("invalid Worker host reply")
}
fn resource(id: u64) -> *mut u64 {
    assert_ne!(id, 0);
    unsafe {
        let data = allocate_box(8, 8, false, &host()).cast::<u64>();
        data.write(id);
        data.cast::<u8>()
            .sub(8 + 16)
            .cast::<Allocation>()
            .as_mut()
            .unwrap()
            .resource = id;
        data
    }
}
unsafe fn consume_resource(data: *mut u64) -> u64 {
    let id = unsafe { data.read() };
    unsafe {
        decref_box_with(data.cast(), 8, false, None, &host());
    }
    id
}

macro_rules! tagged {
    ($ty:ty, $tag:expr, $value:expr) => {{
        let mut result: $ty = unsafe { mem::zeroed() };
        result.tag = $tag;
        unsafe {
            ptr::write(result.payload.as_mut_ptr().cast(), $value);
        }
        result
    }};
}
fn sql_error(value: &Value) -> HostSqliteOpenErr {
    HostSqliteOpenErr {
        code: value["code"].as_i64().unwrap_or(1),
        message: text(
            value["error"]
                .as_str()
                .unwrap_or("Worker SQL operation failed"),
        ),
    }
}
macro_rules! sql_handle {
    ($ty:ty, $tag:ty, $reply:expr) => {{
        let reply = $reply;
        if let Some(id) = reply["id"].as_u64() {
            tagged!($ty, <$tag>::Ok, resource(id))
        } else {
            tagged!($ty, <$tag>::Err, sql_error(&reply))
        }
    }};
}
fn sql_value(value: &BytesOrIntegerOrNullOrRealOrString) -> Value {
    unsafe {
        match value.tag {
            BytesOrIntegerOrNullOrRealOrStringTag::Null => json!(["null"]),
            BytesOrIntegerOrNullOrRealOrStringTag::Integer => json!([
                "integer",
                value.borrow_payload_integer_unchecked().to_string()
            ]),
            BytesOrIntegerOrNullOrRealOrStringTag::Real => {
                json!(["real", value.borrow_payload_real_unchecked()])
            }
            BytesOrIntegerOrNullOrRealOrStringTag::String => {
                json!(["text", value.borrow_payload_string_unchecked().as_str()])
            }
            BytesOrIntegerOrNullOrRealOrStringTag::Bytes => {
                json!(["blob", value.borrow_payload_bytes_unchecked().as_slice()])
            }
        }
    }
}
fn from_sql(value: &Value) -> BytesOrIntegerOrNullOrRealOrString {
    use BytesOrIntegerOrNullOrRealOrStringTag as T;
    match value[0].as_str().unwrap() {
        "integer" => tagged!(
            BytesOrIntegerOrNullOrRealOrString,
            T::Integer,
            value[1]
                .as_str()
                .unwrap()
                .parse::<i64>()
                .expect("host I64 outside range")
        ),
        "real" => tagged!(
            BytesOrIntegerOrNullOrRealOrString,
            T::Real,
            value[1].as_f64().unwrap()
        ),
        "text" => tagged!(
            BytesOrIntegerOrNullOrRealOrString,
            T::String,
            text(value[1].as_str().unwrap())
        ),
        "blob" => tagged!(
            BytesOrIntegerOrNullOrRealOrString,
            T::Bytes,
            bytes(
                &value[1]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
                    .collect::<Vec<_>>()
            )
        ),
        "null" => tagged!(BytesOrIntegerOrNullOrRealOrString, T::Null, ()),
        _ => panic!("unknown SQL value tag"),
    }
}

#[unsafe(no_mangle)]
extern "C" fn hosted_env_is_windows(_: RocStr) -> bool {
    false
}
#[unsafe(no_mangle)]
extern "C" fn hosted_env_var(name: UnixBytesOrUtf8OrWindowsU16s) -> HostEnvVarResult {
    let key = unsafe {
        match name.tag {
            UnixBytesOrUtf8OrWindowsU16sTag::Utf8 => {
                name.borrow_payload_utf8_unchecked().as_str().to_owned()
            }
            UnixBytesOrUtf8OrWindowsU16sTag::UnixBytes => {
                std::str::from_utf8(name.borrow_payload_unix_bytes_unchecked().as_slice())
                    .expect("Worker environment names require UTF-8")
                    .to_owned()
            }
            _ => panic!("Windows environment strings are unsupported"),
        }
    };
    let reply = rpc::<1>(json!({"name": key}));
    if let Some(value) = reply["value"].as_str() {
        unsafe {
            name.decref(&host());
        }
        let value = tagged!(
            UnixBytesOrUtf8OrWindowsU16s,
            UnixBytesOrUtf8OrWindowsU16sTag::Utf8,
            text(value)
        );
        tagged!(HostEnvVarResult, HostEnvVarResultTag::Ok, value)
    } else {
        let error = tagged!(
            EnvErrOrVarNotFound,
            EnvErrOrVarNotFoundTag::VarNotFound,
            name
        );
        tagged!(HostEnvVarResult, HostEnvVarResultTag::Err, error)
    }
}
#[unsafe(no_mangle)]
extern "C" fn hosted_stderr_line(value: RocStr) -> HostStderrLineResult {
    rpc::<2>(json!({"message": value.as_str()}));
    unsafe {
        value.decref(&host());
    }
    tagged!(HostStderrLineResult, HostStderrLineResultTag::Ok, ())
}
#[unsafe(no_mangle)]
extern "C" fn hosted_stdout_line(value: RocStr) -> HostStdoutLineResult {
    rpc::<2>(json!({"message": value.as_str()}));
    unsafe {
        value.decref(&host());
    }
    tagged!(HostStdoutLineResult, HostStdoutLineResultTag::Ok, ())
}
#[unsafe(no_mangle)]
extern "C" fn hosted_sleep_millis(milliseconds: u64) {
    let reply = rpc::<3>(json!({"milliseconds": milliseconds}));
    assert!(reply["error"].is_null(), "Worker timer effect was rejected");
}
#[unsafe(no_mangle)]
extern "C" fn hosted_unix_time_now() -> i128 {
    i128::from(
        rpc::<4>(json!({}))["milliseconds"]
            .as_i64()
            .expect("provider timestamp"),
    ) * 1_000_000
}

#[unsafe(no_mangle)]
extern "C" fn hosted_sqlite_open(
    path: HostSqliteOpenArg0,
    connections: u64,
    acquire: u64,
    busy: u64,
    cache: u64,
    journal: i64,
    sync: i64,
) -> HostSqliteOpenResult {
    let value = std::str::from_utf8(path.unix_bytes.as_slice())
        .expect("Worker SQLite aliases require UTF-8")
        .to_owned();
    unsafe {
        path.decref(&host());
    }
    sql_handle!(
        HostSqliteOpenResult,
        HostSqliteOpenResultTag,
        rpc::<10>(
            json!({"path":value,"options":[connections, acquire, busy, cache, journal as u64, sync as u64]})
        )
    )
}
#[unsafe(no_mangle)]
extern "C" fn hosted_sqlite_prepare(db: *mut u64, query: RocStr) -> HostSqlitePrepareResult {
    let reply = rpc::<11>(json!({"db": unsafe { *db }, "query": query.as_str()}));
    unsafe {
        consume_resource(db);
        query.decref(&host());
    }
    sql_handle!(HostSqlitePrepareResult, HostSqlitePrepareResultTag, reply)
}
#[unsafe(no_mangle)]
extern "C" fn hosted_sqlite_columns(statement: *mut u64) -> HostSqliteColumnsResult {
    let reply = rpc::<12>(json!({"statement": unsafe { *statement }}));
    unsafe {
        consume_resource(statement);
    }
    match reply["columns"].as_array() {
        Some(columns) => tagged!(
            HostSqliteColumnsResult,
            HostSqliteColumnsResultTag::Ok,
            list(
                &columns
                    .iter()
                    .map(|value| text(value.as_str().unwrap()))
                    .collect::<Vec<_>>()
            )
        ),
        None => tagged!(
            HostSqliteColumnsResult,
            HostSqliteColumnsResultTag::Err,
            sql_error(&reply)
        ),
    }
}
#[unsafe(no_mangle)]
extern "C" fn hosted_sqlite_start(
    statement: *mut u64,
    bindings: RocList<HostSqliteStartArg1>,
    timeout: u64,
) -> HostSqliteStartResult {
    let params: Vec<_> = bindings
        .as_slice()
        .iter()
        .map(|binding| json!({"name":binding.name.as_str(), "value":sql_value(&binding.value)}))
        .collect();
    let reply =
        rpc::<13>(json!({"statement":unsafe{*statement}, "params":params, "timeout":timeout}));
    unsafe {
        consume_resource(statement);
        decref_list_of_anon_struct2782504baf739389(bindings, &host());
    }
    sql_handle!(HostSqliteStartResult, HostSqliteStartResultTag, reply)
}
#[unsafe(no_mangle)]
extern "C" fn hosted_sqlite_next_row(
    execution: *mut u64,
    max_bytes: u64,
    allow_row: bool,
) -> HostSqliteNextRowResult {
    let reply = rpc::<14>(
        json!({"execution":unsafe{*execution}, "maxBytes":max_bytes, "allowRow":allow_row}),
    );
    unsafe {
        consume_resource(execution);
    }
    use DoneOrResultTooLargeOrRowOrRowLimitExceededTag as T;
    let state = match reply["state"].as_str() {
        Some("done") => tagged!(HostSqliteNextRowOk, T::Done, ()),
        Some("bytes") => tagged!(HostSqliteNextRowOk, T::ResultTooLarge, ()),
        Some("rows") => tagged!(HostSqliteNextRowOk, T::RowLimitExceeded, ()),
        Some("row") => tagged!(
            HostSqliteNextRowOk,
            T::Row,
            HostSqliteNextRowOkRow {
                bytes: reply["bytes"].as_u64().unwrap(),
                values: list(
                    &reply["values"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(from_sql)
                        .collect::<Vec<_>>()
                )
            }
        ),
        _ => {
            return tagged!(
                HostSqliteNextRowResult,
                HostSqliteNextRowResultTag::Err,
                sql_error(&reply)
            );
        }
    };
    tagged!(
        HostSqliteNextRowResult,
        HostSqliteNextRowResultTag::Ok,
        state
    )
}

#[unsafe(no_mangle)]
extern "C" fn hosted_sqlite_begin(db: *mut u64, mode: i64) -> HostSqliteBeginResult {
    let reply = rpc::<20>(json!({"db":unsafe{*db}, "mode":mode}));
    unsafe {
        consume_resource(db);
    }
    sql_handle!(HostSqliteBeginResult, HostSqliteBeginResultTag, reply)
}
#[unsafe(no_mangle)]
extern "C" fn hosted_sqlite_txn_prepare(
    transaction: *mut u64,
    query: RocStr,
) -> HostSqlitePrepareResult {
    let reply = rpc::<21>(json!({"transaction":unsafe{*transaction}, "query":query.as_str()}));
    unsafe {
        consume_resource(transaction);
        query.decref(&host());
    }
    sql_handle!(HostSqlitePrepareResult, HostSqlitePrepareResultTag, reply)
}
#[unsafe(no_mangle)]
extern "C" fn hosted_sqlite_txn_finish(
    transaction: *mut u64,
    commit: bool,
) -> HostSqliteTxnFinishResult {
    let reply = rpc::<22>(json!({"transaction":unsafe{*transaction}, "commit":commit}));
    unsafe {
        consume_resource(transaction);
    }
    if reply["ok"] == true {
        tagged!(
            HostSqliteTxnFinishResult,
            HostSqliteTxnFinishResultTag::Ok,
            ()
        )
    } else {
        tagged!(
            HostSqliteTxnFinishResult,
            HostSqliteTxnFinishResultTag::Err,
            sql_error(&reply)
        )
    }
}

#[unsafe(no_mangle)]
extern "C" fn hosted_request_body_read_all(
    body: *mut u64,
    limit: u64,
) -> HostRequestBodyReadAllResult {
    let reply = rpc::<30>(json!({"body":unsafe{*body}, "limit":limit}));
    unsafe {
        consume_resource(body);
    }
    if let Some(value) = reply["bytes"].as_array() {
        let value = bytes(
            &value
                .iter()
                .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap())
                .collect::<Vec<_>>(),
        );
        tagged!(
            HostRequestBodyReadAllResult,
            HostRequestBodyReadAllResultTag::Ok,
            value
        )
    } else {
        let error = match reply["kind"].as_str() {
            Some("too-large") => tagged!(
                HostRequestBodyReadAllErr,
                HostRequestBodyReadAllErrTag::TooLarge,
                HostRequestBodyReadAllErrTooLarge {
                    limit_bytes: limit,
                    received_at_least: reply["received"].as_u64().unwrap()
                }
            ),
            Some("finished") => tagged!(
                HostRequestBodyReadAllErr,
                HostRequestBodyReadAllErrTag::RequestFinished,
                ()
            ),
            Some("cancelled") => tagged!(
                HostRequestBodyReadAllErr,
                HostRequestBodyReadAllErrTag::Cancelled,
                ()
            ),
            Some("timeout") => tagged!(
                HostRequestBodyReadAllErr,
                HostRequestBodyReadAllErrTag::Timeout,
                ()
            ),
            _ => tagged!(
                HostRequestBodyReadAllErr,
                HostRequestBodyReadAllErrTag::InvalidBody,
                text(reply["error"].as_str().unwrap_or("Worker body read failed"))
            ),
        };
        tagged!(
            HostRequestBodyReadAllResult,
            HostRequestBodyReadAllResultTag::Err,
            error
        )
    }
}

#[unsafe(no_mangle)]
extern "C" fn hosted_http_send_request(
    request: HostHttpSendRequestArgs,
) -> HostHttpSendRequestResult {
    let reply = rpc::<31>(json!({
        "method":request.method, "methodExt":request.method_ext.as_str(), "uri":request.uri.as_str(),
        "headers":request.headers.as_slice().iter().map(|header| [header.name.as_str(),header.value.as_str()]).collect::<Vec<_>>(),
        "body":request.body.as_slice(), "limit":request.max_response_bytes,"timeout":request.timeout_ms,
    }));
    unsafe {
        request.decref(&host());
    }
    if let Some(status) = reply["status"].as_u64() {
        let response = HostHttpSendRequestOk {
            status: u16::try_from(status).unwrap(),
            body: bytes(
                &reply["bytes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
                    .collect::<Vec<_>>(),
            ),
            headers: list(
                &reply["headers"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| HostHttpSendRequestOkHeaders {
                        name: text(v[0].as_str().unwrap()),
                        value: text(v[1].as_str().unwrap()),
                    })
                    .collect::<Vec<_>>(),
            ),
        };
        tagged!(
            HostHttpSendRequestResult,
            HostHttpSendRequestResultTag::Ok,
            response
        )
    } else {
        let error = match reply["kind"].as_str() {
            Some("too-large") => tagged!(
                HostHttpSendRequestErrTransport,
                HostHttpSendRequestErrTransportTag::ResponseTooLarge,
                HostHttpSendRequestErrTransportResponseTooLarge {
                    limit_bytes: reply["limit"].as_u64().unwrap(),
                    received_at_least: reply["received"].as_u64().unwrap()
                }
            ),
            Some("timeout") => tagged!(
                HostHttpSendRequestErrTransport,
                HostHttpSendRequestErrTransportTag::Timeout,
                ()
            ),
            Some("cancelled") => tagged!(
                HostHttpSendRequestErrTransport,
                HostHttpSendRequestErrTransportTag::Cancelled,
                ()
            ),
            _ => tagged!(
                HostHttpSendRequestErrTransport,
                HostHttpSendRequestErrTransportTag::Other,
                text(reply["error"].as_str().unwrap_or("Worker fetch failed"))
            ),
        };
        let error = tagged!(
            HostHttpSendRequestErr,
            HostHttpSendRequestErrTag::Transport,
            error
        );
        tagged!(
            HostHttpSendRequestResult,
            HostHttpSendRequestResultTag::Err,
            error
        )
    }
}

#[unsafe(no_mangle)]
extern "C" fn worker_abi_version() -> u32 {
    crate::ABI_VERSION
}
#[unsafe(no_mangle)]
extern "C" fn worker_input_ptr() -> *mut u8 {
    ptr::addr_of_mut!(INPUT).cast()
}
#[unsafe(no_mangle)]
extern "C" fn worker_input_capacity() -> usize {
    LIMIT
}
#[unsafe(no_mangle)]
extern "C" fn worker_output_ptr() -> *const u8 {
    unsafe { (&*ptr::addr_of!(OUTPUT)).as_ptr() }
}
#[unsafe(no_mangle)]
extern "C" fn worker_output_len() -> usize {
    unsafe { (&*ptr::addr_of!(OUTPUT)).len() }
}
#[unsafe(no_mangle)]
extern "C" fn worker_live_bytes() -> usize {
    unsafe { LIVE_BYTES }
}
#[unsafe(no_mangle)]
extern "C" fn worker_live_allocations() -> usize {
    unsafe { LIVE_COUNT }
}

fn request(value: &Value) -> RespondForHostArg0 {
    let mut request: RespondForHostArg0 = unsafe { mem::zeroed() };
    request.body_limit_bytes = unsafe { BODY_LIMIT };
    request.body_handle = resource(value["body"].as_u64().expect("body resource"));
    request.target_path = text(value["path"].as_str().expect("request path"));
    request.target_query = text(value["query"].as_str().unwrap_or(""));
    request.target_query_present = value["query"].is_string();
    request.target_tag = 0;
    request.method = value["method"].as_u64().expect("request method") as u8;
    request.method_ext = text(value["methodExt"].as_str().unwrap_or(""));
    request.headers = list(
        &value["headers"]
            .as_array()
            .expect("request headers")
            .iter()
            .map(|header| RespondForHostArg0Headers {
                name: text(header[0].as_str().unwrap()),
                value: text(header[1].as_str().unwrap()),
            })
            .collect::<Vec<_>>(),
    );
    request
}
fn outcome(mut outcome: InternalServerOutcomeToHost) -> Value {
    unsafe {
        let output = match outcome.tag {
            InternalServerOutcomeToHostTag::Ordinary => {
                let response = outcome.borrow_payload_ordinary_unchecked();
                assert!(
                    response.body.len() <= 1024 * 1024,
                    "Worker finite response exceeds 1 MiB"
                );
                json!({"kind":"response", "status":response.status, "headers":response.headers.as_slice().iter().map(|header|[header.name.as_str(), header.value.as_str()]).collect::<Vec<_>>(), "body":response.body.as_slice()})
            }
            InternalServerOutcomeToHostTag::Stream => {
                let source = outcome.take_payload_stream_unchecked();
                let streams = &mut *ptr::addr_of_mut!(STREAMS);
                assert!(streams.len() < STREAM_LIMIT, "stream capacity exhausted");
                let id = NEXT_STREAM;
                NEXT_STREAM = NEXT_STREAM.checked_add(1).unwrap();
                streams.insert(id, source);
                return json!({"kind":"stream", "id":id});
            }
            InternalServerOutcomeToHostTag::File => {
                panic!("file outcomes require a declared asset binding")
            }
        };
        outcome.decref(&host());
        output
    }
}
#[unsafe(no_mangle)]
extern "C" fn worker_invoke(operation: u32, length: usize) {
    unsafe {
        assert!(
            !ACTIVE && length <= LIMIT,
            "invalid or overlapping Worker invocation"
        );
        ACTIVE = true;
        let input = std::slice::from_raw_parts(ptr::addr_of!(INPUT).cast::<u8>(), length);
        let value: Value = serde_json::from_slice(input).expect("invalid Worker request envelope");
        let output = match operation {
            0 => {
                assert!(!INITIALIZED, "already initialized");
                let mut result = roc_init_for_host();
                match result.tag {
                    InitForHostResultTag::Ok => {
                        let initialized = result.take_payload_ok_unchecked();
                        CONTEXT = initialized.context;
                        INITIALIZED = true;
                        let config = initialized.config;
                        let valid = config.body_max_bytes > 0
                            && config.body_max_bytes <= 1024 * 1024
                            && config.request_target_max_bytes > 0
                            && config.request_target_max_bytes <= 8192
                            && config.request_header_max_bytes > 0
                            && config.request_header_max_bytes <= 32768
                            && config.request_header_max_fields > 0
                            && config.request_header_max_fields <= 100
                            && config.sse_max_event_bytes > 0
                            && config.sse_max_event_bytes <= 1024 * 1024
                            && config.sse_max_streams > 0;
                        BODY_LIMIT = config.body_max_bytes;
                        STREAM_LIMIT = usize::from(config.sse_max_streams).min(128);
                        let limits = json!({"bodyBytes":config.body_max_bytes,"targetBytes":config.request_target_max_bytes,"headerBytes":config.request_header_max_bytes,"headerFields":config.request_header_max_fields,"eventBytes":config.sse_max_event_bytes});
                        config.decref(&host());
                        if valid {
                            json!({"ok":true,"limits":limits})
                        } else {
                            json!({"error":"Server.Config exceeds the Worker body/header/target/SSE capability profile; use positive limits within the documented bounds"})
                        }
                    }
                    InitForHostResultTag::Err => {
                        json!({"error":"Roc init failed", "code":*result.borrow_payload_err_unchecked()})
                    }
                }
            }
            1 => {
                assert!(INITIALIZED, "not initialized");
                incref_box(CONTEXT, 1);
                outcome(roc_respond_for_host(request(&value), CONTEXT))
            }
            2 => {
                let id = value["id"].as_u64().unwrap() as u32;
                let source = (&mut *ptr::addr_of_mut!(STREAMS))
                    .remove(&id)
                    .expect("invalid stream");
                let mut step = roc_sse_advance_for_host(source, 0);
                match step.tag {
                    SseStepToHostTag::EmitToHost => {
                        let step = step.take_payload_emit_to_host_unchecked();
                        assert!(
                            step.item.len() <= 1024 * 1024,
                            "Worker SSE event exceeds 1 MiB"
                        );
                        let output = json!({"kind":"emit", "body":step.item.as_slice(), "wait":step.wait_millis});
                        step.item.decref(&host());
                        (&mut *ptr::addr_of_mut!(STREAMS)).insert(id, step.source);
                        output
                    }
                    SseStepToHostTag::WaitToHost => {
                        let step = step.take_payload_wait_to_host_unchecked();
                        (&mut *ptr::addr_of_mut!(STREAMS)).insert(id, step.source);
                        json!({"kind":"wait", "wait":step.wait_millis})
                    }
                    SseStepToHostTag::EndToHost => json!({"kind":"end"}),
                    SseStepToHostTag::ErrorToHost => {
                        let error = step.take_payload_error_to_host_unchecked();
                        let output = json!({"error":error.as_str()});
                        error.decref(&host());
                        output
                    }
                }
            }
            3 => {
                let id = value["id"].as_u64().unwrap() as u32;
                if let Some(source) = (&mut *ptr::addr_of_mut!(STREAMS)).remove(&id) {
                    roc_sse_drop_source_for_host(source);
                }
                json!({"ok":true})
            }
            4 => {
                for (_, source) in mem::take(&mut *ptr::addr_of_mut!(STREAMS)) {
                    roc_sse_drop_source_for_host(source);
                }
                if INITIALIZED {
                    let reason = ShutdownForHostArg0 {
                        tag: 0,
                        detail: text("Worker context teardown"),
                    };
                    let result = roc_shutdown_for_host(reason, CONTEXT);
                    result.decref(&host());
                    CONTEXT = ptr::null_mut();
                    INITIALIZED = false;
                }
                json!({"ok":true})
            }
            _ => panic!("unsupported Worker invocation"),
        };
        *ptr::addr_of_mut!(OUTPUT) = serde_json::to_vec(&output).unwrap();
        ACTIVE = false;
    }
}
