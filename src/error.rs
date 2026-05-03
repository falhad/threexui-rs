use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not authenticated — call login() first")]
    NotAuthenticated,

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("api error: {0}")]
    Api(String),

    #[error("endpoint not found: {0} (panel version may be too old)")]
    EndpointNotFound(String),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid config: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_not_authenticated() {
        let e = Error::NotAuthenticated;
        assert_eq!(e.to_string(), "not authenticated — call login() first");
    }

    #[test]
    fn error_display_api() {
        let e = Error::Api("bad request".to_string());
        assert_eq!(e.to_string(), "api error: bad request");
    }

    #[test]
    fn error_display_endpoint_not_found() {
        let e = Error::EndpointNotFound("/panel/api/inbounds/X/copyClients".to_string());
        assert!(e.to_string().contains("endpoint not found"));
        assert!(e.to_string().contains("copyClients"));
    }

    #[test]
    fn error_display_config() {
        let e = Error::Config("port cannot be 0".to_string());
        assert_eq!(e.to_string(), "invalid config: port cannot be 0");
    }
}
