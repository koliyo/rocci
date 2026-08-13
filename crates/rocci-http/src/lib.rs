//! Loopback HTTP server for Rocci applications.
//!
//! The application supplies an Axum [`Router`] (or any Tower service converted
//! into one). This crate binds an ephemeral loopback port, wraps the router
//! with bootstrap/session/security middleware, and serves optional assets.

mod assets;
mod server;
mod session;

pub use assets::{Asset, AssetMap, AssetSource};
pub use axum::Router;
pub use server::{HttpContext, HttpServer, wrap_router};
pub use session::{SESSION_COOKIE, parse_session_cookie};
