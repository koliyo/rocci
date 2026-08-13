use roc_core::WindowEvent;
use tao::dpi::PhysicalSize;

#[derive(Debug)]
pub enum ShellEvent {
    NewWindow,
    #[cfg(target_os = "macos")]
    Menu(muda::MenuEvent),
}

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
}
