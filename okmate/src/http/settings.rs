use std::collections::BTreeMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::{Result, bail};
use axum::extract::{ConnectInfo, Form, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response, Sse, sse::Event};
use datastar::prelude::PatchElements;
use futures_util::stream;
use okf::Bundle;

use crate::config::{
    DirectoryRoot, GitRoot, Incoming, RootConfig, UserConfig, expand_tilde, save, valid_git_url,
    valid_id,
};
use crate::http::AppState;
use crate::site;
use crate::views::{Document, NavNode, SettingsRoot};

pub async fn post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(fields): Form<BTreeMap<String, String>>,
) -> Response {
    if !addr.ip().is_loopback() {
        return (StatusCode::FORBIDDEN, "settings POST is loopback-only").into_response();
    }
    let message =
        apply_action(&state.config_path, &fields).unwrap_or_else(|error| error.to_string());
    let config = crate::config::load_or_default(&state.config_path);
    let bundle = okf::load(&state.root, state.profile).ok();
    let fragment = render_fragment(bundle.as_ref(), &config, Some(&message), &state.config_path);
    if is_datastar(&headers) {
        let patch = PatchElements::new(fragment);
        return Sse::new(stream::once(async move {
            Ok::<Event, Infallible>(patch.write_as_axum_sse_event())
        }))
        .into_response();
    }
    let html = render_page(bundle.as_ref(), &config, Some(&message), &state.config_path);
    let _ = write_settings_page(&state, &html);
    axum::response::Html(html).into_response()
}

fn is_datastar(headers: &HeaderMap) -> bool {
    headers
        .get("datastar-request")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

pub fn settings_roots(config: &UserConfig) -> Vec<SettingsRoot> {
    config
        .roots
        .iter()
        .map(|root| match root {
            RootConfig::Directory(dir) => SettingsRoot {
                id: dir.id.clone(),
                kind: "folder".into(),
                detail: expand_tilde(&dir.path).display().to_string(),
                incoming: dir.incoming.as_str().into(),
                token_env: String::new(),
                has_token: false,
                warning: index_md_warning(dir),
            },
            RootConfig::Git(git) => SettingsRoot {
                id: git.id.clone(),
                kind: "git".into(),
                detail: git.url.clone(),
                incoming: git.incoming.as_str().into(),
                token_env: git.token_env.clone().unwrap_or_default(),
                has_token: git.token.as_ref().is_some_and(|token| !token.is_empty()),
                warning: String::new(),
            },
        })
        .collect()
}

pub fn render_page(
    bundle: Option<&Bundle>,
    config: &UserConfig,
    message: Option<&str>,
    config_file: &Path,
) -> String {
    settings_document(bundle, config, message, config_file)
        .render_settings()
        .unwrap_or_else(|error| error.to_string())
}

pub fn render_fragment(
    bundle: Option<&Bundle>,
    config: &UserConfig,
    message: Option<&str>,
    config_file: &Path,
) -> String {
    settings_document(bundle, config, message, config_file)
        .render_settings_fragment()
        .unwrap_or_else(|error| error.to_string())
}

fn settings_document(
    bundle: Option<&Bundle>,
    config: &UserConfig,
    message: Option<&str>,
    config_file: &Path,
) -> Document {
    let mut document = if let Some(bundle) = bundle {
        site::settings_shell(bundle)
    } else {
        Document {
            title: "Knowledge roots".into(),
            nav: vec![NavNode {
                href: "/settings/".into(),
                title: "Settings".into(),
                current: true,
                open: false,
                children: Vec::new(),
            }],
            toc: Vec::new(),
            article_html: String::new(),
            concept_type: String::new(),
            status: String::new(),
            authority: String::new(),
            review_rows: Vec::new(),
            message: String::new(),
            config_path: String::new(),
            settings_roots: Vec::new(),
        }
    };
    document.message = message.unwrap_or("").to_string();
    document.config_path = config_file.display().to_string();
    document.settings_roots = settings_roots(config);
    document
}

fn apply_action(path: &Path, fields: &BTreeMap<String, String>) -> Result<String> {
    let action = fields.get("action").map(String::as_str).unwrap_or("");
    let mut config = crate::config::load_or_default(path);
    let message = match action {
        "add_directory" => {
            let id = required(fields, "id")?;
            if !valid_id(id) {
                bail!("invalid root id `{id}`");
            }
            if config.roots.iter().any(|root| root.id() == id) {
                bail!("duplicate root id `{id}`");
            }
            let folder = required(fields, "path")?;
            config.roots.push(RootConfig::Directory(DirectoryRoot {
                id: id.to_string(),
                path: folder.to_string(),
                incoming: Incoming::Allow,
            }));
            format!("added directory root `{id}`")
        }
        "add_git" => {
            let id = required(fields, "id")?;
            if !valid_id(id) {
                bail!("invalid root id `{id}`");
            }
            if config.roots.iter().any(|root| root.id() == id) {
                bail!("duplicate root id `{id}`");
            }
            let url = required(fields, "url")?;
            if !valid_git_url(url) {
                bail!("git root `{id}` has unsupported url `{url}`");
            }
            let token = fields
                .get("token")
                .cloned()
                .filter(|value| !value.is_empty());
            config.roots.push(RootConfig::Git(GitRoot {
                id: id.to_string(),
                url: url.to_string(),
                branch: fields
                    .get("branch")
                    .cloned()
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "main".into()),
                bundle: fields.get("bundle").cloned().unwrap_or_default(),
                token,
                token_env: fields
                    .get("token_env")
                    .cloned()
                    .filter(|value| !value.is_empty()),
                incoming: Incoming::Deny,
                poll: None,
            }));
            format!("added git root `{id}`")
        }
        "remove" => {
            let id = required(fields, "id")?;
            let before = config.roots.len();
            config.roots.retain(|root| root.id() != id);
            if config.roots.len() == before {
                bail!("unknown root id `{id}`");
            }
            format!("removed `{id}`")
        }
        "incoming" => {
            let id = required(fields, "id")?;
            let incoming = Incoming::parse(required(fields, "incoming")?)?;
            let root = config
                .roots
                .iter_mut()
                .find(|root| root.id() == id)
                .ok_or_else(|| anyhow::anyhow!("unknown root id `{id}`"))?;
            root.set_incoming(incoming);
            format!("updated incoming for `{id}`")
        }
        other => bail!("unknown settings action `{other}`"),
    };
    save(&config, path)?;
    Ok(message)
}

fn required<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str> {
    fields
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing {key}"))
}

fn index_md_warning(dir: &DirectoryRoot) -> String {
    let path = expand_tilde(&dir.path);
    if !path.is_dir() {
        return "Path is missing or not a directory.".into();
    }
    if !path.join("index.md").is_file() {
        return "No index.md in this folder yet.".into();
    }
    String::new()
}

fn write_settings_page(state: &AppState, html: &str) -> Result<()> {
    let path = state.output.join("settings").join("index.html");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, html)?;
    Ok(())
}
