use crate::config::ClientConfig;

pub struct Client {
    config: ClientConfig,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }
}
