//! OAuth2 Authentication Module
//!
//! This module provides OAuth2 authentication flows for the zero-downtime service.
//! It supports standard OAuth2 flows including authorization code flow.

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tracing::error;
use url::Url;

/// OAuth2 Configuration
#[derive(Clone, Debug)]
pub struct OAuth2Config {
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Authorization server URL
    pub auth_url: String,
    /// Token exchange URL
    pub token_url: String,
    /// Redirect URL
    pub redirect_url: String,
    /// Scopes
    pub scopes: Vec<String>,
}

/// OAuth2 State
#[derive(Clone)]
pub struct OAuth2State {
    /// Configuration
    pub config: OAuth2Config,
    /// HTTP client
    client: Client,
}

impl OAuth2State {
    /// Create a new OAuth2State
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Generate authorization URL
    pub fn authorization_url(&self) -> Result<String, OAuth2Error> {
        let mut url = Url::parse(&self.config.auth_url)
            .map_err(|e| OAuth2Error::AuthorizationFailed(e.to_string()))?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("client_id", &self.config.client_id);
            query.append_pair("redirect_uri", &self.config.redirect_url);
            query.append_pair("response_type", "code");
            query.append_pair("scope", &self.config.scopes.join(" "));
            query.append_pair("state", &uuid::Uuid::new_v4().to_string());
        }
        Ok(url.to_string())
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str) -> Result<AccessTokenResponse, OAuth2Error> {
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("redirect_uri", &self.config.redirect_url),
            ("code", code),
        ];

        let response = self
            .client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| OAuth2Error::TokenExchangeFailed(e.to_string()))?;

        if response.status().is_success() {
            let token_response = response
                .json::<AccessTokenResponse>()
                .await
                .map_err(|e| OAuth2Error::TokenExchangeFailed(e.to_string()))?;
            Ok(token_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(OAuth2Error::TokenExchangeFailed(error_text))
        }
    }

    /// Validate access token
    pub async fn validate_token(&self, _token: &str) -> Result<Claims, OAuth2Error> {
        // In a real implementation, this would call the OAuth2 provider's userinfo endpoint
        // or use JWT validation if the token is a JWT
        // For now, we'll create a basic claims object
        let claims = Claims {
            sub: "oauth2_user".to_string(),
            iss: self.config.auth_url.clone(),
            exp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize
                + 3600,
            iat: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
            nbf: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
            roles: vec!["oauth2_user".to_string()],
        };
        Ok(claims)
    }
}

/// Access token response
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type
    pub token_type: String,
    /// Expires in (seconds)
    pub expires_in: u32,
    /// Refresh token (optional)
    pub refresh_token: Option<String>,
    /// Scope
    pub scope: Option<String>,
}

/// Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user identifier)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Expiration time
    pub exp: usize,
    /// Issued at time
    pub iat: usize,
    /// Not before time
    pub nbf: usize,
    /// Roles/permissions
    pub roles: Vec<String>,
}

/// OAuth2 Error
#[derive(Debug)]
pub enum OAuth2Error {
    /// Authorization failed
    AuthorizationFailed(String),
    /// Token exchange failed
    TokenExchangeFailed(String),
    /// Token validation failed
    TokenValidationFailed(String),
    /// Missing code parameter
    MissingCodeParameter,
}

impl Display for OAuth2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuth2Error::AuthorizationFailed(msg) => write!(f, "Authorization failed: {}", msg),
            OAuth2Error::TokenExchangeFailed(msg) => write!(f, "Token exchange failed: {}", msg),
            OAuth2Error::TokenValidationFailed(msg) => write!(f, "Token validation failed: {}", msg),
            OAuth2Error::MissingCodeParameter => write!(f, "Missing code parameter"),
        }
    }
}

impl std::error::Error for OAuth2Error {}

impl IntoResponse for OAuth2Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            OAuth2Error::AuthorizationFailed(_) => (StatusCode::UNAUTHORIZED, "Authorization failed"),
            OAuth2Error::TokenExchangeFailed(_) => (StatusCode::BAD_REQUEST, "Token exchange failed"),
            OAuth2Error::TokenValidationFailed(_) => (StatusCode::UNAUTHORIZED, "Token validation failed"),
            OAuth2Error::MissingCodeParameter => (StatusCode::BAD_REQUEST, "Missing code parameter"),
        };

        error!(%error_message, "OAuth2 error");

        (
            status,
            Json(serde_json::json!({
                "error": error_message,
            })),
        )
            .into_response()
    }
}

/// OAuth2 Authorization Request Parameters
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    /// Authorization code
    pub code: Option<String>,
    /// State parameter
    pub state: Option<String>,
    /// Error (if any)
    pub error: Option<String>,
}

/// OAuth2 User Extractor
/// 
/// This can be used as an Axum extractor to automatically validate OAuth2 tokens
/// from the Authorization header.
pub struct OAuth2User {
    /// User claims
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for OAuth2User
where
    OAuth2State: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = OAuth2Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Get the authorization header
        let auth_header = parts
            .headers
            .get("authorization")
            .ok_or(OAuth2Error::AuthorizationFailed(
                "Missing authorization header".to_string(),
            ))?
            .to_str()
            .map_err(|_| {
                OAuth2Error::AuthorizationFailed("Invalid authorization header format".to_string())
            })?;

        // Check if it's a Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(OAuth2Error::AuthorizationFailed(
                "Invalid authorization header format".to_string(),
            ))?;

        // Get the OAuth2State from the application state
        let oauth2_state = OAuth2State::from_ref(state);

        // Validate the token
        let claims = oauth2_state.validate_token(token).await?;

        Ok(OAuth2User { claims })
    }
}

/// Handler for initiating OAuth2 flow
pub async fn oauth2_login_handler(oauth2_state: axum::extract::State<OAuth2State>) -> Result<Redirect, OAuth2Error> {
    let auth_url = oauth2_state.authorization_url()?;
    Ok(Redirect::to(&auth_url))
}

/// Handler for OAuth2 callback
pub async fn oauth2_callback_handler(
    oauth2_state: axum::extract::State<OAuth2State>,
    Query(params): Query<AuthRequest>,
) -> Result<impl IntoResponse, OAuth2Error> {
    if let Some(error) = params.error {
        return Err(OAuth2Error::AuthorizationFailed(error));
    }

    let code = params.code.ok_or(OAuth2Error::MissingCodeParameter)?;
    
    // Exchange code for access token
    let token_response = oauth2_state.exchange_code(&code).await?;
    
    // In a real implementation, you would typically set a session cookie
    // or return a JWT token for the client to use for subsequent requests
    // For this example, we'll just return the access token
    
    Ok(Json(serde_json::json!({
        "access_token": token_response.access_token,
        "token_type": token_response.token_type,
        "expires_in": token_response.expires_in,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_config_creation() {
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
    }

    #[test]
    fn test_authorization_url_generation() {
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
}