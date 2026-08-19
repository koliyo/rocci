use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    Error, Result,
    discovery::PluginSpec,
    protocol::{
        CAP_LIST, CAP_OPEN, CAP_PROBE, CAP_SHUTDOWN, Document, InitializeParams, InitializeResult,
        ListDocumentsParams, ListDocumentsResult, OpenParams, OpenResult, PROTOCOL_VERSION,
        ProbeParams, ProbeResult, RpcRequest, RpcResponse,
    },
};

pub struct AdapterClient {
    pub spec: PluginSpec,
    pub initialize: InitializeResult,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl AdapterClient {
    pub fn spawn(spec: PluginSpec, bin: &Path) -> Result<Self> {
        let mut command = Command::new(bin);
        command
            .args(&spec.argv)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        let mut child = command.spawn().map_err(|error| {
            Error::message(format!("failed to spawn plugin {}: {error}", spec.id))
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::message("adapter stdin missing"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::message("adapter stdout missing"))?;
        let mut client = Self {
            spec,
            initialize: InitializeResult {
                protocol_version: PROTOCOL_VERSION,
                adapter_id: String::new(),
                capabilities: Vec::new(),
            },
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };
        let init: InitializeResult = client.call(
            "initialize",
            InitializeParams {
                protocol_version: PROTOCOL_VERSION,
            },
        )?;
        client.initialize = init;
        Ok(client)
    }

    pub fn adapter_id(&self) -> &str {
        &self.initialize.adapter_id
    }

    pub fn probe(&mut self, path: &str) -> Result<ProbeResult> {
        if !self.initialize.supports(CAP_PROBE) {
            return Ok(ProbeResult {
                claimed: false,
                label: None,
                detail: None,
            });
        }
        self.call(
            "probe",
            ProbeParams {
                path: path.to_string(),
            },
        )
    }

    pub fn list_documents(&mut self, root: &str) -> Result<ListDocumentsResult> {
        if !self.initialize.supports(CAP_LIST) {
            return Ok(ListDocumentsResult {
                documents: Vec::new(),
            });
        }
        self.call(
            CAP_LIST,
            ListDocumentsParams {
                root: root.to_string(),
            },
        )
    }

    pub fn open(&mut self, params: OpenParams) -> Result<OpenResult> {
        if !self.initialize.supports(CAP_OPEN) {
            return Err(Error::message(format!(
                "adapter {} does not support open",
                self.adapter_id()
            )));
        }
        self.call(CAP_OPEN, params)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        if self.initialize.supports(CAP_SHUTDOWN) {
            let _: Value = self.call("shutdown", Value::Object(Default::default()))?;
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        Ok(())
    }

    fn call<P: Serialize, R: DeserializeOwned>(&mut self, method: &str, params: P) -> Result<R> {
        let id = self.next_id;
        self.next_id += 1;
        let request = RpcRequest {
            jsonrpc: "2.0".into(),
            id,
            method: method.into(),
            params,
        };
        let mut line = serde_json::to_string(&request)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes())?;
        self.stdin.flush()?;
        let mut response_line = String::new();
        let read = self.stdout.read_line(&mut response_line)?;
        if read == 0 {
            return Err(Error::message(format!(
                "adapter {} closed stdout during {method}",
                self.spec.id
            )));
        }
        let response: RpcResponse<R> = serde_json::from_str(response_line.trim_end())?;
        if let Some(error) = response.error {
            return Err(Error::message(
                error
                    .message
                    .unwrap_or_else(|| format!("adapter {method} failed")),
            ));
        }
        response
            .result
            .ok_or_else(|| Error::message(format!("adapter {method} returned no result")))
    }
}

impl Drop for AdapterClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

pub fn documents_reason(documents: &[Document]) -> Option<String> {
    documents
        .is_empty()
        .then(|| "adapter returned no documents".to_string())
}
