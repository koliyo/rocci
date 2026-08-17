use rocci_core::{Result, WindowConfig, WindowId};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    keyboard::ModifiersState,
    platform::run_return::EventLoopExtRunReturn,
};
use wry::{PageLoadEvent, WebContext};

use crate::{
    chrome,
    events::{self, PreviewEvent, ShellEvent},
    history::{NavCommand, NavHistory},
    menu::{self, MenuConfig},
    web_context_dir,
    window::{LiveWindow, WebViewHooks},
};

pub struct PreviewOptions {
    pub url: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub devtools: bool,
    pub state_key: Option<String>,
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
            state_key: None,
        }
    }
}

pub fn preview(options: PreviewOptions) -> Result<()> {
    let mut event_loop = EventLoopBuilder::<ShellEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let id = WindowId::new("preview");
    let state_key = options
        .state_key
        .clone()
        .unwrap_or_else(|| "preview".to_string());

    let saved_state = crate::state::load_window_state(&state_key);
    let (width, height) = match saved_state {
        Some(state) if state.width >= 100.0 && state.height >= 100.0 => (state.width, state.height),
        _ => (options.width, options.height),
    };

    let (initial_position, initial_maximized) = match saved_state {
        Some(state) => {
            let visible =
                crate::state::is_position_visible(&event_loop, state.x, state.y, width, height);
            let pos = if visible {
                Some(state.position())
            } else {
                None
            };
            (pos, state.is_maximized)
        }
        None => (None, false),
    };

    let template = WindowConfig {
        label: "preview".into(),
        title: options.title.clone(),
        width,
        height,
        ..WindowConfig::default()
    };
    let context = WebContext::new(Some(web_context_dir("dev.rocci.preview", &id)));
    let ipc_proxy = proxy.clone();
    let load_proxy = proxy.clone();
    let title_proxy = proxy.clone();
    let live = LiveWindow::create(
        &event_loop,
        &template,
        id,
        options.url.clone(),
        context,
        options.devtools,
        WebViewHooks {
            initialization_script: Some(chrome::INITIALIZATION_SCRIPT.into()),
            ipc_handler: Some(Box::new(move |request| {
                if let Some(command) = NavCommand::parse(request.body()) {
                    let _ =
                        ipc_proxy.send_event(ShellEvent::Preview(PreviewEvent::Command(command)));
                }
            })),
            on_page_load: Some(Box::new(move |event, url| {
                if matches!(event, PageLoadEvent::Finished) {
                    let _ = load_proxy.send_event(ShellEvent::Preview(PreviewEvent::Loaded(url)));
                }
            })),
            on_title_changed: Some(Box::new(move |title| {
                let _ = title_proxy.send_event(ShellEvent::Preview(PreviewEvent::Title(title)));
            })),
        },
        initial_position,
        initial_maximized,
    )?;
    let menu = menu::NativeMenu::install(
        proxy,
        MenuConfig {
            app_name: &options.title,
            version: None,
            new_window: false,
            navigation: true,
            reload: true,
            devtools: options.devtools,
        },
    )?;
    menu.attach(&live.window)?;

    let mut history = NavHistory::new(options.url);
    let mut title = options.title;
    let mut modifiers = ModifiersState::empty();
    let save_key = state_key.clone();
    event_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let _keep = &menu;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if let Some(state) = crate::state::capture_window_state(&live.window) {
                    crate::state::save_window_state(&save_key, state);
                }
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(next),
                ..
            } => modifiers = next,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } if events::is_close_key_event(&event, modifiers) => {
                if let Some(state) = crate::state::capture_window_state(&live.window) {
                    crate::state::save_window_state(&save_key, state);
                }
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(ShellEvent::Menu(menu_event)) => {
                if menu::is(&menu_event, menu::CLOSE_WINDOW_ID) {
                    if let Some(state) = crate::state::capture_window_state(&live.window) {
                        crate::state::save_window_state(&save_key, state);
                    }
                    *control_flow = ControlFlow::Exit;
                } else if menu::is(&menu_event, menu::BACK_ID) {
                    apply_command(&live, &mut history, NavCommand::Back);
                } else if menu::is(&menu_event, menu::FORWARD_ID) {
                    apply_command(&live, &mut history, NavCommand::Forward);
                } else if menu::is(&menu_event, menu::HOME_ID) {
                    apply_command(&live, &mut history, NavCommand::Home);
                } else if menu::is(&menu_event, menu::RELOAD_ID) {
                    apply_command(&live, &mut history, NavCommand::Reload);
                } else if menu::is(&menu_event, menu::WEB_INSPECTOR_ID) {
                    if live.webview.is_devtools_open() {
                        live.webview.close_devtools();
                    } else {
                        live.webview.open_devtools();
                    }
                }
            }
            Event::UserEvent(ShellEvent::Preview(PreviewEvent::Command(command))) => {
                apply_command(&live, &mut history, command);
            }
            Event::UserEvent(ShellEvent::Preview(PreviewEvent::Loaded(url))) => {
                history.commit(&url);
                sync_chrome(&live, &history, &title);
            }
            Event::UserEvent(ShellEvent::Preview(PreviewEvent::Title(next))) => {
                live.window.set_title(&next);
                title = next;
                sync_chrome(&live, &history, &title);
            }
            _ => {}
        }
    });

    if let Some(state) = crate::state::capture_window_state(&live.window) {
        crate::state::save_window_state(&state_key, state);
    }
    Ok(())
}

fn apply_command(live: &LiveWindow, history: &mut NavHistory, command: NavCommand) {
    let result = match command {
        NavCommand::Back => {
            if history.request_back() {
                live.webview.evaluate_script("history.back()")
            } else {
                return;
            }
        }
        NavCommand::Forward => {
            if history.request_forward() {
                live.webview.evaluate_script("history.forward()")
            } else {
                return;
            }
        }
        NavCommand::Home => {
            history.request_home();
            live.webview.load_url(history.home())
        }
        NavCommand::Reload => live.webview.reload(),
    };
    if let Err(error) = result {
        tracing::error!(%error, ?command, "failed to apply preview navigation");
    }
}

fn sync_chrome(live: &LiveWindow, history: &NavHistory, title: &str) {
    if let Err(error) = live.webview.evaluate_script(&chrome::update_script(
        title,
        &history.display_path(),
        history.can_back(),
        history.can_forward(),
    )) {
        tracing::error!(%error, "failed to update preview chrome");
    }
}
