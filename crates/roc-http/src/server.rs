use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use roc_core::{
    Config, Error, Result, RunningBackend, Session, SessionStore, WindowId, join_origin,
};
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    assets::AssetSource,
    session::{parse_session_cookie, session_cookie_header},
};

#[derive(Clone)]
pub struct HttpContext {
    pub sessions: SessionStore,
    expected_host: Arc<str>,
    expected_origin: Arc<str>,
    extra_origins: Arc<Vec<String>>,
    csp: Arc<str>,
    assets: Option<AssetSource>,
}

impl HttpContext {
    pub fn new(config: &Config, address: SocketAddr, assets: Option<AssetSource>) -> Self {
        let expected_host: Arc<str> = address.to_string().into();
        let expected_origin: Arc<str> = format!("http://{expected_host}").into();
        let mut extra_origins = config.security.allowed_origins.clone();
        if let Some(frontend_url) = &config.development.frontend_url {
            extra_origins.push(frontend_url.trim_end_matches('/').to_owned());
        }
        Self {
            sessions: SessionStore::new(),
            expected_host,
            expected_origin,
            extra_origins: extra_origins.into(),
            csp: config.csp().into(),
            assets,
        }
    }

    pub fn expected_host(&self) -> &str {
        &self.expected_host
    }
}

/// In-process Axum server that owns loopback binding, sessions, and shutdown.
pub struct HttpServer {
    origin: String,
    context: HttpContext,
    shutdown: CancellationToken,
    task: Option<JoinHandle<Result<()>>>,
}

impl HttpServer {
    pub async fn start(
        config: Config,
        router: Router,
        assets: Option<AssetSource>,
    ) -> Result<Self> {
        let host: IpAddr = config
            .http
            .host
            .parse()
            .map_err(|error| Error::config(format!("invalid http.host: {error}")))?;
        let listener = TcpListener::bind((host, config.http.port))
            .await
            .map_err(|error| {
                Error::backend(format!("failed to bind the loopback HTTP server: {error}"))
            })?;
        let address = listener.local_addr()?;
        let context = HttpContext::new(&config, address, assets);
        let app = wrap_router(router, context.clone());
        let shutdown = CancellationToken::new();
        let shutdown_signal = shutdown.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal.cancelled_owned())
                .await
                .map_err(|error| Error::backend(format!("desktop HTTP server failed: {error}")))
        });

        Ok(Self {
            origin: format!("http://{address}"),
            context,
            shutdown,
            task: Some(task),
        })
    }

    pub fn context(&self) -> &HttpContext {
        &self.context
    }

    pub fn address(&self) -> &str {
        self.origin.trim_start_matches("http://")
    }

    pub async fn stop(mut self) -> Result<()> {
        self.shutdown.cancel();
        let Some(task) = self.task.take() else {
            return Ok(());
        };
        match tokio::time::timeout(Duration::from_secs(5), task).await {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => Err(Error::backend(format!("HTTP server task failed: {error}"))),
            Err(_) => Err(Error::backend("HTTP server shutdown timed out")),
        }
    }
}

impl RunningBackend for HttpServer {
    fn origin(&self) -> &str {
        &self.origin
    }

    fn attach_window(&self, window: &WindowId, start_url: &str) -> Result<String> {
        let session = self.context.sessions.create(window.clone(), start_url);
        Ok(format!(
            "{}/_roc/bootstrap/{}/{}",
            self.origin,
            window.as_str(),
            session.token
        ))
    }

    fn detach_window(&self, window: &WindowId) {
        self.context.sessions.remove_window(window);
    }

    fn shutdown(&mut self) {
        self.shutdown.cancel();
    }
}

