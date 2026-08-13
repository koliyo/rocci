mod python;
mod routes;
mod templates;

use std::env;

use anyhow::Result;
use rocci::{App, Config};
use tracing_subscriber::EnvFilter;

use python::PythonBackend;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let mut app = App::builder()
        .config(load_config()?)
        .embed_asset(
            "app.css",
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/app.css").as_slice(),
        )
        .embed_asset(
            "datastar.js",
            "text/javascript; charset=utf-8",
            include_bytes!("../../../assets/datastar.js").as_slice(),
        )
        .embed_asset(
            "htmx.min.js",
            "text/javascript; charset=utf-8",
            include_bytes!("../../../assets/htmx.min.js").as_slice(),
        )
        .serve_only(env::args().any(|argument| argument == "--serve-only"));

    match BackendKind::from_env_and_args()? {
        BackendKind::Rust => {
            app = app.router(routes::router());
        }
        BackendKind::Python => {
            app = app.backend(PythonBackend::default());
        }
    }

    app.run()?;
    Ok(())
}

fn load_config() -> rocci::Result<Config> {
    if let Ok(config) = Config::load() {
        return Ok(config);
    }
    if let Ok(executable) = env::current_exe()
        && let Some(resources) = executable
            .parent()
            .and_then(|path| path.parent())
            .map(|contents| contents.join("Resources/rocci.toml"))
        && resources.is_file()
    {
        return Config::from_file(resources);
    }
    Config::from_toml(include_str!("../../../rocci.toml"))
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum BackendKind {
    #[default]
    Rust,
    Python,
}

impl BackendKind {
    fn from_env_and_args() -> anyhow::Result<Self> {
        let mut selected = env::var("ROCCI_BACKEND").unwrap_or_else(|_| "rust".into());
        let mut args = env::args().skip(1);
        while let Some(argument) = args.next() {
            if argument == "--backend" {
                selected = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--backend requires rust or python"))?;
            } else if let Some(value) = argument.strip_prefix("--backend=") {
                selected = value.to_owned();
            }
        }
        Self::parse(&selected)
    }

    fn parse(value: &str) -> anyhow::Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "rust" => Ok(Self::Rust),
            "python" => Ok(Self::Python),
            other => anyhow::bail!("unknown backend {other:?}; expected rust or python"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_backend_names() {
        assert!(matches!(BackendKind::parse("rust"), Ok(BackendKind::Rust)));
        assert!(matches!(
            BackendKind::parse("PYTHON"),
            Ok(BackendKind::Python)
        ));
        assert!(BackendKind::parse("node").is_err());
    }
}
