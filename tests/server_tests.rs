use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use no_downtime_service::{auth::AuthState, server};
use tower::ServiceExt; // for `oneshot` and `ready`

#[tokio::test]
async fn test_root_endpoint() {
    // Create a simple auth state for testing (using HMAC for simplicity)
    let auth_state = AuthState::new(
        jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
        jsonwebtoken::DecodingKey::from_secret("secret".as_ref()),
        "test-issuer".to_string(),
    );
    let app = server::create_app().with_state(auth_state);

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
    // Create a simple auth state for testing (using HMAC for simplicity)
    let auth_state = AuthState::new(
        jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
        jsonwebtoken::DecodingKey::from_secret("secret".as_ref()),
        "test-issuer".to_string(),
    );
    let app = server::create_app().with_state(auth_state);

    let response = app
        .oneshot(Request::builder().uri("/nonexistent").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}