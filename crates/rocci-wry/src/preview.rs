use rocci_core::{Result, WindowConfig, WindowId};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};
use wry::WebContext;

use crate::{web_context_dir, window::LiveWindow};

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
    let mut event_loop = EventLoop::new();
    let id = WindowId::new("preview");
    let template = WindowConfig {
        label: "preview".into(),
        title: options.title,
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

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let _keep = &live;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
    Ok(())
}
