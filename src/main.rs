use axum::{
    http,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::Value;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vpn_sub_scraper=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/", get(get_sub)).route("/res", get(get_sub_res));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[tracing::instrument]
async fn get_sub() -> Response {
    match get_subscription_link().await {
        Ok(link) => link.into_response(),
        Err(e) => {
            tracing::error!("Failed to get subscription link: {}", e);
            (http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

#[tracing::instrument]
async fn get_sub_res() -> Response {
    match get_subscription_msg().await {
        Ok(msg) => msg.into_response(),
        Err(e) => {
            tracing::error!("Failed to get subscription link: {}", e);
            (http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}
#[tracing::instrument]
async fn get_subscription_msg() -> anyhow::Result<String> {
    let client = Client::builder().build()?;
    let link = get_subscription_link().await?;
    let subscribe_msg_res = client.get(link).send().await?;
    if subscribe_msg_res.status().is_success() {
        let body = subscribe_msg_res.text().await?;
        Ok(body)
    } else {
        anyhow::bail!("'subscribe_url' not found in the response.")
    }
}

#[tracing::instrument]
async fn get_subscription_link() -> anyhow::Result<String> {
    let client = Client::builder().cookie_store(true).build()?;

    let login_url = "https://xcvpn.us/api/v1/passport/auth/login";

    let mut form_data = std::collections::HashMap::new();
    form_data.insert("email", std::env::var("XCVPN_EMAIL")?);
    form_data.insert("password", std::env::var("XCVPN_PASSWORD")?);

    info!("Logging in...");
    let res = client.post(login_url).json(&form_data).send().await?;

    let status = res.status();
    let body = res.text().await?;

    if status.is_success() {
        info!("Login successful.");
        let json_body: Value = serde_json::from_str(&body)?;
        if let Some(auth_data) = json_body["data"]["auth_data"].as_str() {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("Authorization", auth_data.parse()?);

            let subscribe_url_api = "https://xcvpn.us/api/v1/user/getSubscribe";
            info!("Fetching subscription link...");
            let subscribe_res = client
                .get(subscribe_url_api)
                .headers(headers)
                .send()
                .await?;

            let subscribe_status = subscribe_res.status();
            let subscribe_body = subscribe_res.text().await?;

            if subscribe_status.is_success() {
                info!("Subscription link fetched successfully.");
                let subscribe_json: Value = serde_json::from_str(&subscribe_body)?;
                if let Some(subscribe_url) = subscribe_json["data"]["subscribe_url"].as_str() {
                    Ok(subscribe_url.to_string())
                } else {
                    anyhow::bail!("'subscribe_url' not found in the response.")
                }
            } else {
                anyhow::bail!(
                    "Failed to get subscription info. Status: {}. Body: {}",
                    subscribe_status,
                    subscribe_body
                )
            }
        } else {
            anyhow::bail!("'auth_data' not found in the login response.")
        }
    } else {
        anyhow::bail!("Login failed! Status: {}. Body: {}", status, body)
    }
}
