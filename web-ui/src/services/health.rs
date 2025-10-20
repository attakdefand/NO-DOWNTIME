use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HealthStatus {
    pub status: String,
    pub checks: Vec<HealthCheck>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
}

pub struct HealthService {
    base_url: String,
}

impl HealthService {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
        }
    }
    
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }
    
    pub async fn get_live_status(&self) -> Result<HealthStatus, gloo_net::Error> {
        let url = format!("{}/live", self.base_url);
        let resp = Request::get(&url)
            .send()
            .await?;
            
        let health_status: HealthStatus = resp.json().await?;
        Ok(health_status)
    }
    
    pub async fn get_ready_status(&self) -> Result<HealthStatus, gloo_net::Error> {
        let url = format!("{}/ready", self.base_url);
        let resp = Request::get(&url)
            .send()
            .await?;
            
        let health_status: HealthStatus = resp.json().await?;
        Ok(health_status)
    }
    
    pub async fn set_ready(&self, ready: bool) -> Result<(), gloo_net::Error> {
        // In a real implementation, this would make an API call to set the service ready/not ready
        // For now, we'll just simulate the behavior
        gloo_console::log!(format!("Setting service ready status to: {}", ready).as_str());
        Ok(())
    }
}