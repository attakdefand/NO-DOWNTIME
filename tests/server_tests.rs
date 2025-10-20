use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use no_downtime_service::server;
use tower::ServiceExt; // for `oneshot` and `ready`

#[tokio::test]
async fn test_root_endpoint() {
    let app = server::create_app();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"Hello, Zero-Downtime World!");
}

#[tokio::test]
async fn test_nonexistent_endpoint() {
    let app = server::create_app();

    let response = app
        .oneshot(Request::builder().uri("/nonexistent").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}