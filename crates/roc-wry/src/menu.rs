#![cfg(target_os = "macos")]

use muda::{
    AboutMetadata, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};
use roc_core::{Error, Result};
use tao::event_loop::EventLoopProxy;

use crate::ShellEvent;

pub const NEW_WINDOW_ID: &str = "file.new-window";
pub const RELOAD_ID: &str = "view.reload";
pub const WEB_INSPECTOR_ID: &str = "view.web-inspector";

pub struct NativeMenu {
    _menu: Menu,
}

impl NativeMenu {
    pub fn install(
        proxy: EventLoopProxy<ShellEvent>,
        app_name: &str,
        version: Option<&str>,
        reload: bool,
        devtools: bool,
    ) -> Result<Self> {
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = proxy.send_event(ShellEvent::Menu(event));
        }));

        let menu = Menu::new();

        let app = Submenu::new(app_name, true);
        app.append_items(&[
            &PredefinedMenuItem::about(
                Some(&format!("About {app_name}")),
                Some(AboutMetadata {
                    name: Some(app_name.into()),
                    version: version.map(str::to_owned),
                    copyright: Some("Built with Rust, tao, wry, and Datastar".into()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(Some(&format!("Hide {app_name}"))),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(Some(&format!("Quit {app_name}"))),
        ])
        .map_err(menu_error)?;

        let file = Submenu::new("File", true);
        file.append(&MenuItem::with_id(
            NEW_WINDOW_ID,
            "New Window",
            true,
            Some(Accelerator::new(Some(Modifiers::META), Code::KeyN)),
        ))
        .map_err(menu_error)?;
        file.append(&PredefinedMenuItem::close_window(Some("Close Window")))
            .map_err(menu_error)?;

        let edit = Submenu::new("Edit", true);
        edit.append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::select_all(None),
        ])
        .map_err(menu_error)?;

        let view = Submenu::new("View", true);
        if reload {
            view.append(&MenuItem::with_id(
                RELOAD_ID,
                "Reload",
                true,
                Some(Accelerator::new(Some(Modifiers::META), Code::KeyR)),
            ))
            .map_err(menu_error)?;
        }
        if devtools {
            view.append(&MenuItem::with_id(
                WEB_INSPECTOR_ID,
                "Toggle Web Inspector",
                true,
                Some(Accelerator::new(
                    Some(Modifiers::META | Modifiers::ALT),
                    Code::KeyI,
                )),
            ))
            .map_err(menu_error)?;
        }
        view.append(&PredefinedMenuItem::separator())
            .map_err(menu_error)?;
        view.append(&PredefinedMenuItem::fullscreen(Some("Enter Full Screen")))
            .map_err(menu_error)?;

        let window = Submenu::new("Window", true);
        window
            .append_items(&[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::bring_all_to_front(None),
            ])
            .map_err(menu_error)?;

        let help = Submenu::new("Help", true);

        menu.append_items(&[&app, &file, &edit, &view, &window, &help])
            .map_err(menu_error)?;
        menu.init_for_nsapp();
        window.set_as_windows_menu_for_nsapp();
        help.set_as_help_menu_for_nsapp();

        Ok(Self { _menu: menu })
    }
}

pub fn is(event: &MenuEvent, expected: &str) -> bool {
    event.id() == &MenuId::new(expected)
}

fn menu_error(error: impl std::fmt::Display) -> roc_core::Error {
    Error::message(format!("failed to install the native menu: {error}"))
}
