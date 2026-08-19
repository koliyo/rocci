use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Result;
use rocci_browser::{Host, Opened, Paths, Session, SessionTable, overlay, spawn_launcher};
use rocci_desktop::{PreviewEvent, PreviewOptions, PreviewSink};
use serde::Deserialize;
use serde_json::json;

pub fn run() -> Result<()> {
    let paths = Paths::from_env()?;
    let host = Host::connect(paths)?;
    let launcher = spawn_launcher()?;
    let host = Arc::new(Mutex::new(host));
    let sessions = Arc::new(Mutex::new(SessionTable::default()));
    let on_ipc = {
        let host = host.clone();
        let sessions = sessions.clone();
        Arc::new(move |message: &str, sink: Arc<dyn PreviewSink>| {
            handle_ipc(message, host.clone(), sessions.clone(), sink);
        })
    };
    rocci_desktop::preview(PreviewOptions {
        url: launcher.origin().to_string(),
        title: "rocci-browser".into(),
        state_key: Some("browser".into()),
        extra_initialization_script: Some(overlay::initialization_script()),
        on_ipc: Some(on_ipc),
        picker: true,
        ..PreviewOptions::default()
    })?;
    drop(launcher);
    drop(host);
    drop(sessions);
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenIpc {
    adapter_id: String,
    root: String,
    #[serde(default)]
    document: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListIpc {
    adapter_id: String,
    root: String,
}

fn handle_ipc(
    message: &str,
    host: Arc<Mutex<Host>>,
    sessions: Arc<Mutex<SessionTable>>,
    sink: Arc<dyn PreviewSink>,
) {
    let message = message.trim();
    if message == "browser:catalog" {
        thread::spawn(move || {
            let mut host = host.lock().unwrap();
            let script = match host.probe_targets() {
                Ok(targets) => {
                    let rows: Vec<_> = targets
                        .iter()
                        .map(|target| {
                            json!({
                                "id": target.id,
                                "path": target.path,
                                "adapterId": target.adapter_id,
                                "label": target.label,
                            })
                        })
                        .collect();
                    format!(
                        "window.__rocciBrowser&&window.__rocciBrowser.setTargets({})",
                        serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into())
                    )
                }
                Err(error) => format!(
                    "window.__rocciBrowser&&window.__rocciBrowser.setDocuments([],{})",
                    serde_json::to_string(&error.to_string())
                        .unwrap_or_else(|_| "\"error\"".into())
                ),
            };
            sink.send(PreviewEvent::Evaluate(script));
        });
        return;
    }
    if let Some(raw) = message.strip_prefix("browser:list:") {
        let request: ListIpc = match serde_json::from_str(raw) {
            Ok(request) => request,
            Err(_) => return,
        };
        thread::spawn(move || {
            let mut host = host.lock().unwrap();
            let (documents, reason) = host
                .documents_or_reason(&request.adapter_id, &request.root)
                .unwrap_or_else(|error| (Vec::new(), Some(error.to_string())));
            let rows: Vec<_> = documents
                .iter()
                .map(|document| {
                    json!({
                        "id": document.id,
                        "title": document.title,
                        "path": document.path,
                        "route": document.route,
                    })
                })
                .collect();
            let script = format!(
                "window.__rocciBrowser&&window.__rocciBrowser.setDocuments({},{})",
                serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into()),
                serde_json::to_string(&reason).unwrap_or_else(|_| "null".into())
            );
            sink.send(PreviewEvent::Evaluate(script));
        });
        return;
    }
    if let Some(raw) = message.strip_prefix("browser:open:") {
        let request: OpenIpc = match serde_json::from_str(raw) {
            Ok(request) => request,
            Err(_) => return,
        };
        thread::spawn(move || {
            let mut host = host.lock().unwrap();
            match host.open_target(
                &request.adapter_id,
                &request.root,
                request.document.as_deref(),
            ) {
                Ok(opened) => {
                    sessions
                        .lock()
                        .unwrap()
                        .record(Session::from_opened(&opened));
                    sink.send(navigate_event(&opened));
                }
                Err(error) => sink.send(PreviewEvent::Evaluate(format!(
                    "console.error({})",
                    serde_json::to_string(&error.to_string())
                        .unwrap_or_else(|_| "\"open failed\"".into())
                ))),
            }
        });
    }
}

fn navigate_event(opened: &Opened) -> PreviewEvent {
    PreviewEvent::Navigate {
        url: opened.url.clone(),
        title: opened.title.clone(),
        inspector_url: opened.inspector_url.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocci_browser::{Opened, Target};

    #[test]
    fn navigate_event_forwards_inspector_url() {
        let opened = Opened {
            url: "http://127.0.0.1:9/".into(),
            title: "Hello".into(),
            inspector_url: Some("http://127.0.0.1:9/inspect".into()),
            target: Target {
                id: "fixture".into(),
                path: "/tmp/fixture".into(),
                adapter_id: "fixture".into(),
                label: "Fixture".into(),
                detail: None,
            },
            document: None,
        };
        match navigate_event(&opened) {
            PreviewEvent::Navigate {
                url, inspector_url, ..
            } => {
                assert_eq!(url, opened.url);
                assert_eq!(inspector_url, opened.inspector_url);
            }
            other => panic!("{other:?}"),
        }
    }
}
