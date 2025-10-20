use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_bind_address")]
    pub bind_address: SocketAddr,
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .set_default("bind_address", "0.0.0.0:3000")?
            .set_default("shutdown_timeout", 30)?
            .add_source(config::Environment::with_prefix("APP"))
            .build()?
            .try_deserialize()
    }
}

fn default_bind_address() -> SocketAddr {
    "0.0.0.0:3000".parse().expect("Invalid bind address")
}

fn default_shutdown_timeout() -> u64 {
    30
}