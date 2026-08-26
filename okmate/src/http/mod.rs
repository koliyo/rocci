use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use axum::Router;
use tower_http::services::ServeDir;

pub fn bind_addr(public: bool, port: u16) -> SocketAddr {
    let ip = if public {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    } else {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    };
    SocketAddr::new(ip, port)
}

pub fn router(output: impl AsRef<Path>) -> Router {
    let output = output.as_ref().to_path_buf();
    Router::new()
        .nest_service("/__okmate", ServeDir::new(output.join("__okmate")))
        .fallback_service(ServeDir::new(output).append_index_html_on_directories(true))
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
