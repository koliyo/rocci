mod common;

use std::fs;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{temp_dir, valid_rocci_concept, write_index};
use http_body_util::BodyExt;
use okf::Profile;
use tower::ServiceExt;

#[tokio::test]
async fn view_router_serves_home_and_concept() {
    let root = temp_dir("view-src");
    write_index(&root);
    fs::write(
        root.join("hello.md"),
        valid_rocci_concept("Hello", "", "Intro.\n\n## Details\n\nBody.\n"),
    )
    .unwrap();
    let output = temp_dir("view-out");
    okmate::site::build(&root, &output, Profile::Rocci).unwrap();

    let app = okmate::http::router(&output);
    let home = app
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(home.status(), StatusCode::OK);
    let home_body = String::from_utf8(
        home.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(home_body.contains("id=\"okmate-nav\""), "{home_body}");
    assert!(home_body.contains("id=\"okmate-main\""), "{home_body}");

    let concept = okmate::http::router(&output)
        .oneshot(Request::get("/hello/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(concept.status(), StatusCode::OK);
    let concept_body = String::from_utf8(
        concept
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(
        concept_body.contains("id=\"okmate-main\""),
        "{concept_body}"
    );
    assert!(concept_body.contains("Details"), "{concept_body}");

    let css = okmate::http::router(&output)
        .oneshot(
            Request::get("/__okmate/app.css")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(css.status(), StatusCode::OK);
}

#[test]
fn view_binds_localhost_by_default() {
    let addr = okmate::http::bind_addr(false, 0);
    assert!(addr.ip().is_loopback());
    assert!(!okmate::http::bind_addr(true, 0).ip().is_loopback());
}
