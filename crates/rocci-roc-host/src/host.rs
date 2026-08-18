use anyhow::{Context, Result, bail};
use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::cache::TwoTierCache;
use crate::fingerprint::InputFingerprint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostChoice {
    #[default]
    Auto,
    Native,
    Wasm,
}

impl HostChoice {
    pub fn from_env() -> Option<Self> {
        env::var("ROCCI_HOST")
            .ok()
            .and_then(|v| match v.to_ascii_lowercase().as_str() {
                "native" => Some(Self::Native),
                "wasm" => Some(Self::Wasm),
                "auto" => Some(Self::Auto),
                _ => None,
            })
    }

    pub fn resolve(self) -> Self {
        if self == Self::Auto {
            Self::from_env().unwrap_or(Self::Auto)
        } else {
            self
        }
    }
}

impl std::str::FromStr for HostChoice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "native" => Ok(Self::Native),
            "wasm" => Ok(Self::Wasm),
            other => bail!("unknown host '{other}'; expected 'auto', 'native', or 'wasm'"),
        }
    }
}

impl std::fmt::Display for HostChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Native => write!(f, "native"),
            Self::Wasm => write!(f, "wasm"),
        }
    }
}

pub struct NativeHost {
    pub cache: TwoTierCache,
}

impl NativeHost {
    pub fn new(cache: TwoTierCache) -> Self {
        Self { cache }
    }

    pub fn default() -> Self {
        Self::new(TwoTierCache::default())
    }

    pub fn compile_or_cached(
        &self,
        workspace: &Path,
        compile_hash: &str,
        fingerprints: &[InputFingerprint],
    ) -> Result<(PathBuf, bool)> {
        let target = format!("native:{}", env::consts::ARCH);
        if let Some(cached_bin) = self.cache.lookup_renderer(compile_hash, &target) {
            return Ok((cached_bin, false));
        }

        let apply_bin = workspace.join("apply");
        let output = Command::new("roc")
            .arg("build")
            .arg("main.roc")
            .arg("--opt=dev")
            .arg(format!("--output={}", apply_bin.display()))
            .current_dir(workspace)
            .output()
            .context("failed to invoke roc build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            bail!("roc build failed:\n{stdout}{stderr}");
        }

        if !apply_bin.is_file() {
            bail!("roc build did not create {}", apply_bin.display());
        }

        let bytes = std::fs::read(&apply_bin)
            .with_context(|| format!("failed to read {}", apply_bin.display()))?;
        let stored_path = self
            .cache
            .store_renderer(compile_hash, &target, &bytes, fingerprints)?;
        Ok((stored_path, true))
    }

    pub fn run_apply(&self, apply_bin: &Path, workspace: &Path, staging: &Path) -> Result<Output> {
        Command::new(apply_bin)
            .current_dir(workspace)
            .env("ROCDOWN_STAGING", staging)
            .output()
            .with_context(|| format!("failed to run {}", apply_bin.display()))
    }
}

#[cfg(feature = "wasmtime")]
pub struct WasmHost {
    engine: wasmtime::Engine,
    module: wasmtime::Module,
}

#[cfg(feature = "wasmtime")]
impl WasmHost {
    pub fn compile_or_cached(
        cache: &TwoTierCache,
        workspace: &Path,
        compile_hash: &str,
        fingerprints: &[InputFingerprint],
    ) -> Result<(PathBuf, bool)> {
        let target = "wasm32";
        if let Some(cached_bin) = cache.lookup_renderer(compile_hash, target) {
            return Ok((cached_bin, false));
        }

        let wasm_file = workspace.join("components.wasm");
        let output = Command::new("roc")
            .arg("build")
            .arg("main.roc")
            .arg("--target=wasm32")
            .arg("--opt=dev")
            .arg(format!("--output={}", wasm_file.display()))
            .current_dir(workspace)
            .output()
            .context("failed to invoke roc build for wasm32")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            bail!("roc build --target=wasm32 failed:\n{stdout}{stderr}");
        }

        if !wasm_file.is_file() {
            bail!("roc build did not create {}", wasm_file.display());
        }

