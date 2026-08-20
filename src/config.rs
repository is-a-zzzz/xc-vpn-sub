use crate::error::{AppError, Result};
use serde::Deserialize;

/// 默认使用的官方客户端标识：服务端据此返回完整节点列表的 YAML 配置
const DEFAULT_SUB_USER_AGENT: &str = "Clash/Meta/Mihomo/ClashMetaForAndroid/Bettbox/v2.11.22";

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub xcvpn_email: String,
    pub xcvpn_password: String,
    pub server_host: String,
    pub server_port: u16,
    pub login_url: String,
    pub create_ticket_url: String,
    /// 拉取订阅内容时使用的 User-Agent；
    /// 未设置时默认携带官方客户端标识，显式设为空则转发客户端请求自带的
    pub sub_user_agent: String,
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
            create_ticket_url: std::env::var("CREATE_TICKET_URL")
                .unwrap_or_else(|_| "https://xcvpn.us/api/v1/user/subscribe/createTicket".to_string()),
            sub_user_agent: std::env::var("SUB_USER_AGENT")
                .unwrap_or_else(|_| DEFAULT_SUB_USER_AGENT.to_string()),
        })
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
