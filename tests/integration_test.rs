use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use no_downtime_service::server;
use tower::ServiceExt;

#[tokio::test]
async fn test_service_endpoints() {
    let app = server::create_app();

    // Test root endpoint
    let response = app.clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test liveness endpoint
    let response = app.clone()
        .oneshot(Request::builder().uri("/live").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test readiness endpoint
    let response = app.clone()
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test nonexistent endpoint
    let response = app
        .oneshot(Request::builder().uri("/nonexistent").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}