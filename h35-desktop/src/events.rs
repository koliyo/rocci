use crate::WindowEvent;
use tao::{
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, ModifiersState},
};

use crate::history::NavCommand;

#[derive(Debug)]
pub enum ShellEvent {
    NewWindow,
    Menu(muda::MenuEvent),
    Preview(PreviewEvent),
}

#[derive(Debug)]
pub enum PreviewEvent {
    Command(NavCommand),
    Reveal(String),
    CopySource(String),
    LiveReload(bool),
    Devtools(bool),
    InspectorPrefs(String),
    Layout(String),
    Location(String),
    Drag,
    Zoom,
    Loaded(String),
    Title(String),
    PickFolder,
    PickFolderResult(Option<String>),
    Navigate {
        url: String,
        title: String,
        inspector_url: Option<String>,
    },
    Evaluate(String),
}

pub trait PreviewSink: Send + Sync {
    fn send(&self, event: PreviewEvent);
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn map_window_event(event: &tao::event::WindowEvent) -> Option<WindowEvent> {
    match event {
        tao::event::WindowEvent::CloseRequested => Some(WindowEvent::CloseRequested),
        tao::event::WindowEvent::Destroyed => Some(WindowEvent::Destroyed),
        tao::event::WindowEvent::Focused(focused) => Some(WindowEvent::Focused(*focused)),
        tao::event::WindowEvent::Resized(PhysicalSize { width, height }) => {
            Some(WindowEvent::Resized {
                width: *width,
                height: *height,
            })
        }
        _ => None,
    }
}

pub fn is_close_shortcut(key: KeyCode, modifiers: ModifiersState) -> bool {
    key == KeyCode::KeyW && close_modifier(modifiers)
}

pub fn is_close_key_event(event: &KeyEvent, modifiers: ModifiersState) -> bool {
    event.state == ElementState::Pressed
        && !event.repeat
        && is_close_shortcut(event.physical_key, modifiers)
}

fn close_modifier(modifiers: ModifiersState) -> bool {
    #[cfg(target_os = "macos")]
    {
        modifiers.super_key()
            && !modifiers.control_key()
            && !modifiers.alt_key()
            && !modifiers.shift_key()
    }
    #[cfg(not(target_os = "macos"))]
    {
        modifiers.control_key()
            && !modifiers.super_key()
            && !modifiers.alt_key()
            && !modifiers.shift_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_close_and_focus_events() {
        assert_eq!(
            map_window_event(&tao::event::WindowEvent::CloseRequested),
            Some(WindowEvent::CloseRequested)
        );
        assert_eq!(
            map_window_event(&tao::event::WindowEvent::Focused(true)),
            Some(WindowEvent::Focused(true))
        );
    }

    #[test]
    fn close_shortcut_uses_platform_modifier() {
        #[cfg(target_os = "macos")]
        {
            assert!(is_close_shortcut(KeyCode::KeyW, ModifiersState::SUPER));
            assert!(!is_close_shortcut(KeyCode::KeyW, ModifiersState::CONTROL));
            assert!(!is_close_shortcut(
                KeyCode::KeyW,
                ModifiersState::SUPER | ModifiersState::SHIFT
            ));
            assert!(!is_close_shortcut(
                KeyCode::KeyW,
                ModifiersState::SUPER | ModifiersState::CONTROL
            ));
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(is_close_shortcut(KeyCode::KeyW, ModifiersState::CONTROL));
            assert!(!is_close_shortcut(KeyCode::KeyW, ModifiersState::SUPER));
            assert!(!is_close_shortcut(
                KeyCode::KeyW,
                ModifiersState::CONTROL | ModifiersState::SHIFT
            ));
            assert!(!is_close_shortcut(
                KeyCode::KeyW,
                ModifiersState::CONTROL | ModifiersState::ALT
            ));
        }
        assert!(!is_close_shortcut(KeyCode::KeyN, ModifiersState::SUPER));
        assert!(!is_close_shortcut(KeyCode::KeyW, ModifiersState::empty()));
    }
}
