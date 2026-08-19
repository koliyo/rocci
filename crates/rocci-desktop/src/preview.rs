use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use rocci_core::{Result, WindowConfig, WindowId};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    keyboard::ModifiersState,
    platform::run_return::EventLoopExtRunReturn,
};
use wry::{PageLoadEvent, WebContext};

use crate::{
    chrome,
    events::{self, PreviewEvent, PreviewSink, ShellEvent},
    history::{IpcMessage, NavCommand, NavHistory},
    menu::{self, MenuConfig},
    source, web_context_dir,
    window::{LiveWindow, WebViewHooks},
};

pub struct PreviewOptions {
    pub url: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub devtools: bool,
    pub state_key: Option<String>,
    pub inspector_url: Option<String>,
    pub source_root: Option<std::path::PathBuf>,
    pub extra_initialization_script: Option<String>,
    pub on_ipc: Option<Arc<dyn Fn(&str, Arc<dyn PreviewSink>) + Send + Sync>>,
    pub picker: bool,
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
            inspector_url: None,
            source_root: None,
            extra_initialization_script: None,
            on_ipc: None,
            picker: false,
        }
    }
}

struct ProxySink(EventLoopProxy<ShellEvent>);

impl PreviewSink for ProxySink {
    fn send(&self, event: PreviewEvent) {
        let _ = self.0.send_event(ShellEvent::Preview(event));
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
    let host_ipc = options.on_ipc.clone();
    let host_sink: Arc<dyn PreviewSink> = Arc::new(ProxySink(proxy.clone()));
    let mut init_script = chrome::initialization_script(
        options.inspector_url.as_deref(),
        options.source_root.is_some(),
    );
    if let Some(extra) = &options.extra_initialization_script {
        init_script.push('\n');
        init_script.push_str(extra);
    }
    let live = LiveWindow::create(
        &event_loop,
        &template,
        id,
        options.url.clone(),
        context,
        options.devtools,
        WebViewHooks {
            initialization_script: Some(init_script),
            ipc_handler: Some(Box::new(move |request| {
                match IpcMessage::parse(request.body()) {
                    Some(IpcMessage::Nav(command)) => {
                        let _ = ipc_proxy
                            .send_event(ShellEvent::Preview(PreviewEvent::Command(command)));
                    }
                    Some(IpcMessage::Reveal(path)) => {
                        let _ =
                            ipc_proxy.send_event(ShellEvent::Preview(PreviewEvent::Reveal(path)));
                    }
                    Some(IpcMessage::CopySource(path)) => {
                        let _ = ipc_proxy
                            .send_event(ShellEvent::Preview(PreviewEvent::CopySource(path)));
                    }
                    None => {
                        if let Some(handler) = &host_ipc {
                            handler(request.body(), host_sink.clone());
                        }
                    }
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
            search: true,
            reload: true,
            devtools: options.devtools,
            picker: options.picker,
        },
    )?;
    menu.attach(&live.window)?;

    let mut history = NavHistory::new(options.url);
    let mut title = options.title;
    let mut modifiers = ModifiersState::empty();
    let save_key = state_key.clone();
    let source_root = options.source_root.clone();
    let mut last_persist = Instant::now() - Duration::from_secs(1);
    event_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let _keep = &menu;
        match event {
            Event::NewEvents(StartCause::Init) => crate::icon::apply_host_icon(),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                crate::state::persist_window_state(&save_key, &live.window);
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Moved(_) | WindowEvent::Resized(_),
                ..
            } => {
                if last_persist.elapsed() >= Duration::from_millis(250) {
                    crate::state::persist_window_state(&save_key, &live.window);
                    last_persist = Instant::now();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(next),
                ..
            } => modifiers = next,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } if events::is_close_key_event(&event, modifiers) => {
                crate::state::persist_window_state(&save_key, &live.window);
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(ShellEvent::Menu(menu_event)) => {
                if menu::is(&menu_event, menu::CLOSE_WINDOW_ID)
                    || menu::is(&menu_event, menu::QUIT_ID)
                {
                    crate::state::persist_window_state(&save_key, &live.window);
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
                } else if menu::is(&menu_event, menu::FIND_ID) {
                    apply_overlay(&live, chrome::FIND_OPEN_SCRIPT);
                } else if menu::is(&menu_event, menu::FIND_NEXT_ID) {
                    apply_overlay(&live, chrome::FIND_NEXT_SCRIPT);
                } else if menu::is(&menu_event, menu::FIND_PREVIOUS_ID) {
                    apply_overlay(&live, chrome::FIND_PREV_SCRIPT);
                } else if menu::is(&menu_event, menu::USE_SELECTION_ID) {
                    apply_overlay(&live, chrome::FIND_USE_SELECTION_SCRIPT);
                } else if menu::is(&menu_event, menu::GO_TO_FILE_ID) {
                    apply_overlay(&live, chrome::GOTO_OPEN_SCRIPT);
                } else if menu::is(&menu_event, menu::OPEN_PICKER_ID) {
                    apply_overlay(&live, chrome::PICKER_OPEN_SCRIPT);
                } else if menu::is(&menu_event, menu::SELECT_ALL_ID) {
                    apply_overlay(&live, chrome::SELECT_ALL_SCRIPT);
                }
            }
            Event::UserEvent(ShellEvent::Preview(PreviewEvent::Command(command))) => {
                apply_command(&live, &mut history, command);
            }
            Event::UserEvent(ShellEvent::Preview(PreviewEvent::Reveal(spec))) => {
                apply_source(&source_root, &spec, SourceAction::Reveal);
            }
            Event::UserEvent(ShellEvent::Preview(PreviewEvent::CopySource(spec))) => {
                apply_source(&source_root, &spec, SourceAction::Copy);
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
            Event::UserEvent(ShellEvent::Preview(PreviewEvent::Navigate {
                url,
                title: next_title,
                inspector_url,
            })) => {
                history.reset_origin(&url);
                if let Err(error) = live.webview.load_url(&url) {
                    tracing::error!(%error, "failed to load preview origin");
                }
                live.window.set_title(&next_title);
                title = next_title;
                if let Some(inspector) = inspector_url {
                    apply_overlay(&live, &chrome::set_inspector_script(&inspector));
                }
            }
            Event::UserEvent(ShellEvent::Preview(PreviewEvent::Evaluate(script))) => {
                apply_overlay(&live, &script);
            }
            _ => {}
        }
    });

    crate::state::persist_window_state(&state_key, &live.window);
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

fn apply_overlay(live: &LiveWindow, script: &str) {
    if let Err(error) = live.webview.evaluate_script(script) {
        tracing::error!(%error, "failed to apply preview overlay action");
    }
}

enum SourceAction {
    Reveal,
    Copy,
}

fn apply_source(root: &Option<std::path::PathBuf>, spec: &str, action: SourceAction) {
    let Some(root) = root else {
        return;
    };
    let Some(path) = source::resolve_source_file(root, spec) else {
        tracing::warn!(spec, root = %root.display(), "preview source file not found");
        return;
    };
    match action {
        SourceAction::Reveal => source::reveal_in_file_manager(&path),
        SourceAction::Copy => source::copy_file_text(&path),
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
