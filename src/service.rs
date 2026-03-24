use crate::config::Config;
use crate::error::{AppError, Result};
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, info};

#[derive(Clone)]
pub struct VpnService {
    config: Config,
    api_client: Client,
    sub_client: Client,
}

impl VpnService {
    pub fn new(config: &Config) -> Self {
        let api_client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to create API client");

        let sub_client = Client::builder()
            .build()
            .expect("Failed to create subscription client");

        Self {
            config: config.clone(),
            api_client,
            sub_client,
        }
    }

    pub async fn get_subscription_link(&self) -> Result<String> {
        let auth_data = self.login().await?;
        self.resolve_subscribe_url(&auth_data).await
    }

    pub async fn get_subscription_msg(&self, client_headers: Option<HeaderMap>) -> Result<String> {
        let link = self.get_subscription_link().await?;
        debug!("Fetching subscription message from: {}", link);

        let mut request = self.sub_client.get(&link);

        if let Some(headers) = client_headers {
            debug!("Forwarding {} headers from client request", headers.len());
            for (name, value) in headers.iter() {
                if matches!(
                    name.as_str(),
                    "host" | "connection" | "content-length" | "transfer-encoding"
                ) {
                    continue;
                }
                debug!("Forwarding header: {}", name);
                request = request.header(name, value);
            }
        }

        let response = request.send().await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(AppError::Custom(format!(
                "Failed to fetch subscription message. Status: {}",
                response.status()
            )))
        }
    }

    async fn login(&self) -> Result<String> {
        info!("Logging in to {}...", self.config.login_url);

        let body = serde_json::json!({
            "email": self.config.xcvpn_email,
            "password": self.config.xcvpn_password,
        });

        let response = self
            .api_client
            .post(&self.config.login_url)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(AppError::LoginFailed { status, body: text });
        }

        info!("Login successful.");
        let json: Value = serde_json::from_str(&text)?;

        json["data"]["auth_data"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(AppError::AuthDataNotFound)
    }

    async fn resolve_subscribe_url(&self, auth_data: &str) -> Result<String> {
        let auth_header: reqwest::header::HeaderValue = auth_data.parse()?;

        // 第一步：在 xcvpn.us 创建 ticket
        info!("Creating ticket from {}...", self.config.create_ticket_url);

        let response = self
            .api_client
            .post(&self.config.create_ticket_url)
            .header("Authorization", auth_header.clone())
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(AppError::TicketFailed { status, body: text });
        }

        let json: Value = serde_json::from_str(&text)?;
        let next_url = json["data"]["next_url"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(AppError::TicketUrlNotFound)?;

        info!(
            "Ticket created, fetching secure subscribe from: {}",
            next_url
        );

        // 第二步：在 xcsuburl.kilxs.cn 解析 ticket（无 cookie，避免 session 复用导致 token 缓存）
        let response = self
            .sub_client
            .get(&next_url)
            .header("Authorization", auth_header)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(AppError::SubscribeFailed { status, body: text });
        }

        info!("Secure subscribe URL fetched successfully.");
        let json: Value = serde_json::from_str(&text)?;

        json["data"]["url"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(AppError::SubscribeUrlNotFound)
    }
}
