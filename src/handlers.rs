use crate::service::VpnService;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use tracing::{error, instrument};

#[instrument(skip_all)]
pub async fn get_sub(Extension(service): Extension<VpnService>) -> Response {
    match service.get_subscription_link().await {
        Ok(link) => link.into_response(),
        Err(e) => {
            error!("Failed to get subscription link: {}", e);
            e.into_response()
        }
    }
}

#[instrument(skip_all)]
pub async fn get_sub_res(
    Extension(service): Extension<VpnService>,
    headers: HeaderMap,
) -> Response {
    match service.get_subscription_msg(Some(headers)).await {
        Ok(msg) => msg.into_response(),
        Err(e) => {
            error!("Failed to get subscription message: {}", e);
            e.into_response()
        }
    }
}
