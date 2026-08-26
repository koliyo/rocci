use tao::event_loop::EventLoopProxy;

use crate::events::{PreviewEvent, ShellEvent};

pub fn pick_folder_result_script(path: Option<&str>) -> String {
    let detail = match path {
        Some(path) => format!(
            r#"{{"path":{}}}"#,
            serde_json::to_string(path).unwrap_or_else(|_| "null".into())
        ),
        None => r#"{"path":null}"#.to_string(),
    };
    format!(r#"window.dispatchEvent(new CustomEvent("h35-pick-folder",{{detail:{detail}}}));"#)
}

pub fn start_pick_folder(proxy: EventLoopProxy<ShellEvent>) {
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(_) => {
                let _ = proxy.send_event(ShellEvent::Preview(PreviewEvent::PickFolderResult(None)));
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
        let _ = proxy.send_event(ShellEvent::Preview(PreviewEvent::PickFolderResult(path)));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_folder_script_json_escapes_paths() {
        let script = pick_folder_result_script(Some(r#"C:\tmp\"quotes""#));
        assert!(script.contains(r#"h35-pick-folder"#), "{script}");
        assert!(script.contains(r#"C:\\tmp\\\"quotes\""#), "{script}");
        let cancelled = pick_folder_result_script(None);
        assert!(cancelled.contains(r#""path":null"#), "{cancelled}");
    }
}
