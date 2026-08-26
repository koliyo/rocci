use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use axum::Router;
use axum::middleware;
use axum::routing::post;
use okf::Profile;
use tower_http::services::ServeDir;

mod pages;
mod settings;

pub use settings::{render_fragment, render_page, settings_roots};

#[derive(Clone)]
pub struct AppState {
    pub output: PathBuf,
    pub root: PathBuf,
    pub profile: Profile,
    pub config_path: PathBuf,
}

pub fn bind_addr(public: bool, port: u16) -> SocketAddr {
    let ip = if public {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    } else {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    };
    SocketAddr::new(ip, port)
}

pub fn router(state: AppState) -> Router {
    let output = state.output.clone();
    Router::new()
        .route("/__okmate/settings", post(settings::post))
        .nest_service("/__okmate", ServeDir::new(output.join("__okmate")))
        .fallback_service(ServeDir::new(output).append_index_html_on_directories(true))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            pages::datastar_get,
        ))
        .with_state(state)
}

pub fn output_path(output: Option<&Path>, root: &Path) -> PathBuf {
    output.map(Path::to_path_buf).unwrap_or_else(|| {
        std::env::temp_dir().join(format!(
            "okmate-view-{}-{}",
            std::process::id(),
            root.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("bundle")
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bind_is_localhost() {
        let addr = bind_addr(false, 8000);
        assert!(addr.ip().is_loopback());
        assert_eq!(addr.port(), 8000);
    }

    #[test]
    fn public_bind_is_unspecified() {
        let addr = bind_addr(true, 9000);
        assert!(addr.ip().is_unspecified());
    }
}
