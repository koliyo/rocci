pub mod chrome {
    pub const BREADCRUMBS: &str = include_str!("../templates/chrome/Breadcrumbs.rocci");
    pub const NAV_LIST: &str = include_str!("../templates/chrome/NavList.rocci");
    pub const PAGE_OUTLINE: &str = include_str!("../templates/chrome/PageOutline.rocci");
}
pub mod html;
pub mod view;

pub const TOC_SCRIPT: &str = include_str!("../assets/toc.js");
pub const GOTO_SCRIPT: &str = include_str!("../assets/goto.js");
pub const COPY_SCRIPT: &str = include_str!("../assets/copy.js");
pub const RESIZE_SCRIPT: &str = include_str!("../assets/resize.js");
pub const HTML_ROC: &str = include_str!("../runtime/Html.roc");

pub fn chrome_script() -> String {
    format!("{GOTO_SCRIPT}\n{COPY_SCRIPT}\n{TOC_SCRIPT}\n{RESIZE_SCRIPT}")
}

pub use html::*;
pub use view::*;
