#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub base_path: String,
    pub tls: bool,
    pub accept_invalid_certs: bool,
    pub timeout_secs: u64,
    /// Optional outbound proxy URL. Supports `http://`, `https://`,
    /// `socks5://` and `socks5h://` schemes. Auth may be embedded
    /// (`http://user:pass@host:port`) or supplied separately via
    /// [`ClientConfigBuilder::proxy_auth`].
    pub proxy: Option<String>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
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
    proxy: Option<String>,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
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

    /// Route every request through an outbound proxy.
    ///
    /// Accepts `http://`, `https://`, `socks5://` or `socks5h://` URLs.
    /// Authentication may be embedded (`http://u:p@host:1080`) or set
    /// separately via [`Self::proxy_auth`].
    pub fn proxy(mut self, url: impl Into<String>) -> Self {
        self.proxy = Some(url.into());
        self
    }

    /// Supply basic-auth credentials for the proxy.
    /// Has no effect unless [`Self::proxy`] is also set.
    pub fn proxy_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.proxy_username = Some(username.into());
        self.proxy_password = Some(password.into());
        self
    }

    /// Disable any previously configured proxy.
    pub fn no_proxy(mut self) -> Self {
        self.proxy = None;
        self.proxy_username = None;
        self.proxy_password = None;
        self
    }

    pub fn build(self) -> crate::Result<ClientConfig> {
        let host = self
            .host
            .ok_or_else(|| crate::Error::Config("host is required".into()))?;
        let port = self
            .port
            .ok_or_else(|| crate::Error::Config("port is required".into()))?;
        if port == 0 {
            return Err(crate::Error::Config("port cannot be 0".into()));
        }
        if let Some(url) = &self.proxy {
            // Surface bad URLs / unsupported schemes early as Error::Config.
            reqwest::Proxy::all(url.as_str())
                .map_err(|e| crate::Error::Config(format!("invalid proxy url: {}", e)))?;
        }
        Ok(ClientConfig {
            host,
            port,
            base_path: self.base_path.unwrap_or_else(|| "/".to_string()),
            tls: self.tls,
            accept_invalid_certs: self.accept_invalid_certs,
            timeout_secs: self.timeout_secs.unwrap_or(30),
            proxy: self.proxy,
            proxy_username: self.proxy_username,
            proxy_password: self.proxy_password,
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
    fn proxy_http_url_accepted() {
        let cfg = ClientConfig::builder()
            .host("localhost")
            .port(2053)
            .proxy("http://127.0.0.1:8080")
            .build()
            .unwrap();
        assert_eq!(cfg.proxy.as_deref(), Some("http://127.0.0.1:8080"));
    }

    #[test]
    fn proxy_socks5_url_accepted() {
        let cfg = ClientConfig::builder()
            .host("localhost")
            .port(2053)
            .proxy("socks5://127.0.0.1:1080")
            .proxy_auth("user", "pass")
            .build()
            .unwrap();
        assert_eq!(cfg.proxy_username.as_deref(), Some("user"));
        assert_eq!(cfg.proxy_password.as_deref(), Some("pass"));
    }

    #[test]
    fn proxy_invalid_url_errors() {
        let err = ClientConfig::builder()
            .host("localhost")
            .port(2053)
            .proxy("not a url at all")
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("invalid proxy url"), "{}", err);
    }

    #[test]
    fn no_proxy_clears_settings() {
        let cfg = ClientConfig::builder()
            .host("localhost")
            .port(2053)
            .proxy("http://1.2.3.4:8080")
            .proxy_auth("u", "p")
            .no_proxy()
            .build()
            .unwrap();
        assert!(cfg.proxy.is_none());
        assert!(cfg.proxy_username.is_none());
        assert!(cfg.proxy_password.is_none());
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
