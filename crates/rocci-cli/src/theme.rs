use std::path::Path;

use clap::Args;
use rocci_rocdown::index_pages_in_dir;
use rocci_theme::{ColorSchemePolicy, ThemeOptions};

#[derive(Args, Clone, Debug, Default)]
pub struct ThemeArgs {
    /// Theme name (`paper`, `rocci`) or path to a CSS file / theme directory.
    /// Named themes are loaded from `~/.rocci/themes`.
    #[arg(long, env = "ROCCI_THEME", value_name = "NAME|PATH")]
    pub theme: Option<String>,
    /// Force `light`, `dark`, or `auto` (follows the OS). Overridden by `@page.color_scheme`.
    #[arg(
        long = "color-scheme",
        env = "ROCCI_COLOR_SCHEME",
        value_name = "SCHEME",
        value_parser = ["auto", "light", "dark"]
    )]
    pub color_scheme: Option<String>,
}

impl ThemeArgs {
    pub fn from_env() -> Self {
        Self {
            theme: std::env::var("ROCCI_THEME")
                .ok()
                .filter(|value| !value.is_empty()),
            color_scheme: std::env::var("ROCCI_COLOR_SCHEME")
                .ok()
                .filter(|value| !value.is_empty()),
        }
    }

    pub fn compile_options(&self, input: Option<&Path>) -> rocci_rocdown::CompileOptions {
        compile_options(input, self)
    }
}

pub fn compile_options(input: Option<&Path>, args: &ThemeArgs) -> rocci_rocdown::CompileOptions {
    let env_theme = std::env::var("ROCCI_THEME").ok();
    let env_scheme = std::env::var("ROCCI_COLOR_SCHEME").ok();
    let theme = args
        .theme
        .clone()
        .or_else(|| env_theme.filter(|value| !value.is_empty()));
    let color_scheme = args
        .color_scheme
        .as_deref()
        .or(env_scheme.as_deref())
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<ColorSchemePolicy>().ok());
    let pages = input
        .and_then(Path::parent)
        .map(index_pages_in_dir)
        .unwrap_or_default();
    rocci_rocdown::CompileOptions {
        theme: ThemeOptions {
            default_id: theme,
            color_scheme,
            source_dir: input.and_then(Path::parent).map(Path::to_path_buf),
        },
        pages,
        ..rocci_rocdown::CompileOptions::default()
    }
}
