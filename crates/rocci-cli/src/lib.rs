pub mod browse;
pub mod bundle;
pub mod datastar_asset;
pub mod dev_server;
pub(crate) mod dispatch;
pub mod driver;
pub mod error_page;
pub mod http_module;
pub mod inspect;
pub(crate) mod inspector;
pub mod logs;
pub mod native_target;
pub mod path_hint;
pub mod playground;
pub(crate) mod playground_compile;
pub(crate) mod playground_html;
pub mod profile;
pub(crate) mod roc_module;
pub mod rocci_test;
pub mod run;
pub(crate) mod runtime_assets;
pub mod serve;
pub mod style;
pub mod view;

pub use playground_html::render_file;

pub fn resolve_platform_pin(spec: Option<&str>) -> anyhow::Result<Option<String>> {
    dispatch::resolve_platform_pin(spec).map_err(|err| anyhow::anyhow!("{err}"))
}
