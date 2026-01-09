use crate::error::{AppError, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub xcvpn_email: String,
    pub xcvpn_password: String,
    pub server_host: String,
    pub server_port: u16,
    pub login_url: String,
    pub subscribe_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            xcvpn_email: std::env::var("XCVPN_EMAIL")?,
            xcvpn_password: std::env::var("XCVPN_PASSWORD")?,
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| AppError::Custom("Invalid SERVER_PORT".to_string()))?,
            login_url: std::env::var("LOGIN_URL")
                .unwrap_or_else(|_| "https://xcvpn.us/api/v1/passport/auth/login".to_string()),
            subscribe_url: std::env::var("SUBSCRIBE_URL")
                .unwrap_or_else(|_| "https://xcvpn.us/api/v1/user/getSubscribe".to_string()),
        })
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
