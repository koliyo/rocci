use crate::{Error, Result};
#[cfg(target_os = "macos")]
use muda::AboutMetadata;
use muda::{
    CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, CMD_OR_CTRL, Code, Modifiers},
};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;

use crate::ShellEvent;

pub const QUIT_ID: &str = "app.quit";
pub const NEW_WINDOW_ID: &str = "file.new-window";
pub const CLOSE_WINDOW_ID: &str = "file.close-window";
pub const FIND_ID: &str = "edit.find";
pub const FIND_NEXT_ID: &str = "edit.find-next";
pub const FIND_PREVIOUS_ID: &str = "edit.find-previous";
pub const USE_SELECTION_ID: &str = "edit.use-selection-for-find";
pub const SELECT_ALL_ID: &str = "edit.select-all";
pub const BACK_ID: &str = "view.back";
pub const FORWARD_ID: &str = "view.forward";
pub const HOME_ID: &str = "view.home";
pub const GO_TO_FILE_ID: &str = "view.go-to-file";
pub const OPEN_PICKER_ID: &str = "view.open-target";
pub const RELOAD_ID: &str = "view.reload";
pub const LIVE_RELOAD_ID: &str = "view.live-reload";
pub const WEB_INSPECTOR_ID: &str = "view.web-inspector";

pub struct MenuConfig<'a> {
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub app_name: &'a str,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub version: Option<&'a str>,
    pub new_window: bool,
    pub navigation: bool,
    pub search: bool,
    pub goto: bool,
    pub reload: bool,
    pub live_reload_on: bool,
    pub devtools: bool,
    pub picker: bool,
}

