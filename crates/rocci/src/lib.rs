//! Facade crate for the Rocci desktop runtime.
//!
//! Applications configure windows and mount an Axum router (or another
//! [`Backend`]) through [`App`].

mod builder;

pub use builder::App;
pub use rocci_core::{
    AppConfig, AppEvent, AssetConfig, Backend, BundleConfig, BundleResource, Config,
    DevelopmentConfig, Error, ExternalBackend, Hooks, HttpConfig, ManagedState, Result,
    RunningBackend, SecurityConfig, Session, SessionStore, WindowConfig, WindowEvent, WindowId,
    join_origin,
};
pub use rocci_http::{
    Asset, AssetMap, AssetSource, HttpContext, HttpServer, Router, SESSION_COOKIE, wrap_router,
};
