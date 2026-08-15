#[cfg(target_os = "macos")]
use muda::AboutMetadata;
use muda::{
    Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, CMD_OR_CTRL, Code, Modifiers},
};
use rocci_core::{Error, Result};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;

use crate::ShellEvent;

pub const NEW_WINDOW_ID: &str = "file.new-window";
pub const CLOSE_WINDOW_ID: &str = "file.close-window";
pub const RELOAD_ID: &str = "view.reload";
pub const WEB_INSPECTOR_ID: &str = "view.web-inspector";

pub struct MenuConfig<'a> {
    pub app_name: &'a str,
    pub version: Option<&'a str>,
    pub new_window: bool,
    pub reload: bool,
    pub devtools: bool,
}

pub struct NativeMenu {
    #[allow(dead_code)]
    menu: Menu,
}

impl NativeMenu {
    pub fn install(proxy: EventLoopProxy<ShellEvent>, config: MenuConfig<'_>) -> Result<Self> {
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = proxy.send_event(ShellEvent::Menu(event));
        }));

        let menu = Menu::new();

        #[cfg(target_os = "macos")]
        {
            let app = Submenu::new(config.app_name, true);
            app.append_items(&[
                &PredefinedMenuItem::about(
                    Some(&format!("About {}", config.app_name)),
                    Some(AboutMetadata {
                        name: Some(config.app_name.into()),
                        version: config.version.map(str::to_owned),
                        copyright: Some("Built with Rust, tao, wry, and Datastar".into()),
                        ..Default::default()
                    }),
                ),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(Some(&format!("Hide {}", config.app_name))),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(Some(&format!("Quit {}", config.app_name))),
            ])
            .map_err(menu_error)?;
            menu.append(&app).map_err(menu_error)?;
        }

        let file = Submenu::new("File", true);
        if config.new_window {
            file.append(&MenuItem::with_id(
                NEW_WINDOW_ID,
                "New Window",
                true,
                Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyN)),
            ))
            .map_err(menu_error)?;
        }
        file.append(&MenuItem::with_id(
            CLOSE_WINDOW_ID,
            "Close Window",
            true,
            Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyW)),
        ))
        .map_err(menu_error)?;
        menu.append(&file).map_err(menu_error)?;

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
        menu.append(&edit).map_err(menu_error)?;

        let view = Submenu::new("View", true);
        if config.reload {
            view.append(&MenuItem::with_id(
                RELOAD_ID,
                "Reload",
                true,
                Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyR)),
            ))
            .map_err(menu_error)?;
        }
        if config.devtools {
            view.append(&MenuItem::with_id(
                WEB_INSPECTOR_ID,
                "Toggle Web Inspector",
                true,
                Some(inspector_accelerator()),
            ))
            .map_err(menu_error)?;
        }
        #[cfg(target_os = "macos")]
        {
            view.append(&PredefinedMenuItem::separator())
                .map_err(menu_error)?;
            view.append(&PredefinedMenuItem::fullscreen(Some("Enter Full Screen")))
                .map_err(menu_error)?;
        }
        menu.append(&view).map_err(menu_error)?;

        #[cfg(target_os = "macos")]
        {
            let window = Submenu::new("Window", true);
            window
                .append_items(&[
                    &PredefinedMenuItem::minimize(None),
                    &PredefinedMenuItem::separator(),
                    &PredefinedMenuItem::bring_all_to_front(None),
                ])
                .map_err(menu_error)?;
            let help = Submenu::new("Help", true);
            menu.append_items(&[&window, &help]).map_err(menu_error)?;
            menu.init_for_nsapp();
            window.set_as_windows_menu_for_nsapp();
            help.set_as_help_menu_for_nsapp();
        }

        Ok(Self { menu })
    }

    pub fn attach(&self, window: &Window) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            unsafe { self.menu.init_for_hwnd(window.hwnd()) }.map_err(menu_error)?;
        }
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            self.menu
                .init_for_gtk_window(window.gtk_window(), window.default_vbox())
                .map_err(menu_error)?;
        }
        let _ = window;
        Ok(())
    }
}

pub fn is(event: &MenuEvent, expected: &str) -> bool {
    event.id() == &MenuId::new(expected)
}

fn inspector_accelerator() -> Accelerator {
    #[cfg(target_os = "macos")]
    {
        Accelerator::new(Some(Modifiers::META | Modifiers::ALT), Code::KeyI)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Accelerator::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyI)
    }
}

fn menu_error(error: impl std::fmt::Display) -> rocci_core::Error {
    Error::message(format!("failed to install the native menu: {error}"))
}
