use std::{
    env,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

use roc::{Backend, Error, Result, RunningBackend, WindowId};

const READY_PREFIX: &str = "ROC_BACKEND_READY ";

#[derive(Debug)]
pub struct PythonBackend {
    interpreter: String,
    script: PathBuf,
    assets: PathBuf,
}

impl Default for PythonBackend {
    fn default() -> Self {
        let interpreter = env::var("ROC_PYTHON").unwrap_or_else(|_| "python3".into());
        let (script, assets) = locate_python_resources();
        Self {
            interpreter,
            script,
            assets,
        }
    }
}

impl Backend for PythonBackend {
    fn name(&self) -> &str {
        "python"
    }

    fn start(&self) -> Result<Box<dyn RunningBackend>> {
        let mut child = Command::new(&self.interpreter)
            .arg(&self.script)
            .arg("--assets")
            .arg(&self.assets)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| {
                Error::backend(format!(
                    "failed to start Python backend with {} (override with ROC_PYTHON): {error}",
                    self.interpreter
                ))
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::backend("Python stdout was not piped"))?;
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(stdout)
                .read_line(&mut line)
                .map(|_| line)
                .map_err(|error| {
                    Error::backend(format!("failed to read Python readiness message: {error}"))
                });
            let _ = sender.send(result);
        });

        let line = match receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(result) => result?,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Error::backend(
                    "Python backend did not become ready within 10 seconds",
                ));
            }
        };
        let bootstrap_url = line
            .trim()
            .strip_prefix(READY_PREFIX)
            .ok_or_else(|| Error::backend("invalid Python readiness message"))?
            .to_owned();
        if !bootstrap_url.starts_with("http://127.0.0.1:") {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::backend(
                "backend returned a non-loopback bootstrap URL",
            ));
        }
        let origin = bootstrap_url
            .split("/_roc/")
            .next()
            .unwrap_or(&bootstrap_url)
            .to_owned();

        Ok(Box::new(ProcessBackend {
            name: "python",
            origin,
            bootstrap_url,
            child: Some(child),
        }))
    }
}

struct ProcessBackend {
    name: &'static str,
    origin: String,
    bootstrap_url: String,
    child: Option<Child>,
}

impl RunningBackend for ProcessBackend {
    fn origin(&self) -> &str {
        &self.origin
    }

    fn attach_window(&self, _window: &WindowId, _start_url: &str) -> Result<String> {
        Ok(self.bootstrap_url.clone())
    }

    fn shutdown(&mut self) {
        if let Some(mut child) = self.child.take() {
            if let Err(error) = child.kill()
                && error.kind() != std::io::ErrorKind::InvalidInput
            {
                tracing::warn!(backend = self.name, %error, "failed to stop backend process");
            }
            let _ = child.wait();
        }
    }
}

impl Drop for ProcessBackend {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn locate_python_resources() -> (PathBuf, PathBuf) {
    if let Ok(executable) = env::current_exe()
        && let Some(contents) = executable.parent().and_then(Path::parent)
    {
        let resources = contents.join("Resources");
        let script = resources.join("python/backend.py");
        if script.is_file() {
            return (script, resources.join("assets"));
        }
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    (root.join("backends/python/backend.py"), root.join("assets"))
}
