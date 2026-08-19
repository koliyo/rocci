//! Native window and webview shell built on tao and wry.

mod chrome;
mod events;
mod history;
mod icon;
mod menu;
mod preview;
mod source;
pub mod state;
mod window;

use std::{collections::HashMap, env, fs, path::PathBuf};

use rocci_core::{
    AppEvent, Config, Hooks, ManagedState, Result, RunningBackend, WindowConfig, WindowEvent,
    WindowId,
};
use tao::{
    event::{Event, StartCause},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopWindowTarget},
    keyboard::ModifiersState,
};
use wry::WebContext;

use crate::window::LiveWindow;

pub use events::ShellEvent;
pub use preview::{PreviewOptions, preview};

pub struct RunOptions {
    pub config: Config,
    pub backend: Box<dyn RunningBackend>,
    pub runtime: tokio::runtime::Runtime,
    pub state: ManagedState,
    pub hooks: Hooks,
    pub devtools: bool,
    pub reload: bool,
}

struct Shell {
    config: Config,
    backend: Box<dyn RunningBackend>,
    windows: HashMap<WindowId, LiveWindow>,
    tao_ids: HashMap<tao::window::WindowId, WindowId>,
    focused: Option<WindowId>,
    hooks: Hooks,
    devtools: bool,
    reload: bool,
    menu: menu::NativeMenu,
    modifiers: ModifiersState,
}

pub fn run(mut options: RunOptions) -> Result<()> {
    if let Some(setup) = options.hooks.setup.take() {
        setup(&options.state)?;
    }

    let event_loop = EventLoopBuilder::<ShellEvent>::with_user_event().build();
    crate::icon::apply_host_icon();

    let native_menu = menu::NativeMenu::install(
        event_loop.create_proxy(),
        menu::MenuConfig {
            app_name: &options.config.app.name,
            version: options.config.app.version.as_deref(),
            new_window: true,
            navigation: false,
            search: false,
            reload: options.reload,
            devtools: options.devtools,
        },
    )?;

    let mut shell = Shell {
        config: options.config,
        backend: options.backend,
        windows: HashMap::new(),
        tao_ids: HashMap::new(),
        focused: None,
        hooks: options.hooks,
        devtools: options.devtools,
        reload: options.reload,
        menu: native_menu,
        modifiers: ModifiersState::empty(),
    };

    let templates: Vec<WindowConfig> = shell
        .config
        .windows
        .iter()
        .filter(|window| window.visible)
        .cloned()
        .collect();
    for template in templates {
        shell.open_window(&event_loop, &template, None)?;
    }
    shell.hooks.emit(&AppEvent::Ready);

    let runtime = options.runtime;
    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        let _runtime = &runtime;

        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::WindowEvent {
                window_id: tao_id,
                event,
                ..
            } => {
                if let Some(id) = shell.tao_ids.get(&tao_id).cloned() {
                    shell.handle_window_event(event_loop, control_flow, id, event);
                }
            }
            #[cfg(target_os = "macos")]
            Event::Reopen {
                has_visible_windows,
                ..
            } => {
                shell.hooks.emit(&AppEvent::Reopen {
                    has_visible_windows,
                });
                if !has_visible_windows
                    && let Some(template) = shell.config.windows.first().cloned()
                    && let Err(error) = shell.open_window(event_loop, &template, None)
                {
                    tracing::error!(%error, "failed to reopen window");
                }
            }
            Event::UserEvent(user_event) => {
                shell.handle_user_event(event_loop, control_flow, user_event)
            }
            Event::LoopDestroyed => {
                for (id, live) in &shell.windows {
                    let state_key = format!("{}:{}", shell.config.app.identifier, id.as_str());
                    state::persist_window_state(&state_key, &live.window);
                }
                shell.hooks.emit(&AppEvent::Exited);
                if let Some(on_exit) = shell.hooks.on_exit.take() {
                    on_exit();
                }
                shell.backend.shutdown();
            }
            _ => {}
        }
    });
}

