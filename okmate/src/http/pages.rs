use std::convert::Infallible;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response, Sse, sse::Event};
use datastar::prelude::PatchElements;
use futures_util::stream;

use crate::http::AppState;
use crate::site;

pub async fn datastar_get(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if request.method() != Method::GET || !is_datastar(request.headers()) {
        return next.run(request).await;
    }
    let path = request.uri().path();
    if path.starts_with("/__okmate") || path.ends_with(".json") {
        return next.run(request).await;
    }
    let Some(fragment) = render_main_fragment(&state, path) else {
        return next.run(request).await;
    };
    let patch = PatchElements::new(fragment);
    Sse::new(stream::once(async move {
        Ok::<Event, Infallible>(patch.write_as_axum_sse_event())
    }))
    .into_response()
}

pub fn is_datastar(headers: &HeaderMap) -> bool {
    headers
        .get("datastar-request")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn render_main_fragment(state: &AppState, path: &str) -> Option<String> {
    let bundle = okf::load(&state.root, state.profile).ok()?;
    let mut document = site::page_for_route(&bundle, path)?;
    if document.page_kind == "settings" {
        let config = crate::config::load_or_default(&state.config_path);
        document.config_path = state.config_path.display().to_string();
        document.settings_roots = crate::http::settings_roots(&config);
    }
    document.render_main_fragment().ok()
}
