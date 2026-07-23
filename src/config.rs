use std::{env, net::SocketAddr};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let bind = env::var("MARKETSHIELD_BIND")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
            .parse::<SocketAddr>()
            .map_err(|error| {
                AppError::Configuration(format!("invalid MARKETSHIELD_BIND: {error}"))
            })?;

        let database_url = env::var("MARKETSHIELD_DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://data/marketshield.db".to_string());

        Ok(Self { bind, database_url })
    }
}
