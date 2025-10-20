//! Integration tests for the OAuth2 authentication module

use no_downtime_service::oauth2::{OAuth2Config, OAuth2State};

#[tokio::test]
async fn test_oauth2_initialization() {
    let config = OAuth2Config {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        auth_url: "https://example.com/oauth/authorize".to_string(),
        token_url: "https://example.com/oauth/token".to_string(),
        redirect_url: "https://example.com/callback".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };
    
    let oauth2_state = OAuth2State::new(config);
    assert_eq!(oauth2_state.config.client_id, "test_client");
    assert_eq!(oauth2_state.config.client_secret, "test_secret");
}

#[tokio::test]
async fn test_authorization_url_generation() {
    let config = OAuth2Config {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        auth_url: "https://example.com/oauth/authorize".to_string(),
        token_url: "https://example.com/oauth/token".to_string(),
        redirect_url: "https://example.com/callback".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };
    
    let oauth2_state = OAuth2State::new(config);
    let auth_url = oauth2_state.authorization_url().unwrap();
    
    assert!(auth_url.starts_with("https://example.com/oauth/authorize"));
    assert!(auth_url.contains("client_id=test_client"));
    assert!(auth_url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback"));
    assert!(auth_url.contains("response_type=code"));
    assert!(auth_url.contains("scope=read+write"));
}

#[tokio::test]
async fn test_oauth2_config_clone() {
    let config = OAuth2Config {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        auth_url: "https://example.com/oauth/authorize".to_string(),
        token_url: "https://example.com/oauth/token".to_string(),
        redirect_url: "https://example.com/callback".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };
    
    // Test that config can be cloned
    let config_clone = config.clone();
    assert_eq!(config.client_id, config_clone.client_id);
    assert_eq!(config.client_secret, config_clone.client_secret);
    assert_eq!(config.scopes, config_clone.scopes);
}

#[tokio::test]
async fn test_oauth2_state_clone() {
    let config = OAuth2Config {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        auth_url: "https://example.com/oauth/authorize".to_string(),
        token_url: "https://example.com/oauth/token".to_string(),
        redirect_url: "https://example.com/callback".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };
    
    let oauth2_state = OAuth2State::new(config);
    
    // Test that OAuth2State can be cloned
    let oauth2_state_clone = oauth2_state.clone();
    assert_eq!(oauth2_state.config.client_id, oauth2_state_clone.config.client_id);
}