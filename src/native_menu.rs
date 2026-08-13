use anyhow::Result;
use muda::{
    AboutMetadata, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};
use tao::event_loop::EventLoopProxy;

pub(crate) const RELOAD_ID: &str = "view.reload";
pub(crate) const WEB_INSPECTOR_ID: &str = "view.web-inspector";

/// Owns the native menu tree for the lifetime of the application.
pub(crate) struct NativeMenu {
    _menu: Menu,
}

impl NativeMenu {
    pub(crate) fn install(proxy: EventLoopProxy<crate::AppEvent>) -> Result<Self> {
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = proxy.send_event(crate::AppEvent::Menu(event));
        }));

        let menu = Menu::new();

        let app = Submenu::new("Roc Datastar", true);
        app.append_items(&[
            &PredefinedMenuItem::about(
                Some("About Roc Datastar"),
                Some(AboutMetadata {
                    name: Some("Roc Datastar".into()),
                    version: Some(env!("CARGO_PKG_VERSION").into()),
                    copyright: Some("Built with Rust, tao, wry, and Datastar".into()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(Some("Hide Roc Datastar")),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(Some("Quit Roc Datastar")),
        ])?;

        let file = Submenu::new("File", true);
        file.append(&PredefinedMenuItem::close_window(Some("Close Window")))?;

        let edit = Submenu::new("Edit", true);
        edit.append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::select_all(None),
        ])?;

        let view = Submenu::new("View", true);
        view.append(&MenuItem::with_id(
            RELOAD_ID,
            "Reload",
            true,
            Some(Accelerator::new(Some(Modifiers::META), Code::KeyR)),
        ))?;
        if cfg!(debug_assertions) {
            view.append(&MenuItem::with_id(
                WEB_INSPECTOR_ID,
                "Toggle Web Inspector",
                true,
                Some(Accelerator::new(
                    Some(Modifiers::META | Modifiers::ALT),
                    Code::KeyI,
                )),
            ))?;
        }
        view.append(&PredefinedMenuItem::separator())?;
        view.append(&PredefinedMenuItem::fullscreen(Some("Enter Full Screen")))?;

        let window = Submenu::new("Window", true);
        window.append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::bring_all_to_front(None),
        ])?;

        let help = Submenu::new("Help", true);

        menu.append_items(&[&app, &file, &edit, &view, &window, &help])?;
        menu.init_for_nsapp();
        window.set_as_windows_menu_for_nsapp();
        help.set_as_help_menu_for_nsapp();

        Ok(Self { _menu: menu })
    }
}

pub(crate) fn is(event: &MenuEvent, expected: &str) -> bool {
    event.id() == &MenuId::new(expected)
}