        let bytes = std::fs::read(&wasm_file)
            .with_context(|| format!("failed to read {}", wasm_file.display()))?;
        let stored_path = cache.store_renderer(compile_hash, target, &bytes, fingerprints)?;
        Ok((stored_path, true))
    }

    pub fn from_bytes(wasm_bytes: &[u8]) -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_memory64(false);
        let engine = wasmtime::Engine::new(&config)?;
        let module = wasmtime::Module::new(&engine, wasm_bytes)?;
        Ok(Self { engine, module })
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("failed to read wasm file {}", path.display()))?;
        Self::from_bytes(&bytes)
    }

    pub fn render(&self, view_json: &str, article_html: &str) -> Result<String> {
        struct HostState {
            view: String,
            article: String,
            output: String,
        }

        let mut store = wasmtime::Store::new(
            &self.engine,
            HostState {
                view: view_json.to_string(),
                article: article_html.to_string(),
                output: String::new(),
            },
        );

        let mut linker = wasmtime::Linker::new(&self.engine);

        linker.func_wrap(
            "env",
            "roc_host_get_view_len",
            |caller: wasmtime::Caller<'_, HostState>| -> i32 { caller.data().view.len() as i32 },
        )?;
        linker.func_wrap(
            "env",
            "roc_host_get_article_len",
            |caller: wasmtime::Caller<'_, HostState>| -> i32 { caller.data().article.len() as i32 },
        )?;
        linker.func_wrap(
            "env",
            "roc_host_read_view",
            |mut caller: wasmtime::Caller<'_, HostState>, ptr: i32| -> i32 {
                let bytes = caller.data().view.as_bytes().to_vec();
                let Some(wasmtime::Extern::Memory(mem)) = caller.get_export("memory") else {
                    return -1;
                };
                if mem.write(&mut caller, ptr as usize, &bytes).is_err() {
                    return -1;
                }
                bytes.len() as i32
            },
        )?;
        linker.func_wrap(
            "env",
            "roc_host_read_article",
            |mut caller: wasmtime::Caller<'_, HostState>, ptr: i32| -> i32 {
                let bytes = caller.data().article.as_bytes().to_vec();
                let Some(wasmtime::Extern::Memory(mem)) = caller.get_export("memory") else {
                    return -1;
                };
                if mem.write(&mut caller, ptr as usize, &bytes).is_err() {
                    return -1;
                }
                bytes.len() as i32
            },
        )?;
        linker.func_wrap(
            "env",
            "roc_host_write_output",
            |mut caller: wasmtime::Caller<'_, HostState>, ptr: i32, len: i32| -> i32 {
                let Some(wasmtime::Extern::Memory(mem)) = caller.get_export("memory") else {
                    return -1;
                };
                let mut buf = vec![0u8; len as usize];
                if mem.read(&caller, ptr as usize, &mut buf).is_err() {
                    return -1;
                }
                if let Ok(out_str) = String::from_utf8(buf) {
                    caller.data_mut().output = out_str;
                    0
                } else {
                    -1
                }
            },
        )?;

        let instance = linker.instantiate(&mut store, &self.module)?;
        if let Ok(render_fn) = instance.get_typed_func::<(), ()>(&mut store, "render") {
            render_fn.call(&mut store, ())?;
            return Ok(store.into_data().output);
        } else if let Ok(main_fn) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
            main_fn.call(&mut store, ())?;
            return Ok(store.into_data().output);
        }

        bail!("wasm module does not export render or _start function");
    }

    pub fn run_wasi(&self, staging: &Path) -> Result<String> {
        use wasmtime_wasi::WasiCtxBuilder;
        use wasmtime_wasi::pipe::MemoryOutputPipe;

        let stdout_pipe = MemoryOutputPipe::new(4096);
        let stderr_pipe = MemoryOutputPipe::new(4096);

        let wasi = WasiCtxBuilder::new()
            .env("ROCDOWN_STAGING", staging.to_str().unwrap_or_default())
            .preopened_dir(
                staging,
                staging.to_str().unwrap_or_default(),
                wasmtime_wasi::DirPerms::all(),
                wasmtime_wasi::FilePerms::all(),
            )?
            .stdout(stdout_pipe.clone())
            .stderr(stderr_pipe.clone())
            .build_p1();

        let mut store = wasmtime::Store::new(&self.engine, wasi);
        let mut linker = wasmtime::Linker::new(&self.engine);
        wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |t| t)?;

        let instance = linker.instantiate(&mut store, &self.module)?;
        if let Ok(start) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
            let _ = start.call(&mut store, ());
        }

        let stdout_bytes = stdout_pipe.contents();
        let stderr_bytes = stderr_pipe.contents();
        let mut output = String::from_utf8_lossy(&stdout_bytes).into_owned();
        let stderr = String::from_utf8_lossy(&stderr_bytes);
        if !stderr.is_empty() {
            output.push_str(&stderr);
        }
        Ok(output)
    }
}
