use rocci_core::{Error, Result, WindowConfig, WindowId};
use tao::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};
use wry::{WebContext, WebView, WebViewBuilder};

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

        let webview_builder = WebViewBuilder::new_with_web_context(&mut context)
            .with_url(&url)
            .with_devtools(devtools);
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
