#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub base_path: String,
    pub tls: bool,
    pub accept_invalid_certs: bool,
    pub timeout_secs: u64,
}

impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }

    pub fn base_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{}://{}:{}{}", scheme, self.host, self.port, self.base_path)
    }
}

#[derive(Debug, Default)]
pub struct ClientConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    base_path: Option<String>,
    tls: bool,
    accept_invalid_certs: bool,
    timeout_secs: Option<u64>,
}

impl ClientConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(normalize_base_path(path.into()));
        self
    }

    pub fn tls(mut self, tls: bool) -> Self {
        self.tls = tls;
        self
    }

    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn build(self) -> crate::Result<ClientConfig> {
        let host = self.host.ok_or_else(|| crate::Error::Config("host is required".into()))?;
        let port = self.port.ok_or_else(|| crate::Error::Config("port is required".into()))?;
        if port == 0 {
            return Err(crate::Error::Config("port cannot be 0".into()));
        }
        Ok(ClientConfig {
            host,
            port,
            base_path: self.base_path.unwrap_or_else(|| "/".to_string()),
            tls: self.tls,
            accept_invalid_certs: self.accept_invalid_certs,
            timeout_secs: self.timeout_secs.unwrap_or(30),
        })
    }
}

fn normalize_base_path(mut path: String) -> String {
    if !path.starts_with('/') {
        path.insert(0, '/');
    }
    if !path.ends_with('/') {
        path.push('/');
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_minimal_config() {
        let cfg = ClientConfig::builder()
            .host("192.168.1.1")
            .port(2053)
            .build()
            .unwrap();
        assert_eq!(cfg.host, "192.168.1.1");
        assert_eq!(cfg.port, 2053);
        assert_eq!(cfg.base_path, "/");
        assert!(!cfg.tls);
        assert_eq!(cfg.timeout_secs, 30);
    }

    #[test]
    fn build_full_config() {
        let cfg = ClientConfig::builder()
            .host("example.com")
            .port(443)
            .base_path("secret")
            .tls(true)
            .accept_invalid_certs(true)
            .timeout_secs(60)
            .build()
            .unwrap();
        assert_eq!(cfg.base_path, "/secret/");
        assert!(cfg.tls);
        assert!(cfg.accept_invalid_certs);
        assert_eq!(cfg.timeout_secs, 60);
    }

    #[test]
    fn base_url_http() {
        let cfg = ClientConfig::builder()
            .host("192.168.1.1")
            .port(2053)
            .build()
            .unwrap();
        assert_eq!(cfg.base_url(), "http://192.168.1.1:2053/");
    }

    #[test]
    fn base_url_https_with_path() {
        let cfg = ClientConfig::builder()
            .host("example.com")
            .port(443)
            .base_path("/secret/")
            .tls(true)
            .build()
            .unwrap();
        assert_eq!(cfg.base_url(), "https://example.com:443/secret/");
    }

    #[test]
    fn missing_host_errors() {
        let err = ClientConfig::builder().port(2053).build().unwrap_err();
        assert!(err.to_string().contains("host is required"));
    }

    #[test]
    fn port_zero_errors() {
        let err = ClientConfig::builder()
            .host("localhost")
            .port(0)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("port cannot be 0"));
    }
}