impl Drop for HttpServer {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

pub fn wrap_router(router: Router, context: HttpContext) -> Router {
    let protected = router.layer(middleware::from_fn_with_state(
        context.clone(),
        require_session,
    ));

    let mut roc_routes = Router::new()
        .route("/_roc/bootstrap/{window}/{token}", get(bootstrap))
        .with_state(context.clone());

    if context.assets.is_some() {
        let assets = Router::new()
            .route("/assets/{*path}", get(serve_asset))
            .route_layer(middleware::from_fn_with_state(
                context.clone(),
                require_session,
            ))
            .with_state(context.clone());
        roc_routes = roc_routes.merge(assets);
    }

    roc_routes
        .merge(protected)
        .layer(middleware::from_fn_with_state(context, security_headers))
}

async fn bootstrap(
    State(context): State<HttpContext>,
    Path((window, token)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if !valid_host(&headers, &context) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(session) = context.sessions.get_by_token(&token) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if session.window_id.as_str() != window {
        return StatusCode::NOT_FOUND.into_response();
    }

    let location = if session.start_url.starts_with('/') {
        session.start_url.clone()
    } else {
        join_origin(
            &format!("http://{}", context.expected_host),
            &session.start_url,
        )
    };
    let mut response = Redirect::to(&location).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&session_cookie_header(&session))
            .expect("session cookie is a valid header value"),
    );
    response
}

async fn require_session(
    State(context): State<HttpContext>,
    mut request: Request,
    next: Next,
) -> Response {
    if !valid_host(request.headers(), &context) || !valid_origin(&request, &context) {
        return (StatusCode::UNAUTHORIZED, "desktop session required").into_response();
    }
    let Some(session) = authenticated_session(request.headers(), &context) else {
        return (StatusCode::UNAUTHORIZED, "desktop session required").into_response();
    };
    request.extensions_mut().insert(session.window_id.clone());
    request.extensions_mut().insert(session);
    next.run(request).await
}

