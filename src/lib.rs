//! HTTP runtime for a Datastar-first desktop application.
//!
//! The webview is deliberately a normal HTTP client. There is no privileged
//! JavaScript bridge: backend actions are HTTP requests and backend-driven UI
//! updates are Datastar server-sent events.

use std::{
    convert::Infallible,
    net::{Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result};
use axum::{
    Router,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response, Sse, sse::KeepAlive},
    routing::{get, post},
};
use datastar::prelude::PatchElements;
use futures_util::{Stream, stream};
use tokio::{net::TcpListener, sync::broadcast, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

mod templates;

const DATASTAR_JS: &[u8] = include_bytes!("../assets/datastar.js");
const HTMX_JS: &[u8] = include_bytes!("../assets/htmx.min.js");
const STYLES: &str = include_str!("../assets/app.css");

#[derive(Clone)]
struct AppState {
    session_token: Arc<str>,
    expected_host: Arc<str>,
    counter: Arc<AtomicU64>,
    counter_updates: broadcast::Sender<u64>,
}

/// A loopback HTTP server owned by the desktop process.
pub struct DesktopServer {
    address: SocketAddr,
    bootstrap_url: String,
    shutdown: CancellationToken,
    task: Option<JoinHandle<Result<()>>>,
}

impl DesktopServer {
    /// Bind an ephemeral IPv4 loopback port and begin serving the application.
    pub async fn start() -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .context("failed to bind the loopback HTTP server")?;
        let address = listener.local_addr()?;
        let session_token = Uuid::new_v4().simple().to_string();
        let app = build_router(address, session_token.clone());
        let shutdown = CancellationToken::new();
        let shutdown_signal = shutdown.clone();

        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal.cancelled_owned())
                .await
                .context("desktop HTTP server failed")
        });

        Ok(Self {
            address,
            bootstrap_url: format!("http://{address}/_roc/bootstrap/{session_token}"),
            shutdown,
            task: Some(task),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// The one-time-looking bootstrap URL establishes an HttpOnly session
    /// cookie and immediately redirects to `/`, removing the token from the URL.
    pub fn bootstrap_url(&self) -> &str {
        &self.bootstrap_url
    }

    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }

    pub async fn wait(mut self) -> Result<()> {
        self.task
            .take()
            .expect("server task is present")
            .await
            .context("HTTP server task panicked")?
    }
}

impl Drop for DesktopServer {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

fn build_router(address: SocketAddr, session_token: String) -> Router {
    let (counter_updates, _) = broadcast::channel(32);
    let state = AppState {
        session_token: session_token.into(),
        expected_host: address.to_string().into(),
        counter: Arc::new(AtomicU64::new(0)),
        counter_updates,
    };

    let protected = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/assets/app.css", get(styles))
        .route("/assets/datastar.js", get(datastar_js))
        .route("/assets/htmx.min.js", get(htmx_js))
        .route("/api/counter/events", get(counter_events))
        .route("/api/counter/increment", post(increment_counter))
        .route("/api/counter/reset", post(reset_counter))
        .route("/htmx", get(htmx_demo))
        .route("/htmx/counter/increment", post(htmx_increment))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_session,
        ));

    Router::new()
        .route("/_roc/bootstrap/{token}", get(bootstrap))
        .merge(protected)
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}

async fn bootstrap(
    State(state): State<AppState>,
    Path(token): Path<String>,
    headers: HeaderMap,
) -> Response {
    if !valid_host(&headers, &state) || token.as_bytes() != state.session_token.as_bytes() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let cookie = format!(
        "roc_session={}; HttpOnly; SameSite=Strict; Path=/",
        state.session_token
    );
    let mut response = Redirect::to("/").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("UUID cookie is a valid header value"),
    );
    response
}

async fn require_session(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if !valid_host(request.headers(), &state)
        || !valid_cookie(request.headers(), &state)
        || !valid_origin(&request, &state)
    {
        return (StatusCode::UNAUTHORIZED, "desktop session required").into_response();
    }

    next.run(request).await
}

async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            // Datastar compiles declarative expressions with `Function`, which
            // requires `unsafe-eval`. Scripts themselves remain self-hosted.
            "default-src 'self'; script-src 'self' 'unsafe-eval'; connect-src 'self'; img-src 'self' data:; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    response
}

