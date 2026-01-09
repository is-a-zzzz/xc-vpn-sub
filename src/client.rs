use crate::error::Result;
use reqwest::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct HttpClient {
    client: Arc<Client>,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .build()?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}
