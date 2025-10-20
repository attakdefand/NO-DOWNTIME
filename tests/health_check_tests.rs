use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use no_downtime_service::{health, server};
use serde_json::Value;
use tower::ServiceExt; // for `oneshot` and `ready`

#[tokio::test]
async fn test_liveness_probe() {
    // Ensure we start with a known state
    health::set_alive();
    health::set_ready();
    
    let app = server::create_app();
    let response = app
        .oneshot(Request::builder().uri("/live").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_readiness_probe() {
    // Ensure we start with a known state
    health::set_alive();
    health::set_ready();
    
    let app = server::create_app();
    let response = app
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_liveness_probe_failure() {
    // Ensure we start with a known state
    health::set_alive();
    health::set_ready();
    
    // Test that the liveness probe returns SERVICE_UNAVAILABLE when service is not alive
    health::set_not_alive();
    
    let app = server::create_app();
    let response = app
        .oneshot(Request::builder().uri("/live").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "error");
    
    // Reset to alive state for other tests
    health::set_alive();
}

#[tokio::test]
async fn test_readiness_probe_failure() {
    // Ensure we start with a known state
    health::set_alive();
    health::set_ready();
    
    // Test that the readiness probe returns SERVICE_UNAVAILABLE when service is not ready
    health::set_not_ready();
    
    let app = server::create_app();
    let response = app
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "error");
    
    // Reset to ready state for other tests
    health::set_ready();
}