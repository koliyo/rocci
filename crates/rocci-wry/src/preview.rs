use rocci_core::{Result, WindowConfig, WindowId};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    keyboard::ModifiersState,
    platform::run_return::EventLoopExtRunReturn,
};
use wry::WebContext;

use crate::{
    events::{self, ShellEvent},
    menu::{self, MenuConfig},
    web_context_dir,
    window::LiveWindow,
};

pub struct PreviewOptions {
    pub url: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub devtools: bool,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        let defaults = WindowConfig::default();
        Self {
            url: String::new(),
            title: defaults.title,
            width: defaults.width,
            height: defaults.height,
            devtools: true,
        }
    }
}

pub fn preview(options: PreviewOptions) -> Result<()> {
    let mut event_loop = EventLoopBuilder::<ShellEvent>::with_user_event().build();
    let id = WindowId::new("preview");
    let template = WindowConfig {
        label: "preview".into(),
        title: options.title.clone(),
        width: options.width,
        height: options.height,
        ..WindowConfig::default()
    };
    let context = WebContext::new(Some(web_context_dir("dev.rocci.preview", &id)));
    let live = LiveWindow::create(
        &event_loop,
        &template,
        id,
        options.url,
        context,
        options.devtools,
    )?;
    let menu = menu::NativeMenu::install(
        event_loop.create_proxy(),
        MenuConfig {
            app_name: &options.title,
            version: None,
            new_window: false,
            reload: true,
            devtools: options.devtools,
        },
    )?;
    menu.attach(&live.window)?;

    let mut modifiers = ModifiersState::empty();
    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let _keep = (&live, &menu);
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(next),
                ..
            } => modifiers = next,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } if events::is_close_key_event(&event, modifiers) => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(ShellEvent::Menu(menu_event)) => {
                if menu::is(&menu_event, menu::CLOSE_WINDOW_ID) {
                    *control_flow = ControlFlow::Exit;
                } else if menu::is(&menu_event, menu::RELOAD_ID) {
                    if let Err(error) = live.webview.reload() {
                        tracing::error!(%error, "failed to reload webview");
                    }
                } else if menu::is(&menu_event, menu::WEB_INSPECTOR_ID) {
                    if live.webview.is_devtools_open() {
                        live.webview.close_devtools();
                    } else {
                        live.webview.open_devtools();
                    }
                }
            }
            _ => {}
        }
    });
    Ok(())
}
