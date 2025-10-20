use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

// Global health status indicators
static IS_READY: AtomicBool = AtomicBool::new(true);
static IS_LIVE: AtomicBool = AtomicBool::new(true);

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub checks: Vec<HealthCheck>,
}

#[derive(Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
}

pub async fn live_handler() -> Response {
    let live = IS_LIVE.load(Ordering::Relaxed);
    
    if live {
        let status = HealthStatus {
            status: "ok".to_string(),
            checks: vec![HealthCheck {
                name: "liveness".to_string(),
                status: "ok".to_string(),
                message: None,
            }],
        };
        (StatusCode::OK, Json(status)).into_response()
    } else {
        let status = HealthStatus {
            status: "error".to_string(),
            checks: vec![HealthCheck {
                name: "liveness".to_string(),
                status: "error".to_string(),
                message: Some("Service is not alive".to_string()),
            }],
        };
        (StatusCode::SERVICE_UNAVAILABLE, Json(status)).into_response()
    }
}

pub async fn ready_handler() -> Response {
    let ready = IS_READY.load(Ordering::Relaxed);
    
    if ready {
        let status = HealthStatus {
            status: "ok".to_string(),
            checks: vec![HealthCheck {
                name: "readiness".to_string(),
                status: "ok".to_string(),
                message: None,
            }],
        };
        (StatusCode::OK, Json(status)).into_response()
    } else {
        let status = HealthStatus {
            status: "error".to_string(),
            checks: vec![HealthCheck {
                name: "readiness".to_string(),
                status: "error".to_string(),
                message: Some("Service is not ready".to_string()),
            }],
        };
        (StatusCode::SERVICE_UNAVAILABLE, Json(status)).into_response()
    }
}

/// Set the service as ready to accept traffic
pub fn set_ready() {
    IS_READY.store(true, Ordering::Relaxed);
}

/// Set the service as not ready to accept traffic
pub fn set_not_ready() {
    IS_READY.store(false, Ordering::Relaxed);
}

/// Set the service as alive
pub fn set_alive() {
    IS_LIVE.store(true, Ordering::Relaxed);
}

/// Set the service as not alive
pub fn set_not_alive() {
    IS_LIVE.store(false, Ordering::Relaxed);
}