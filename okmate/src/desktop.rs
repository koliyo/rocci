use std::thread;

use anyhow::{Context, Result};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::{dpi::LogicalSize, window::WindowBuilder};
use wry::{WebContext, WebViewBuilder, http::Request};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpcMessage {
    PickFolder,
    Home,
}

impl IpcMessage {
    pub fn parse(message: &str) -> Option<Self> {
        match message.trim() {
            "pick-folder" => Some(Self::PickFolder),
            "home" => Some(Self::Home),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
enum DesktopEvent {
    PickFolder,
    PickFolderResult(Option<String>),
    Home,
}

pub fn pick_folder_result_script(path: Option<&str>) -> String {
    let detail = match path {
        Some(path) => format!(
            r#"{{"path":{}}}"#,
            serde_json::to_string(path).unwrap_or_else(|_| "null".into())
        ),
        None => r#"{"path":null}"#.to_string(),
    };
    format!(r#"window.dispatchEvent(new CustomEvent("okmate-pick-folder",{{detail:{detail}}}));"#)
}

pub fn home_url(bound: impl std::fmt::Display) -> String {
    format!("http://{bound}/")
}

pub fn run(options: crate::preview::ViewOptions) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = tx.send(Err(anyhow::anyhow!(error)));
                return;
            }
        };
        let result = runtime.block_on(crate::preview::serve_ready(options, tx.clone()));
        if let Err(error) = result {
            let _ = tx.send(Err(error));
        }
    });
    let ready = rx
        .recv()
        .context("preview server thread exited before binding")?;
    let ready = ready?;
    open_window(&ready.initial_url, &ready.home_url)
}

struct Live {
    webview: wry::WebView,
    _context: WebContext,
}

fn open_window(initial_url: &str, home_url: &str) -> Result<()> {
    let mut event_loop = EventLoopBuilder::<DesktopEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title("Okmate")
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)
        .context("failed to create Okmate window")?;

    let mut context = WebContext::new(None);
    let ipc_proxy = proxy.clone();
    let webview = WebViewBuilder::new_with_web_context(&mut context)
        .with_url(initial_url)
        .with_ipc_handler(
            move |request: Request<String>| match IpcMessage::parse(request.body()) {
                Some(IpcMessage::PickFolder) => {
                    let _ = ipc_proxy.send_event(DesktopEvent::PickFolder);
                }
                Some(IpcMessage::Home) => {
                    let _ = ipc_proxy.send_event(DesktopEvent::Home);
                }
                None => {}
            },
        )
        .build(&window)
        .context("failed to create Okmate webview")?;

    let live = Live {
        webview,
        _context: context,
    };
    let home = home_url.to_string();
    let pick_proxy = proxy;

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(DesktopEvent::PickFolder) => {
                start_pick_folder(pick_proxy.clone());
            }
            Event::UserEvent(DesktopEvent::PickFolderResult(path)) => {
                let _ = live
                    .webview
                    .evaluate_script(&pick_folder_result_script(path.as_deref()));
            }
            Event::UserEvent(DesktopEvent::Home) => {
                let _ = live.webview.load_url(&home);
            }
            _ => {}
        }
    });
    Ok(())
}

fn start_pick_folder(proxy: EventLoopProxy<DesktopEvent>) {
    thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(_) => {
                let _ = proxy.send_event(DesktopEvent::PickFolderResult(None));
                return;
            }
        };
        let picked = runtime.block_on(async {
            rfd::AsyncFileDialog::new()
                .set_title("Choose knowledge folder")
                .pick_folder()
                .await
        });
        let path = picked.map(|handle| handle.path().to_string_lossy().into_owned());
        let _ = proxy.send_event(DesktopEvent::PickFolderResult(path));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ipc_pick_folder_and_home() {
        assert_eq!(
            IpcMessage::parse("pick-folder"),
            Some(IpcMessage::PickFolder)
        );
        assert_eq!(IpcMessage::parse("  home  "), Some(IpcMessage::Home));
        assert_eq!(IpcMessage::parse("osascript"), None);
        assert_eq!(IpcMessage::parse("pick-folder-http"), None);
    }

    #[test]
    fn pick_folder_script_json_escapes_paths() {
        let script = pick_folder_result_script(Some(r#"C:\tmp\"quotes""#));
        assert!(script.contains("okmate-pick-folder"), "{script}");
        assert!(script.contains(r#"C:\\tmp\\\"quotes\""#), "{script}");
        let cancelled = pick_folder_result_script(None);
        assert!(cancelled.contains(r#""path":null"#), "{cancelled}");
    }

    #[test]
    fn home_url_is_origin_root() {
        assert_eq!(home_url("127.0.0.1:8000"), "http://127.0.0.1:8000/");
    }

    #[test]
    #[ignore = "opens a native window"]
    fn window_smoke() {}
}