impl Shell {
    fn open_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<ShellEvent>,
        template: &WindowConfig,
        id: Option<WindowId>,
    ) -> Result<()> {
        let id = id.unwrap_or_else(|| self.allocate_id(&template.label));
        let url = self.backend.attach_window(&id, &template.url)?;
        let context = WebContext::new(Some(web_context_dir(&self.config.app.identifier, &id)));

        let state_key = format!("{}:{}", self.config.app.identifier, template.label);
        let saved_state = state::load_window_state(&state_key);
        let mut template = template.clone();
        if let Some(state) = saved_state
            && state.width >= 100.0
            && state.height >= 100.0
        {
            template.width = state.width;
            template.height = state.height;
        }

        let (initial_position, initial_maximized) = match saved_state {
            Some(state) => {
                let visible = state::is_position_visible(
                    event_loop,
                    state.x,
                    state.y,
                    template.width,
                    template.height,
                );
                let pos = if visible {
                    Some(state.position())
                } else {
                    None
                };
                (pos, state.is_maximized)
            }
            None => (None, false),
        };

        let live = LiveWindow::create(
            event_loop,
            &template,
            id.clone(),
            url,
            context,
            self.devtools,
            window::WebViewHooks::default(),
            initial_position,
            initial_maximized,
        )?;
        self.menu.attach(&live.window)?;
        let tao_id = live.window.id();
        self.tao_ids.insert(tao_id, id.clone());
        self.windows.insert(id.clone(), live);
        self.focused = Some(id.clone());
        self.hooks.emit(&AppEvent::Window {
            id,
            event: WindowEvent::Created,
        });
        Ok(())
    }

    fn allocate_id(&self, label: &str) -> WindowId {
        let base = WindowId::new(label);
        if !self.windows.contains_key(&base) {
            return base;
        }
        for index in 2.. {
            let candidate = WindowId::new(format!("{label}-{index}"));
            if !self.windows.contains_key(&candidate) {
                return candidate;
            }
        }
        base
    }

    fn handle_window_event(
        &mut self,
        event_loop: &EventLoopWindowTarget<ShellEvent>,
        control_flow: &mut ControlFlow,
        id: WindowId,
        event: tao::event::WindowEvent,
    ) {
        if let Some(mapped) = events::map_window_event(&event) {
            self.hooks.emit(&AppEvent::Window {
                id: id.clone(),
                event: mapped,
            });
        }

        match event {
            tao::event::WindowEvent::CloseRequested => self.close_window(id, control_flow),
            tao::event::WindowEvent::KeyboardInput { event, .. }
                if events::is_close_key_event(&event, self.modifiers) =>
            {
                self.close_window(id, control_flow);
            }
            tao::event::WindowEvent::Moved(_) | tao::event::WindowEvent::Resized(_) => {
                if let Some(live) = self.windows.get(&id) {
                    let state_key = format!("{}:{}", self.config.app.identifier, id.as_str());
                    state::persist_window_state(&state_key, &live.window);
                }
            }
            tao::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            tao::event::WindowEvent::Focused(true) => self.focused = Some(id),
            tao::event::WindowEvent::Destroyed => {
                let _ = event_loop;
            }
            _ => {}
        }
    }

    fn close_window(&mut self, id: WindowId, control_flow: &mut ControlFlow) {
        if let Some(live) = self.windows.remove(&id) {
            let state_key = format!("{}:{}", self.config.app.identifier, id.as_str());
            state::persist_window_state(&state_key, &live.window);
            self.tao_ids.remove(&live.window.id());
            self.backend.detach_window(&id);
            if self.focused.as_ref() == Some(&id) {
                self.focused = self.windows.keys().next().cloned();
            }
            self.hooks.emit(&AppEvent::Window {
                id,
                event: WindowEvent::Destroyed,
            });
        }

        if self.windows.is_empty() {
            #[cfg(target_os = "macos")]
            {
                let _ = control_flow;
            }
            #[cfg(not(target_os = "macos"))]
            {
                self.hooks.emit(&AppEvent::ExitRequested);
                self.backend.shutdown();
                *control_flow = ControlFlow::Exit;
            }
        }
    }

    fn handle_user_event(
        &mut self,
        event_loop: &EventLoopWindowTarget<ShellEvent>,
        control_flow: &mut ControlFlow,
        event: ShellEvent,
    ) {
        match event {
            ShellEvent::NewWindow => {
                if let Some(template) = self.config.windows.first().cloned()
                    && let Err(error) = self.open_window(event_loop, &template, None)
                {
                    tracing::error!(%error, "failed to open window");
                }
            }
            ShellEvent::Menu(menu_event) => {
                let id = menu_event.id().as_ref().to_owned();
                self.hooks.emit(&AppEvent::Menu { id: id.clone() });
                if menu::is(&menu_event, menu::NEW_WINDOW_ID) {
                    self.handle_user_event(event_loop, control_flow, ShellEvent::NewWindow);
                } else if menu::is(&menu_event, menu::QUIT_ID) {
                    let ids: Vec<WindowId> = self.windows.keys().cloned().collect();
                    for id in ids {
                        self.close_window(id, control_flow);
                    }
                    self.hooks.emit(&AppEvent::ExitRequested);
                    self.backend.shutdown();
                    *control_flow = ControlFlow::Exit;
                } else if menu::is(&menu_event, menu::CLOSE_WINDOW_ID) {
                    if let Some(id) = self.focused.clone() {
                        self.close_window(id, control_flow);
                    }
                } else if menu::is(&menu_event, menu::SELECT_ALL_ID) {
                    self.select_all_focused();
                } else if self.reload && menu::is(&menu_event, menu::RELOAD_ID) {
                    self.reload_focused();
                } else if self.devtools && menu::is(&menu_event, menu::WEB_INSPECTOR_ID) {
                    self.toggle_inspector();
                }
            }
            ShellEvent::Preview(_) => {}
        }
    }

    fn reload_focused(&self) {
        if let Some(window) = self.focused.as_ref().and_then(|id| self.windows.get(id))
            && let Err(error) = window.webview.reload()
        {
            tracing::error!(%error, "failed to reload webview");
        }
    }

    fn select_all_focused(&self) {
        if let Some(window) = self.focused.as_ref().and_then(|id| self.windows.get(id))
            && let Err(error) = window.webview.evaluate_script(chrome::SELECT_ALL_SCRIPT)
        {
            tracing::error!(%error, "failed to select all in webview");
        }
    }

    fn toggle_inspector(&self) {
        if let Some(window) = self.focused.as_ref().and_then(|id| self.windows.get(id)) {
            if window.webview.is_devtools_open() {
                window.webview.close_devtools();
            } else {
                window.webview.open_devtools();
            }
        }
    }
}

pub(crate) fn web_context_dir(identifier: &str, window: &WindowId) -> PathBuf {
    let dir = env::temp_dir()
        .join("rocci")
        .join(identifier)
        .join(window.as_str());
    if let Err(error) = fs::create_dir_all(&dir) {
        tracing::warn!(%error, path = %dir.display(), "failed to create webview data directory");
    }
    dir
}
