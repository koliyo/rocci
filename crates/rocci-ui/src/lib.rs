pub mod chrome {
    pub const BREADCRUMBS: &str = include_str!("../templates/chrome/Breadcrumbs.rocci");
    pub const NAV_LIST: &str = include_str!("../templates/chrome/NavList.rocci");
    pub const PAGE_OUTLINE: &str = include_str!("../templates/chrome/PageOutline.rocci");
}
pub mod html;
pub mod view;

pub const TOC_SCRIPT: &str = include_str!("../assets/toc.js");

pub use html::*;
pub use view::*;
