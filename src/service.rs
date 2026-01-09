use crate::client::HttpClient;
use crate::config::Config;
use crate::error::{AppError, Result};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Clone)]
pub struct VpnService {
    client: HttpClient,
    config: Config,
}

impl VpnService {
    pub fn new(client: HttpClient, config: Config) -> Self {
        Self { client, config }
    }

    pub async fn get_subscription_link(&self) -> Result<String> {
        let auth_data = self.login().await?;
        self.fetch_subscribe_url(&auth_data).await
    }

    pub async fn get_subscription_msg(&self, client_headers: Option<HeaderMap>) -> Result<String> {
        let link = self.get_subscription_link().await?;
        debug!("Fetching subscription message from: {}", link);

        let mut request = self.client.client().get(&link);

        // 转发客户端的请求头（包括 Cookie、User-Agent 等）
        if let Some(headers) = client_headers {
            debug!("Forwarding {} headers from client request", headers.len());
            for (name, value) in headers.iter() {
                // 跳过一些不应该转发的头
                if matches!(name.as_str(), "host" | "connection" | "content-length" | "transfer-encoding") {
                    continue;
                }
                debug!("Forwarding header: {}", name);
                request = request.header(name, value);
            }
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let body = response.text().await?;
            Ok(body)
        } else {
            Err(AppError::Custom(format!(
                "Failed to fetch subscription message. Status: {}",
                response.status()
            )))
        }
    }

    async fn login(&self) -> Result<String> {
        info!("Logging in to {}...", self.config.login_url);

        let mut form_data = HashMap::new();
        form_data.insert("email", self.config.xcvpn_email.clone());
        form_data.insert("password", self.config.xcvpn_password.clone());

        let response = self
            .client
            .client()
            .post(&self.config.login_url)
            .json(&form_data)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(AppError::LoginFailed { status, body });
        }

        info!("Login successful.");
        let json_body: Value = serde_json::from_str(&body)?;

        json_body["data"]["auth_data"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(AppError::AuthDataNotFound)
    }

    async fn fetch_subscribe_url(&self, auth_data: &str) -> Result<String> {
        info!("Fetching subscription link from {}...", self.config.subscribe_url);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", auth_data.parse()?);

        let response = self
            .client
            .client()
            .get(&self.config.subscribe_url)
            .headers(headers)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(AppError::SubscribeFailed { status, body });
        }

        info!("Subscription link fetched successfully.");
        let json_body: Value = serde_json::from_str(&body)?;

        json_body["data"]["subscribe_url"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(AppError::SubscribeUrlNotFound)
    }
}
