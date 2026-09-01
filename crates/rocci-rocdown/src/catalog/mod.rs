mod graph;
mod nav;
mod resolve;
mod types;

pub use nav::format_diagnostics;
pub(crate) use nav::{first_nav_item, section_contains};
#[allow(unused_imports)]
pub use resolve::page_route;
pub use resolve::{
    canonical_route, derived_route, is_collection_id, resolve, route_output_path,
    with_trailing_slash,
};
pub use types::*;

#[cfg(test)]
mod tests;
