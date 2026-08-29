//! Linked guest with 0.16 export names (`roc_*_for_host`).

use anyhow::{Context, Result};

use crate::abi::{OrdinaryResponse, OutcomeToHost, ServerHeader, ServerRequest};
use crate::guest::RocGuest;

const HELLO_WEB_WAT: &str = include_str!("hello_web.wat");

struct GuestState {
    status: u16,
    body: Vec<u8>,
}

pub struct WasmRocGuest {
    engine: wasmtime::Engine,
    module: wasmtime::Module,
    initialized: bool,
}

pub fn hello_web_component_bytes() -> Result<Vec<u8>> {
    wat::parse_str(HELLO_WEB_WAT).context("parse hello-web WAT")
}

impl WasmRocGuest {
    pub fn hello_web() -> Result<Self> {
        Self::from_wat(HELLO_WEB_WAT)
    }

    pub fn from_wat(wat: &str) -> Result<Self> {
        let wasm = wat::parse_str(wat).context("parse guest WAT")?;
        Self::from_bytes(&wasm)
    }

    pub fn from_bytes(wasm: &[u8]) -> Result<Self> {
        let engine =
            wasmtime::Engine::new(&wasmtime::Config::new()).map_err(crate::probe::wasmtime_err)?;
        let module = wasmtime::Module::new(&engine, wasm).map_err(crate::probe::wasmtime_err)?;
        Ok(Self {
            engine,
            module,
            initialized: false,
        })
    }

    fn call_export(&mut self, export: &str) -> Result<(u16, Vec<u8>)> {
        let mut store = wasmtime::Store::new(
            &self.engine,
            GuestState {
                status: 500,
                body: Vec::new(),
            },
        );
        let mut linker = wasmtime::Linker::new(&self.engine);
        linker
            .func_wrap(
                "host",
                "hosted_emit_ordinary",
                |mut caller: wasmtime::Caller<'_, GuestState>, status: i32, ptr: i32, len: i32| {
                    let Some(wasmtime::Extern::Memory(memory)) = caller.get_export("memory") else {
                        return;
                    };
                    let mut buf = vec![0u8; len.max(0) as usize];
                    if memory.read(&caller, ptr.max(0) as usize, &mut buf).is_ok() {
                        caller.data_mut().status = status.max(0) as u16;
                        caller.data_mut().body = buf;
                    }
                },
            )
            .map_err(crate::probe::wasmtime_err)?;
        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(crate::probe::wasmtime_err)?;
        let func = instance
            .get_typed_func::<(), i32>(&mut store, export)
            .map_err(crate::probe::wasmtime_err)
            .with_context(|| format!("missing export {export}"))?;
        let code = func
            .call(&mut store, ())
            .map_err(crate::probe::wasmtime_err)?;
        if code != 0 {
            anyhow::bail!("{export} returned {code}");
        }
        let state = store.into_data();
        Ok((state.status, state.body))
    }
}

impl RocGuest for WasmRocGuest {
    fn init(&mut self) {
        self.call_export("roc_init_for_host")
            .expect("roc_init_for_host");
        self.initialized = true;
    }

    fn respond(&mut self, _request: &ServerRequest) -> OutcomeToHost {
        let (status, body) = self
            .call_export("roc_respond_for_host")
            .expect("roc_respond_for_host");
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
        let _ = self.call_export("roc_shutdown_for_host");
        self.initialized = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::IncomingRequest;
    use crate::handle::Adapter;

    #[tokio::test(flavor = "current_thread")]
    async fn linked_hello_web_get_root() {
        let mut adapter = Adapter::new(WasmRocGuest::hello_web().unwrap());
        let response = adapter
            .handle(IncomingRequest {
                method: "GET".into(),
                path: "/".into(),
                headers: vec![],
                body: Vec::new(),
            })
            .await
            .unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(
            std::str::from_utf8(&response.body).unwrap(),
            "<!doctype html><html><body>hello-web</body></html>"
        );
    }
}
