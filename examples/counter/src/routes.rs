use std::{
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use axum::{
    Router,
    extract::State,
    response::{Html, Sse, sse::KeepAlive},
    routing::{get, post},
};
use datastar::prelude::PatchElements;
use futures_util::{Stream, stream};
use tokio::sync::broadcast;

use crate::templates;

#[derive(Clone)]
struct AppState {
    counter: Arc<AtomicU64>,
    counter_updates: broadcast::Sender<u64>,
}

pub fn router() -> Router {
    let (counter_updates, _) = broadcast::channel(32);
    let state = AppState {
        counter: Arc::new(AtomicU64::new(0)),
        counter_updates,
    };

    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/api/counter/events", get(counter_events))
        .route("/api/counter/increment", post(increment_counter))
        .route("/api/counter/reset", post(reset_counter))
        .route("/htmx", get(htmx_demo))
        .route("/htmx/counter/increment", post(htmx_increment))
        .with_state(state)
}

async fn index(State(state): State<AppState>) -> Html<String> {
    Html(templates::datastar_page(
        state.counter.load(Ordering::Relaxed),
    ))
}

async fn health() -> &'static str {
    "ok"
}

async fn increment_counter(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let count = state.counter.fetch_add(1, Ordering::Relaxed) + 1;
    let _ = state.counter_updates.send(count);
    one_patch(count)
}

async fn reset_counter(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    state.counter.store(0, Ordering::Relaxed);
    let _ = state.counter_updates.send(0);
    one_patch(0)
}

fn one_patch(
    count: u64,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let event = PatchElements::new(templates::datastar_counter(count)).write_as_axum_sse_event();
    Sse::new(stream::once(async move { Ok(event) }))
}

async fn counter_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let initial = state.counter.load(Ordering::Relaxed);
    let mut updates = state.counter_updates.subscribe();
    let counter = state.counter.clone();

    let events = async_stream::stream! {
        yield Ok(PatchElements::new(templates::datastar_counter(initial)).write_as_axum_sse_event());
        loop {
            let count = match updates.recv().await {
                Ok(count) => count,
                Err(broadcast::error::RecvError::Lagged(_)) => counter.load(Ordering::Relaxed),
                Err(broadcast::error::RecvError::Closed) => break,
            };
            yield Ok(PatchElements::new(templates::datastar_counter(count)).write_as_axum_sse_event());
        }
    };

    Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

async fn htmx_demo(State(state): State<AppState>) -> Html<String> {
    Html(templates::htmx_page(state.counter.load(Ordering::Relaxed)))
}

async fn htmx_increment(State(state): State<AppState>) -> Html<String> {
    let count = state.counter.fetch_add(1, Ordering::Relaxed) + 1;
    let _ = state.counter_updates.send(count);
    Html(templates::htmx_counter(count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use rocci::{Config, HttpContext, Session, WindowId, wrap_router};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tower::ServiceExt;

    const ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 43123);
    const TOKEN: &str = "test-token";

    fn wrapped() -> axum::Router {
        let context = HttpContext::new(&Config::default(), ADDRESS, None);
        context.sessions.insert(Session {
            window_id: WindowId::new("main"),
            token: TOKEN.into(),
            start_url: "/".into(),
        });
        wrap_router(router(), context)
    }

    fn request(method: http::Method, uri: &str) -> axum::extract::Request {
        axum::extract::Request::builder()
            .method(method)
            .uri(uri)
            .header(http::header::HOST, ADDRESS.to_string())
            .header(http::header::COOKIE, format!("rocci_session={TOKEN}"))
            .header(http::header::ORIGIN, format!("http://{ADDRESS}"))
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn datastar_action_returns_a_patch_elements_event() {
        let response = wrapped()
            .oneshot(request(http::Method::POST, "/api/counter/increment"))
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert!(
            response.headers()[http::header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/event-stream")
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("event: datastar-patch-elements"));
        assert!(body.contains("<output>1</output>"));
    }

    #[tokio::test]
    async fn htmx_action_returns_an_html_fragment() {
        let response = wrapped()
            .oneshot(request(http::Method::POST, "/htmx/counter/increment"))
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), br#"<output id="htmx-counter">1</output>"#);
    }
}