async fn security_headers(
    State(context): State<HttpContext>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Ok(csp) = HeaderValue::from_str(&context.csp) {
        headers.insert(header::CONTENT_SECURITY_POLICY, csp);
    }
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

async fn serve_asset(State(context): State<HttpContext>, Path(path): Path<String>) -> Response {
    let Some(assets) = &context.assets else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match assets.get(&path) {
        Ok(Some(asset)) => {
            let content_type = HeaderValue::from_str(&asset.content_type)
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
            (
                [(header::CONTENT_TYPE, content_type)],
                Body::from(asset.bytes.clone().into_owned()),
            )
                .into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, path, "failed to read asset");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn authenticated_session(headers: &HeaderMap, context: &HttpContext) -> Option<Session> {
    let token = parse_session_cookie(headers)?;
    context.sessions.get_by_token(token)
}

fn valid_host(headers: &HeaderMap, context: &HttpContext) -> bool {
    headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| host == context.expected_host.as_ref())
}

fn valid_origin(request: &Request, context: &HttpContext) -> bool {
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
        .is_some_and(|origin| {
            origin == context.expected_origin.as_ref()
                || context
                    .extra_origins
                    .iter()
                    .any(|allowed| allowed == origin)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetMap;
    use axum::routing::{get, post};
    use http_body_util::BodyExt;
    use std::net::{IpAddr, Ipv4Addr};
    use tower::ServiceExt;

    const ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 43123);
    const TOKEN: &str = "test-token-aaaaaaaaaaaaaaaaaaaaaa";

    fn context(assets: Option<AssetSource>) -> HttpContext {
        let context = HttpContext::new(&Config::default(), ADDRESS, assets);
        context.sessions.insert(Session {
            window_id: WindowId::new("main"),
            token: TOKEN.into(),
            start_url: "/".into(),
        });
        context
    }

    fn app(assets: Option<AssetSource>) -> Router {
        wrap_router(
            Router::new()
                .route("/", get(|| async { "hello" }))
                .route("/action", post(|| async { "ok" })),
            context(assets),
        )
    }

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
    async fn bootstrap_sets_a_window_session_and_hides_the_token() {
        let response = app(None)
            .oneshot(request(
                http::Method::GET,
                &format!("/_roc/bootstrap/main/{TOKEN}"),
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
    async fn bootstrap_rejects_a_token_bound_to_another_window() {
        let response = app(None)
            .oneshot(request(
                http::Method::GET,
                &format!("/_roc/bootstrap/other/{TOKEN}"),
                false,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn protected_routes_reject_requests_without_the_session() {
        let response = app(None)
            .oneshot(request(http::Method::GET, "/", false))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn window_id_is_injected_for_authenticated_requests() {
        async fn show_window(request: Request) -> impl IntoResponse {
            request
                .extensions()
                .get::<WindowId>()
                .map(|id| id.to_string())
                .unwrap_or_default()
        }

        let response = wrap_router(Router::new().route("/who", get(show_window)), context(None))
            .oneshot(request(http::Method::GET, "/who", true))
            .await
            .unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"main");
    }

    #[tokio::test]
    async fn independent_window_sessions_cannot_reuse_each_others_tokens() {
        let context = HttpContext::new(&Config::default(), ADDRESS, None);
        context.sessions.insert(Session {
            window_id: WindowId::new("main"),
            token: "token-main".into(),
            start_url: "/".into(),
        });
        context.sessions.insert(Session {
            window_id: WindowId::new("htmx"),
            token: "token-htmx".into(),
            start_url: "/htmx".into(),
        });
        let router = wrap_router(
            Router::new().route("/", get(|| async { "ok" })),
            context.clone(),
        );

        let stolen = Request::builder()
            .uri("/")
            .header(header::HOST, ADDRESS.to_string())
            .header(header::COOKIE, "roc_session=token-main")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            router.clone().oneshot(stolen).await.unwrap().status(),
            StatusCode::OK
        );

        context.sessions.remove_window(&WindowId::new("main"));
        let revoked = Request::builder()
            .uri("/")
            .header(header::HOST, ADDRESS.to_string())
            .header(header::COOKIE, "roc_session=token-main")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            router.oneshot(revoked).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        assert!(context.sessions.get_by_token("token-htmx").is_some());
    }

    #[tokio::test]
    async fn rejects_a_rebound_host_even_with_a_valid_cookie() {
        let request = Request::builder()
            .uri("/")
            .header(header::HOST, "attacker.example")
            .header(header::COOKIE, format!("roc_session={TOKEN}"))
            .body(Body::empty())
            .unwrap();
        let response = app(None).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_a_cross_origin_mutation_even_with_a_valid_cookie() {
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/action")
            .header(header::HOST, ADDRESS.to_string())
            .header(header::ORIGIN, "http://127.0.0.1:9999")
            .header(header::COOKIE, format!("roc_session={TOKEN}"))
            .body(Body::empty())
            .unwrap();
        let response = app(None).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn serves_embedded_assets_under_the_session() {
        let mut map = AssetMap::new();
        map.insert("app.css", "text/css; charset=utf-8", b"body{}" as &[u8]);
        let response = app(Some(AssetSource::from(map)))
            .oneshot(request(http::Method::GET, "/assets/app.css", true))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/css; charset=utf-8"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"body{}");
    }

    #[tokio::test]
    async fn content_security_policy_is_applied() {
        let response = app(None)
            .oneshot(request(http::Method::GET, "/", true))
            .await
            .unwrap();
        let policy = response.headers()[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap();
        assert!(policy.contains("script-src 'self' 'unsafe-eval'"));
    }

    #[tokio::test]
    async fn graceful_shutdown_stops_accepting_connections() {
        let server = HttpServer::start(
            Config::default(),
            Router::new().route("/", get(|| async { "ok" })),
            None,
        )
        .await
        .unwrap();
        let origin = server.origin().to_owned();
        server.stop().await.unwrap();
        let error = tokio::net::TcpStream::connect(origin.trim_start_matches("http://"))
            .await
            .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::ConnectionRefused);
    }
}
