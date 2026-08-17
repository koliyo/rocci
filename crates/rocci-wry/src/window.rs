use rocci_core::{Error, Result, WindowConfig, WindowId};
use tao::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};
use wry::{PageLoadEvent, WebContext, WebView, WebViewBuilder, http::Request};

#[derive(Default)]
pub struct WebViewHooks {
    pub initialization_script: Option<String>,
    pub ipc_handler: Option<Box<dyn Fn(Request<String>) + 'static>>,
    pub on_page_load: Option<Box<dyn Fn(PageLoadEvent, String) + 'static>>,
    pub on_title_changed: Option<Box<dyn Fn(String) + 'static>>,
}

pub struct LiveWindow {
    pub window: Window,
    pub webview: WebView,
    #[allow(dead_code)]
    pub context: Option<WebContext>,
}

impl LiveWindow {
    pub fn create<T: 'static>(
        event_loop: &EventLoopWindowTarget<T>,
        template: &WindowConfig,
        id: WindowId,
        url: String,
        mut context: WebContext,
        devtools: bool,
        hooks: WebViewHooks,
    ) -> Result<Self> {
        let mut builder = WindowBuilder::new()
            .with_title(&template.title)
            .with_inner_size(LogicalSize::new(template.width, template.height));
        if let (Some(min_width), Some(min_height)) = (template.min_width, template.min_height) {
            builder = builder.with_min_inner_size(LogicalSize::new(min_width, min_height));
        }
        let window = builder
            .build(event_loop)
            .map_err(|error| Error::message(format!("failed to create window {id}: {error}")))?;

        let mut webview_builder = WebViewBuilder::new_with_web_context(&mut context)
            .with_url(&url)
            .with_devtools(devtools);
        if let Some(script) = hooks.initialization_script {
            webview_builder = webview_builder.with_initialization_script(script);
        }
        if let Some(handler) = hooks.ipc_handler {
            webview_builder = webview_builder.with_ipc_handler(handler);
        }
        if let Some(handler) = hooks.on_page_load {
            webview_builder = webview_builder.with_on_page_load_handler(handler);
        }
        if let Some(handler) = hooks.on_title_changed {
            webview_builder = webview_builder.with_document_title_changed_handler(handler);
        }
        #[cfg(target_os = "windows")]
        let webview_builder = {
            use wry::WebViewBuilderExtWindows;
            webview_builder.with_browser_accelerator_keys(false)
        };
        let webview = webview_builder
            .build(&window)
            .map_err(|error| Error::message(format!("failed to create webview {id}: {error}")))?;

        Ok(Self {
            window,
            webview,
            context: Some(context),
        })
    }
}
