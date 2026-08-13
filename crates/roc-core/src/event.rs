use crate::{ManagedState, Result, WindowId};

pub type SetupHook = Box<dyn FnOnce(&ManagedState) -> Result<()> + Send>;
pub type EventHook = Box<dyn FnMut(&AppEvent) + Send>;
pub type ExitHook = Box<dyn FnOnce() + Send>;

/// Callbacks invoked on the native event-loop thread.
#[derive(Default)]
pub struct Hooks {
    pub setup: Option<SetupHook>,
    pub on_event: Option<EventHook>,
    pub on_exit: Option<ExitHook>,
}

impl Hooks {
    pub fn emit(&mut self, event: &AppEvent) {
        if let Some(on_event) = &mut self.on_event {
            on_event(event);
        }
    }
}

/// Typed application events produced from native shell callbacks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppEvent {
    Ready,
    Window { id: WindowId, event: WindowEvent },
    Menu { id: String },
    Reopen { has_visible_windows: bool },
    ExitRequested,
    Exited,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WindowEvent {
    Created,
    CloseRequested,
    Resized { width: u32, height: u32 },
    Focused(bool),
    Destroyed,
}
