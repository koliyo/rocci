use crate::{Error, Result, WindowConfig, WindowId};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};
use wry::{PageLoadEvent, WebContext, WebView, WebViewBuilder, http::Request};

#[cfg(target_os = "macos")]
const UNIFIED_CHROME_HEIGHT: f64 = 52.0;
#[cfg(target_os = "macos")]
const TRAFFIC_LIGHT_INSET_X: f64 = 16.0;
#[cfg(target_os = "macos")]
const TRAFFIC_LIGHT_INSET_Y: f64 = UNIFIED_CHROME_HEIGHT - 14.0;

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
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    unified_titlebar: bool,
}

impl LiveWindow {
    #[allow(clippy::too_many_arguments)]
    pub fn create<T: 'static>(
        event_loop: &EventLoopWindowTarget<T>,
        template: &WindowConfig,
        id: WindowId,
        url: String,
        mut context: WebContext,
        devtools: bool,
        hooks: WebViewHooks,
        position: Option<LogicalPosition<f64>>,
        maximized: bool,
        unified_titlebar: bool,
    ) -> Result<Self> {
        let mut builder = WindowBuilder::new()
            .with_title(&template.title)
            .with_inner_size(LogicalSize::new(template.width, template.height))
            .with_window_icon(crate::icon::window_icon());
        if let (Some(min_width), Some(min_height)) = (template.min_width, template.min_height) {
            builder = builder.with_min_inner_size(LogicalSize::new(min_width, min_height));
        }
        if let Some(pos) = position {
            builder = builder.with_position(pos);
        }
        if maximized {
            builder = builder.with_maximized(true);
        }
        #[cfg(target_os = "macos")]
        if unified_titlebar {
            use tao::platform::macos::WindowBuilderExtMacOS;
            builder = builder
                .with_titlebar_transparent(true)
                .with_title_hidden(true)
                .with_fullsize_content_view(true)
                .with_traffic_light_inset(LogicalPosition::new(
                    TRAFFIC_LIGHT_INSET_X,
                    TRAFFIC_LIGHT_INSET_Y,
                ));
        }
        #[cfg(not(target_os = "macos"))]
        let _ = unified_titlebar;
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
        #[cfg(target_os = "macos")]
        let webview_builder = {
            if unified_titlebar {
                use wry::WebViewBuilderExtDarwin;
                webview_builder.with_traffic_light_inset(wry::dpi::LogicalPosition::new(
                    TRAFFIC_LIGHT_INSET_X,
                    TRAFFIC_LIGHT_INSET_Y,
                ))
            } else {
                webview_builder
            }
        };
        let webview = webview_builder
            .build(&window)
            .map_err(|error| Error::message(format!("failed to create webview {id}: {error}")))?;

        apply_geometry(&window, template, position, maximized);

        let live = Self {
            window,
            webview,
            context: Some(context),
            unified_titlebar,
        };
        live.sync_unified_chrome();
        Ok(live)
    }

    pub fn sync_unified_chrome(&self) {
        #[cfg(target_os = "macos")]
        if self.unified_titlebar {
            align_traffic_lights(&self.window);
        }
    }

    pub fn realize_unified_chrome(&self) {
        self.sync_unified_chrome();
        #[cfg(target_os = "macos")]
        if self.unified_titlebar {
            let size = self.window.inner_size();
            self.window.set_inner_size(size);
            self.window.request_redraw();
            self.sync_unified_chrome();
        }
    }
}

#[cfg(target_os = "macos")]
pub fn begin_toolbar_drag() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let Some(window) = app.keyWindow() else {
        return;
    };
    let Some(event) = app.currentEvent() else {
        return;
    };
    window.performWindowDragWithEvent(&event);
}

fn apply_geometry(
    window: &Window,
    template: &WindowConfig,
    position: Option<LogicalPosition<f64>>,
    maximized: bool,
) {
    if !maximized {
        window.set_inner_size(LogicalSize::new(template.width, template.height));
    }
    if let Some(pos) = position {
        window.set_outer_position(pos);
    }
    if maximized {
        window.set_maximized(true);
    }
}

#[cfg(target_os = "macos")]
fn align_traffic_lights(window: &Window) {
    use objc2_app_kit::{NSView, NSWindow, NSWindowButton};
    use tao::platform::macos::WindowExtMacOS;

    unsafe {
        let Some(ns_window) = (window.ns_window() as *const NSWindow).as_ref() else {
            return;
        };
        let Some(close) = ns_window.standardWindowButton(NSWindowButton::CloseButton) else {
            return;
        };
        let Some(miniaturize) = ns_window.standardWindowButton(NSWindowButton::MiniaturizeButton)
        else {
            return;
        };
        let zoom = ns_window.standardWindowButton(NSWindowButton::ZoomButton);
        let Some(title_bar) = close.superview().and_then(|view| view.superview()) else {
            return;
        };

        let close_rect = close.frame();
        let y = ((UNIFIED_CHROME_HEIGHT - close_rect.size.height) / 2.0).max(0.0);
        let mut title_bar_rect = title_bar.frame();
        title_bar_rect.size.height = UNIFIED_CHROME_HEIGHT;
        title_bar_rect.origin.y = ns_window.frame().size.height - UNIFIED_CHROME_HEIGHT;
        title_bar.setFrame(title_bar_rect);

        let space = NSView::frame(&miniaturize).origin.x - close_rect.origin.x;
        let mut buttons = vec![close, miniaturize];
        if let Some(zoom) = zoom {
            buttons.push(zoom);
        }
        for (index, button) in buttons.into_iter().enumerate() {
            let mut rect = button.frame();
            rect.origin.x = TRAFFIC_LIGHT_INSET_X + index as f64 * space;
            rect.origin.y = y;
            button.setFrameOrigin(rect.origin);
        }
    }
}
