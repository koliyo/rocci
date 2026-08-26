mod common;

use std::fs;
use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{Request, StatusCode};
use common::{temp_dir, valid_rocci_concept, write_index};
use http_body_util::BodyExt;
use okf::Profile;
use tower::ServiceExt;

fn app(
    root: std::path::PathBuf,
    output: std::path::PathBuf,
    config: std::path::PathBuf,
    peer: [u8; 4],
) -> axum::Router {
    okmate::http::router(okmate::http::AppState {
        output,
        root,
        profile: Profile::Rocci,
        config_path: config,
    })
    .layer(MockConnectInfo(SocketAddr::from((peer, 40000))))
}

async fn body_text(response: axum::http::Response<Body>) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap()
}

fn fixture() -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let root = temp_dir("settings-src");
    write_index(&root);
    fs::write(
        root.join("hello.md"),
        valid_rocci_concept("Hello", "", "Body.\n"),
    )
    .unwrap();
    let output = temp_dir("settings-out");
    okmate::site::build(&root, &output, Profile::Rocci).unwrap();
    let config = temp_dir("settings-cfg").join("config.toml");
    (root, output, config)
}

#[tokio::test]
async fn datastar_post_returns_settings_patch_without_html_shell() {
    let (root, output, config) = fixture();
    let app = app(root, output, config.clone(), [127, 0, 0, 1]);
    let response = app
        .oneshot(
            Request::post("/__okmate/settings")
                .header("datastar-request", "true")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "action=add_directory&id=rocci&path=/tmp/knowledge",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("id=\"okmate-settings\""), "{body}");
    assert!(
        !body.to_ascii_lowercase().contains("<html"),
        "patch should not be a full document: {body}"
    );
    let saved = fs::read_to_string(&config).unwrap();
    assert!(saved.contains("id = \"rocci\""));
}

#[tokio::test]
async fn ordinary_post_returns_full_settings_document() {
    let (root, output, config) = fixture();
    let app = app(root, output, config, [127, 0, 0, 1]);
    let response = app
        .oneshot(
            Request::post("/__okmate/settings")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("action=add_directory&id=docs&path=/tmp/docs"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("<html"), "{body}");
    assert!(body.contains("id=\"okmate-settings\""), "{body}");
    assert!(body.contains("id=\"okmate-nav\""), "{body}");
}

#[tokio::test]
async fn settings_html_does_not_echo_tokens() {
    let (root, output, config) = fixture();
    let app = app(root, output, config.clone(), [127, 0, 0, 1]);
    let response = app
        .oneshot(
            Request::post("/__okmate/settings")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "action=add_git&id=notes&url=https://example.com/notes.git&token=super-secret-token&token_env=GITHUB_TOKEN",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = body_text(response).await;
    assert!(!body.contains("super-secret-token"), "{body}");
    assert!(body.contains("GITHUB_TOKEN"), "{body}");
    assert!(
        body.contains("never shown") || body.contains("is stored"),
        "{body}"
    );
    let saved = fs::read_to_string(config).unwrap();
    assert!(saved.contains("super-secret-token"));
}

#[tokio::test]
async fn settings_post_rejects_non_loopback() {
    let (root, output, config) = fixture();
    let app = app(root, output, config, [10, 0, 0, 1]);
    let response = app
        .oneshot(
            Request::post("/__okmate/settings")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("action=add_directory&id=x&path=/tmp/x"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