fn valid_host(headers: &HeaderMap, state: &AppState) -> bool {
    headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| host == state.expected_host.as_ref())
}

fn valid_origin(request: &Request, state: &AppState) -> bool {
    if matches!(
        *request.method(),
        http::Method::GET | http::Method::HEAD | http::Method::OPTIONS
    ) {
        return true;
    }

    request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| origin == format!("http://{}", state.expected_host))
}

fn valid_cookie(headers: &HeaderMap, state: &AppState) -> bool {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|cookies| {
            cookies.split(';').any(|cookie| {
                let mut parts = cookie.trim().splitn(2, '=');
                parts.next() == Some("roc_session")
                    && parts
                        .next()
                        .is_some_and(|value| value.as_bytes() == state.session_token.as_bytes())
            })
        })
}

async fn index(State(state): State<AppState>) -> Html<String> {
    Html(templates::datastar_page(
        state.counter.load(Ordering::Relaxed),
    ))
}

async fn health() -> &'static str {
    "ok"
}

async fn styles() -> impl IntoResponse {
    static_asset("text/css; charset=utf-8", STYLES.as_bytes())
}

async fn datastar_js() -> impl IntoResponse {
    static_asset("text/javascript; charset=utf-8", DATASTAR_JS)
}

async fn htmx_js() -> impl IntoResponse {
    static_asset("text/javascript; charset=utf-8", HTMX_JS)
}

fn static_asset(content_type: &'static str, bytes: &'static [u8]) -> Response {
    (
        [(header::CONTENT_TYPE, HeaderValue::from_static(content_type))],
        Body::from(bytes),
    )
        .into_response()
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
    use http_body_util::BodyExt;
    use std::net::IpAddr;
    use tower::ServiceExt;

    const ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 43123);
    const TOKEN: &str = "test-token";

    fn request(method: http::Method, uri: &str, authenticated: bool) -> Request {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::HOST, ADDRESS.to_string());
        if authenticated {
            builder = builder
                .header(header::COOKIE, format!("roc_session={TOKEN}"))
                .header(header::ORIGIN, format!("http://{ADDRESS}"));
        }
        builder.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn bootstrap_sets_session_and_hides_token_with_redirect() {
        let response = build_router(ADDRESS, TOKEN.into())
            .oneshot(request(
                http::Method::GET,
                "/_roc/bootstrap/test-token",
                false,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], "/");
        assert!(
            response.headers()[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .contains("HttpOnly; SameSite=Strict")
        );
    }

    #[tokio::test]
    async fn protected_routes_reject_requests_without_the_session() {
        let response = build_router(ADDRESS, TOKEN.into())
            .oneshot(request(http::Method::GET, "/", false))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn datastar_action_returns_a_patch_elements_event() {
        let response = build_router(ADDRESS, TOKEN.into())
            .oneshot(request(http::Method::POST, "/api/counter/increment", true))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()[header::CONTENT_TYPE]
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
        let response = build_router(ADDRESS, TOKEN.into())
            .oneshot(request(http::Method::POST, "/htmx/counter/increment", true))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), br#"<output id="htmx-counter">1</output>"#);
    }

    #[tokio::test]
    async fn content_security_policy_allows_datastar_expressions() {
        let response = build_router(ADDRESS, TOKEN.into())
            .oneshot(request(http::Method::GET, "/", true))
            .await
            .unwrap();

        let policy = response.headers()[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap();
        assert!(policy.contains("script-src 'self' 'unsafe-eval'"));
        assert!(String::from_utf8_lossy(DATASTAR_JS).contains("Function("));
    }

    #[tokio::test]
    async fn rejects_a_rebound_host_even_with_a_valid_cookie() {
        let request = Request::builder()
            .uri("/")
            .header(header::HOST, "attacker.example")
            .header(header::COOKIE, format!("roc_session={TOKEN}"))
            .body(Body::empty())
            .unwrap();
        let response = build_router(ADDRESS, TOKEN.into())
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_a_cross_origin_mutation_even_with_a_valid_cookie() {
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/counter/increment")
            .header(header::HOST, ADDRESS.to_string())
            .header(header::ORIGIN, "http://127.0.0.1:9999")
            .header(header::COOKIE, format!("roc_session={TOKEN}"))
            .body(Body::empty())
            .unwrap();
        let response = build_router(ADDRESS, TOKEN.into())
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
