use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    EnvVar(std::env::VarError),
    Reqwest(reqwest::Error),
    LoginFailed { status: reqwest::StatusCode, body: String },
    SubscribeFailed { status: reqwest::StatusCode, body: String },
    AuthDataNotFound,
    SubscribeUrlNotFound,
    SerdeJson(serde_json::Error),
    Custom(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::EnvVar(e) => write!(f, "Environment variable error: {}", e),
            AppError::Reqwest(e) => write!(f, "HTTP request error: {}", e),
            AppError::LoginFailed { status, body } => {
                write!(f, "Login failed. Status: {}. Body: {}", status, body)
            }
            AppError::SubscribeFailed { status, body } => {
                write!(f, "Subscribe failed. Status: {}. Body: {}", status, body)
            }
            AppError::AuthDataNotFound => write!(f, "'auth_data' not found in the login response."),
            AppError::SubscribeUrlNotFound => {
                write!(f, "'subscribe_url' not found in the response.")
            }
            AppError::SerdeJson(e) => write!(f, "JSON parsing error: {}", e),
            AppError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::EnvVar(_) | AppError::AuthDataNotFound | AppError::SubscribeUrlNotFound => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::Reqwest(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::LoginFailed { .. } | AppError::SubscribeFailed { .. } => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AppError::SerdeJson(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Custom(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, message).into_response()
    }
}

impl From<std::env::VarError> for AppError {
    fn from(e: std::env::VarError) -> Self {
        AppError::EnvVar(e)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Reqwest(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerdeJson(e)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for AppError {
    fn from(e: reqwest::header::InvalidHeaderValue) -> Self {
        AppError::Custom(format!("Invalid header value: {}", e))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
