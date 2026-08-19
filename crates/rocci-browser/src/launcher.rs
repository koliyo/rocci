use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use crate::Result;

const LAUNCHER_HTML: &str = r#"<!doctype html>
<html lang="en" data-rocci-browser-launcher>
<head>
<meta charset="utf-8">
<title>rocci-browser</title>
<style>
body{margin:0;font:15px/1.4 system-ui,sans-serif;color:#d7dae0;background:#21252b;padding:4rem 1.5rem}
p{max-width:36rem;color:#9da5b4}
kbd{font:12px ui-monospace,monospace;border:1px solid #3e4451;border-radius:4px;padding:1px 6px}
</style>
</head>
<body>
<h1>rocci-browser</h1>
<p>Press <kbd>Cmd</kbd>+<kbd>P</kbd> (or <kbd>Ctrl</kbd>+<kbd>P</kbd>) to pick a target. Enter opens it; Tab lists documents.</p>
</body>
</html>
"#;

pub struct Launcher {
    pub origin: String,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

pub fn spawn_launcher() -> Result<Launcher> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    let origin = format!("http://{addr}");
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = shutdown.clone();
    let thread = thread::spawn(move || {
        while !flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 1024];
                    let _ = stream.read(&mut buf);
                    let body = LAUNCHER_HTML.as_bytes();
                    let _ = write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(body);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(_) => break,
            }
        }
    });
    Ok(Launcher {
        origin,
        shutdown,
        thread: Some(thread),
    })
}

impl Drop for Launcher {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Launcher {
    pub fn origin(&self) -> &str {
        &self.origin
    }
}

pub fn launcher_html() -> &'static str {
    LAUNCHER_HTML
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{BufReader, Read, Write},
        net::TcpStream,
        time::Duration,
    };

    #[test]
    fn launcher_serves_host_page() {
        let launcher = spawn_launcher().unwrap();
        assert!(launcher.origin.starts_with("http://127.0.0.1:"));
        let host = launcher.origin.trim_start_matches("http://");
        let mut stream = TcpStream::connect(host).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        write!(
            stream,
            "GET / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut body = String::new();
        BufReader::new(stream).read_to_string(&mut body).unwrap();
        assert!(body.contains("data-rocci-browser-launcher"));
        assert!(body.contains("Cmd"));
    }

    #[test]
    fn html_is_a_host_launcher() {
        let html = launcher_html();
        assert!(html.contains("data-rocci-browser-launcher"));
        assert!(html.contains("Cmd"));
    }
}
