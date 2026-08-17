pub mod html;
pub mod view;

pub use html::*;
pub use view::*;

pub const ROCCI_UI_TEMPLATE: &str = include_str!("../templates/RocciUi.rocci");
pub const BASE_CSS: &str = include_str!("themes/base.css");