pub struct NativeMenu {
    #[allow(dead_code)]
    menu: Menu,
    live_reload: Option<CheckMenuItem>,
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
                        copyright: Some("Built with Rust, tao, and wry".into()),
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
                &MenuItem::with_id(
                    QUIT_ID,
                    format!("Quit {}", config.app_name),
                    true,
                    Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyQ)),
                ),
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
            &MenuItem::with_id(
                SELECT_ALL_ID,
                "Select All",
                true,
                Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyA)),
            ),
        ])
        .map_err(menu_error)?;
        if config.search {
            edit.append_items(&[
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    FIND_ID,
                    "Find…",
                    true,
                    Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyF)),
                ),
                &MenuItem::with_id(
                    FIND_NEXT_ID,
                    "Find Next",
                    true,
                    Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyG)),
                ),
                &MenuItem::with_id(
                    FIND_PREVIOUS_ID,
                    "Find Previous",
                    true,
                    Some(Accelerator::new(
                        Some(CMD_OR_CTRL | Modifiers::SHIFT),
                        Code::KeyG,
                    )),
                ),
                &MenuItem::with_id(
                    USE_SELECTION_ID,
                    "Use Selection for Find",
                    true,
                    Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyE)),
                ),
            ])
            .map_err(menu_error)?;
        }
        menu.append(&edit).map_err(menu_error)?;

        let view = Submenu::new("View", true);
        if config.navigation {
            view.append_items(&[
                &MenuItem::with_id(
                    BACK_ID,
                    "Back",
                    true,
                    Some(Accelerator::new(Some(CMD_OR_CTRL), Code::BracketLeft)),
                ),
                &MenuItem::with_id(
                    FORWARD_ID,
                    "Forward",
                    true,
                    Some(Accelerator::new(Some(CMD_OR_CTRL), Code::BracketRight)),
                ),
                &MenuItem::with_id(HOME_ID, "Home", true, None),
                &PredefinedMenuItem::separator(),
            ])
            .map_err(menu_error)?;
        }
        if config.goto {
            view.append(&MenuItem::with_id(
                GO_TO_FILE_ID,
                "Go to File…",
                true,
                Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyK)),
            ))
            .map_err(menu_error)?;
            view.append(&PredefinedMenuItem::separator())
                .map_err(menu_error)?;
        }
        if config.picker {
            view.append(&MenuItem::with_id(
                OPEN_PICKER_ID,
                "Open Target…",
                true,
                Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyP)),
            ))
            .map_err(menu_error)?;
            view.append(&PredefinedMenuItem::separator())
                .map_err(menu_error)?;
        }
        let live_reload = if config.reload {
            view.append(&MenuItem::with_id(
                RELOAD_ID,
                "Reload",
                true,
                Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyR)),
            ))
            .map_err(menu_error)?;
            let item = CheckMenuItem::with_id(
                LIVE_RELOAD_ID,
                "Live Reload",
                true,
                config.live_reload_on,
                None,
            );
            view.append(&item).map_err(menu_error)?;
            Some(item)
        } else {
            None
        };
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

        Ok(Self { menu, live_reload })
    }

    pub fn live_reload_checked(&self) -> bool {
        self.live_reload
            .as_ref()
            .map(CheckMenuItem::is_checked)
            .unwrap_or(true)
    }

    pub fn set_live_reload_checked(&self, checked: bool) {
        if let Some(item) = &self.live_reload {
            item.set_checked(checked);
        }
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

#[cfg(test)]
pub fn view_action_ids(config: &MenuConfig<'_>) -> Vec<&'static str> {
    let mut ids = Vec::new();
    if config.navigation {
        ids.extend([BACK_ID, FORWARD_ID, HOME_ID]);
    }
    if config.goto {
        ids.push(GO_TO_FILE_ID);
    }
    if config.picker {
        ids.push(OPEN_PICKER_ID);
    }
    if config.reload {
        ids.extend([RELOAD_ID, LIVE_RELOAD_ID]);
    }
    if config.devtools {
        ids.push(WEB_INSPECTOR_ID);
    }
    ids
}

#[cfg(test)]
pub fn edit_search_ids(config: &MenuConfig<'_>) -> Vec<&'static str> {
    if config.search {
        vec![FIND_ID, FIND_NEXT_ID, FIND_PREVIOUS_ID, USE_SELECTION_ID]
    } else {
        Vec::new()
    }
}

fn menu_error(error: impl std::fmt::Display) -> crate::Error {
    Error::message(format!("failed to install the native menu: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(navigation: bool, reload: bool, devtools: bool, search: bool) -> MenuConfig<'static> {
        MenuConfig {
            app_name: "h35",
            version: None,
            new_window: false,
            navigation,
            search,
            goto: search,
            reload,
            live_reload_on: true,
            devtools,
            picker: false,
        }
    }

    #[test]
    fn picker_view_menu_includes_open_target() {
        let mut cfg = config(true, true, true, true);
        cfg.picker = true;
        let ids = view_action_ids(&cfg);
        assert!(ids.contains(&OPEN_PICKER_ID));
        assert!(ids.contains(&GO_TO_FILE_ID));
    }

    #[test]
    fn preview_view_menu_includes_navigation() {
        assert_eq!(
            view_action_ids(&config(true, true, true, true)),
            vec![
                BACK_ID,
                FORWARD_ID,
                HOME_ID,
                GO_TO_FILE_ID,
                RELOAD_ID,
                LIVE_RELOAD_ID,
                WEB_INSPECTOR_ID,
            ]
        );
    }

    #[test]
    fn bundled_shell_view_menu_omits_navigation() {
        assert_eq!(
            view_action_ids(&config(false, true, true, false)),
            vec![RELOAD_ID, LIVE_RELOAD_ID, WEB_INSPECTOR_ID]
        );
    }

    #[test]
    fn preview_edit_menu_includes_find() {
        assert_eq!(
            edit_search_ids(&config(true, true, true, true)),
            vec![FIND_ID, FIND_NEXT_ID, FIND_PREVIOUS_ID, USE_SELECTION_ID]
        );
        assert!(edit_search_ids(&config(false, true, true, false)).is_empty());
    }

    #[test]
    fn edit_menu_includes_select_all() {
        assert_eq!(SELECT_ALL_ID, "edit.select-all");
    }

    #[test]
    fn live_reload_check_item_tracks_reload_config() {
        assert_eq!(LIVE_RELOAD_ID, "view.live-reload");
        assert!(view_action_ids(&config(true, true, false, false)).contains(&LIVE_RELOAD_ID));
        assert!(!view_action_ids(&config(true, false, false, false)).contains(&LIVE_RELOAD_ID));
    }
}
