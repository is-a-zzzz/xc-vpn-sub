mod client;
mod config;
mod error;
mod handlers;
mod service;

use axum::{routing::get, Extension, Router};
use client::HttpClient;
use config::Config;
use dotenvy::dotenv;
use service::VpnService;
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

    let config = Config::from_env().expect("Failed to load configuration");
    let http_client = HttpClient::new().expect("Failed to create HTTP client");
    let vpn_service = VpnService::new(http_client, config.clone());

    let app = Router::new()
        .route("/", get(handlers::get_sub))
        .route("/res", get(handlers::get_sub_res))
        .layer(Extension(vpn_service));

    let addr: SocketAddr = config
        .addr()
        .parse()
        .expect("Invalid server address");
    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
