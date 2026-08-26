mod common;

use std::fs;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{temp_dir, valid_rocci_concept, write_index};
use http_body_util::BodyExt;
use okf::Profile;
use tower::ServiceExt;

fn app(root: std::path::PathBuf, output: std::path::PathBuf) -> axum::Router {
    okmate::http::router(okmate::http::AppState {
        output,
        root,
        profile: Profile::Rocci,
        config_path: std::env::temp_dir().join("okmate-nav-unused.toml"),
    })
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

fn fixture() -> (std::path::PathBuf, std::path::PathBuf) {
    let root = temp_dir("nav-src");
    write_index(&root);
    fs::write(
        root.join("hello.md"),
        valid_rocci_concept("Hello", "", "Intro.\n\n## Details\n\nBody.\n"),
    )
    .unwrap();
    let output = temp_dir("nav-out");
    okmate::site::build(&root, &output, Profile::Rocci).unwrap();
    (root, output)
}

#[tokio::test]
async fn datastar_get_concept_returns_main_fragment() {
    let (root, output) = fixture();
    let app = app(root, output);
    let response = app
        .oneshot(
            Request::get("/hello/")
                .header("datastar-request", "true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("id=\"okmate-main\""), "{body}");
    assert!(body.contains("id=\"okmate-toc\""), "{body}");
    assert!(body.contains("Details"), "{body}");
    assert!(
        !body.to_ascii_lowercase().contains("<html"),
        "patch should not be a full document: {body}"
    );
    assert!(
        !body.contains("id=\"okmate-nav\""),
        "nav should stay in the DOM: {body}"
    );
}

#[tokio::test]
async fn review_page_contains_queue_region() {
    let (root, output) = fixture();
    let app = app(root, output);
    let response = app
        .oneshot(Request::get("/review/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("id=\"okmate-queue\""), "{body}");
    assert!(body.contains("Hello"), "{body}");
}
