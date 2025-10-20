use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use no_downtime_service::{auth::AuthState, health, server};
use serde_json::Value;
use tower::ServiceExt; // for `oneshot` and `ready`

#[tokio::test]
async fn test_liveness_probe() {
    // Ensure we start with a known state
    health::set_alive();
    health::set_ready();
    
    // Create a simple auth state for testing (using HMAC for simplicity)
    let auth_state = AuthState::new(
        jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
        jsonwebtoken::DecodingKey::from_secret("secret".as_ref()),
        "test-issuer".to_string(),
    );
    let app = server::create_app().with_state(auth_state);
    
    let response = app
        .oneshot(Request::builder().uri("/live").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "ok");
    
    // Reset state for other tests
    health::set_alive();
    health::set_ready();
}

#[tokio::test]
async fn test_readiness_probe() {
    // Ensure we start with a known state
    health::set_alive();
    health::set_ready();
    
    // Create a simple auth state for testing (using HMAC for simplicity)
    let auth_state = AuthState::new(
        jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
        jsonwebtoken::DecodingKey::from_secret("secret".as_ref()),
        "test-issuer".to_string(),
    );
    let app = server::create_app().with_state(auth_state);
    
    let response = app
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "ok");
    
    // Reset state for other tests
    health::set_alive();
    health::set_ready();
}

#[tokio::test]
async fn test_liveness_probe_failure() {
    // Ensure we start with a known state
    health::set_alive();
    health::set_ready();
    
    // Test that the liveness probe returns SERVICE_UNAVAILABLE when service is not alive
    health::set_not_alive();
    
    // Create a simple auth state for testing (using HMAC for simplicity)
    let auth_state = AuthState::new(
        jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
        jsonwebtoken::DecodingKey::from_secret("secret".as_ref()),
        "test-issuer".to_string(),
    );
    let app = server::create_app().with_state(auth_state);
    
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
    health::set_ready();
}

#[tokio::test]
async fn test_readiness_probe_failure() {
    // Ensure we start with a known state
    health::set_alive();
    health::set_ready();
    
    // Test that the readiness probe returns SERVICE_UNAVAILABLE when service is not ready
    health::set_not_ready();
    
    // Create a simple auth state for testing (using HMAC for simplicity)
    let auth_state = AuthState::new(
        jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
        jsonwebtoken::DecodingKey::from_secret("secret".as_ref()),
        "test-issuer".to_string(),
    );
    let app = server::create_app().with_state(auth_state);
    
    let response = app
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "error");
    
    // Reset to ready state for other tests
    health::set_alive();
    health::set_ready();
}