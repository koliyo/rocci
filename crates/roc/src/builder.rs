use std::future::pending;

use roc_core::{
    AppEvent, Backend, Config, Error, ExternalBackend, Hooks, ManagedState, Result, RunningBackend,
    WindowId, join_origin,
};
use roc_http::{AssetMap, AssetSource, HttpServer, Router};

pub struct App {
    config: Config,
    router: Option<Router>,
    backend: Option<Box<dyn Backend>>,
    assets: Option<AssetSource>,
    embedded: AssetMap,
    state: ManagedState,
    hooks: Hooks,
    serve_only: bool,
    dev_mode: Option<bool>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            config: Config::default(),
            router: None,
            backend: None,
            assets: None,
            embedded: AssetMap::new(),
            state: ManagedState::new(),
            hooks: Hooks::default(),
            serve_only: false,
            dev_mode: None,
        }
    }
}

impl App {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    pub fn backend<B: Backend + 'static>(mut self, backend: B) -> Self {
        self.backend = Some(Box::new(backend));
        self
    }

    pub fn assets(mut self, assets: impl Into<AssetSource>) -> Self {
        self.assets = Some(assets.into());
        self
    }

    pub fn embed_asset(
        mut self,
        path: impl Into<String>,
        content_type: impl Into<std::borrow::Cow<'static, str>>,
        bytes: impl Into<std::borrow::Cow<'static, [u8]>>,
    ) -> Self {
        self.embedded.insert(path, content_type, bytes);
        self
    }

    pub fn manage<T: Send + Sync + 'static>(self, value: T) -> Self {
        self.state.insert(value);
        self
    }

    pub fn setup<F>(mut self, setup: F) -> Self
    where
        F: FnOnce(&ManagedState) -> Result<()> + Send + 'static,
    {
        self.hooks.setup = Some(Box::new(setup));
        self
    }

    pub fn on_event<F>(mut self, on_event: F) -> Self
    where
        F: FnMut(&AppEvent) + Send + 'static,
    {
        self.hooks.on_event = Some(Box::new(on_event));
        self
    }

    pub fn on_exit<F>(mut self, on_exit: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        self.hooks.on_exit = Some(Box::new(on_exit));
        self
    }

    pub fn serve_only(mut self, serve_only: bool) -> Self {
        self.serve_only = serve_only;
        self
    }

    pub fn dev_mode(mut self, enabled: bool) -> Self {
        self.dev_mode = Some(enabled);
        self
    }

    pub fn run(mut self) -> Result<()> {
        self.config.validate()?;
        let dev_mode = self
            .dev_mode
            .unwrap_or(cfg!(debug_assertions) || std::env::var_os("ROC_DEV").is_some());
        let assets = self.resolve_assets(dev_mode)?;
        let runtime = tokio::runtime::Runtime::new().map_err(|error| {
            Error::backend(format!("failed to start the async runtime: {error}"))
        })?;

        let mut backend = self.start_backend(&runtime, assets, dev_mode)?;
        tracing::info!(origin = backend.origin(), "desktop backend ready");

        if self.serve_only {
            for window in self.config.windows.iter().filter(|window| window.visible) {
                let id = WindowId::new(&window.label);
                let url = backend.attach_window(&id, &window.url)?;
                println!("{url}");
            }
            runtime.block_on(pending::<()>());
            unreachable!("HTTP serve-only mode does not return");
        }

        if let Some(frontend) = dev_frontend(&self.config, dev_mode) {
            backend = Box::new(DevFrontend {
                inner: backend,
                frontend,
            });
        }

        #[cfg(feature = "desktop")]
        {
            let devtools = dev_mode && self.config.development.devtools;
            let reload = self.config.development.reload;
            roc_wry::run(roc_wry::RunOptions {
                config: self.config,
                backend,
                runtime,
                state: self.state,
                hooks: self.hooks,
                devtools,
                reload,
            })
        }

        #[cfg(not(feature = "desktop"))]
        {
            let _ = backend;
            Err(Error::message(
                "opening windows requires the `desktop` feature; use --serve-only to keep the HTTP server",
            ))
        }
    }

    fn start_backend(
        &mut self,
        runtime: &tokio::runtime::Runtime,
        assets: Option<AssetSource>,
        dev_mode: bool,
    ) -> Result<Box<dyn RunningBackend>> {
        if let Some(url) = dev_backend(&self.config, dev_mode) {
            return Ok(Box::new(ExternalBackend::new(url)));
        }
        if let Some(backend) = self.backend.take() {
            tracing::info!(backend = backend.name(), "starting desktop backend");
            return backend.start();
        }
        if let Some(router) = self.router.take() {
            tracing::info!(backend = "rust", "starting in-process HTTP backend");
            return Ok(Box::new(runtime.block_on(HttpServer::start(
                self.config.clone(),
                router,
                assets,
            ))?));
        }
        Err(Error::backend(
            "no backend configured; provide a router, a Backend implementation, or development.backend_url",
        ))
    }

    fn resolve_assets(&self, dev_mode: bool) -> Result<Option<AssetSource>> {
        if let Some(assets) = &self.assets {
            return Ok(Some(assets.clone()));
        }
        if let Some(directory) = &self.config.assets.directory
            && (dev_mode || !self.config.assets.embed)
        {
            return Ok(Some(AssetSource::directory(directory)));
        }
        if !self.embedded.is_empty() {
            return Ok(Some(AssetSource::from(self.embedded.clone())));
        }
        if let Some(directory) = &self.config.assets.directory {
            return Ok(Some(AssetSource::directory(directory)));
        }
        Ok(None)
    }
}

struct DevFrontend {
    inner: Box<dyn RunningBackend>,
    frontend: String,
}

impl RunningBackend for DevFrontend {
    fn origin(&self) -> &str {
        self.inner.origin()
    }

    fn attach_window(&self, window: &WindowId, start_url: &str) -> Result<String> {
        let _ = self.inner.attach_window(window, start_url)?;
        Ok(join_origin(&self.frontend, start_url))
    }

    fn detach_window(&self, window: &WindowId) {
        self.inner.detach_window(window);
    }

    fn shutdown(&mut self) {
        self.inner.shutdown();
    }
}

fn dev_backend(config: &Config, dev_mode: bool) -> Option<String> {
    dev_mode
        .then(|| config.development.backend_url.clone())
        .flatten()
}

fn dev_frontend(config: &Config, dev_mode: bool) -> Option<String> {
    dev_mode
        .then(|| config.development.frontend_url.clone())
        .flatten()
        .map(|url| url.trim_end_matches('/').to_owned())
}
