use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::api::custom_geo::CustomGeoApi;
use crate::api::inbounds::InboundsApi;
use crate::api::server::ServerApi;
use crate::api::settings::SettingsApi;
use crate::api::xray::XrayApi;
use crate::config::ClientConfig;
use crate::error::Result;
use crate::models::common::ApiResponse;
use crate::Error;

/// Centralized response decoder.
///
/// Treats HTTP 404 as `Error::EndpointNotFound` (older 3x-ui versions return 404
/// for endpoints added in newer releases — the previous behaviour was an opaque
/// JSON-decode error).
///
/// For any non-success status with a non-JSON body we surface the status code in
/// the resulting `Error::Api`.
pub(crate) async fn read_api_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<ApiResponse<T>> {
    let status = resp.status();
    let path = resp.url().path().to_string();
    let bytes = resp.bytes().await?;
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::EndpointNotFound(path));
    }
    if bytes.is_empty() {
        return Err(Error::Api(format!("empty response body (HTTP {})", status)));
    }
    serde_json::from_slice::<ApiResponse<T>>(&bytes).map_err(|e| {
        if status.is_success() {
            Error::Json(e)
        } else {
            // Trim & expose the body so calls hitting an HTML error page get
            // something readable.
            let snippet: String = String::from_utf8_lossy(&bytes).chars().take(200).collect();
            Error::Api(format!(
                "HTTP {} — non-JSON body: {}",
                status,
                snippet.trim()
            ))
        }
    })
}

pub(crate) struct ClientInner {
    pub http: reqwest::Client,
    pub base_url: String,
    pub authenticated: AtomicBool,
}

#[derive(Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientInner>,
}

impl Client {
    pub fn new(config: ClientConfig) -> Self {
        let mut builder = reqwest::Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .timeout(Duration::from_secs(config.timeout_secs));

        if let Some(proxy_url) = &config.proxy {
            // URL was already validated in `ClientConfigBuilder::build`, so
            // this should not fail unless the user constructed `ClientConfig`
            // by hand with garbage.
            let mut proxy = reqwest::Proxy::all(proxy_url.as_str())
                .expect("proxy url validated at config build time");
            if let (Some(u), Some(p)) = (&config.proxy_username, &config.proxy_password) {
                proxy = proxy.basic_auth(u, p);
            }
            builder = builder.proxy(proxy);
        }

        let http = builder.build().expect("failed to build reqwest client");

        Client {
            inner: Arc::new(ClientInner {
                http,
                base_url: config.base_url(),
                authenticated: AtomicBool::new(false),
            }),
        }
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.inner.base_url, path)
    }

    pub(crate) fn require_auth(&self) -> Result<()> {
        if self.inner.authenticated.load(Ordering::Relaxed) {
            Ok(())
        } else {
            Err(Error::NotAuthenticated)
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        self.login_inner(username, password, None).await
    }

    pub async fn login_2fa(&self, username: &str, password: &str, code: &str) -> Result<()> {
        self.login_inner(username, password, Some(code)).await
    }

    async fn login_inner(
        &self,
        username: &str,
        password: &str,
        two_factor: Option<&str>,
    ) -> Result<()> {
        let mut params = vec![
            ("username", username.to_string()),
            ("password", password.to_string()),
        ];
        if let Some(code) = two_factor {
            params.push(("twoFactorCode", code.to_string()));
        }

        let resp = self
            .inner
            .http
            .post(self.url("login"))
            .form(&params)
            .send()
            .await?
            .json::<ApiResponse<serde_json::Value>>()
            .await?;

        if resp.success {
            self.inner.authenticated.store(true, Ordering::Relaxed);
            Ok(())
        } else {
            Err(Error::Auth(resp.msg))
        }
    }

    pub async fn logout(&self) -> Result<()> {
        let _ = self.inner.http.get(self.url("logout")).send().await?;
        self.inner.authenticated.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub async fn is_two_factor_enabled(&self) -> Result<bool> {
        let resp = self
            .inner
            .http
            .post(self.url("getTwoFactorEnable"))
            .send()
            .await?
            .json::<ApiResponse<bool>>()
            .await?;
        resp.into_result().map(|v| v.unwrap_or(false))
    }

    pub async fn backup_to_tgbot(&self) -> Result<()> {
        self.require_auth()?;
        self.inner
            .http
            .get(self.url("panel/api/backuptotgbot"))
            .send()
            .await?;
        Ok(())
    }

    pub fn inbounds(&self) -> InboundsApi<'_> {
        InboundsApi { client: self }
    }

    pub fn server(&self) -> ServerApi<'_> {
        ServerApi { client: self }
    }

    pub fn settings(&self) -> SettingsApi<'_> {
        SettingsApi { client: self }
    }

    pub fn xray(&self) -> XrayApi<'_> {
        XrayApi { client: self }
    }

    pub fn custom_geo(&self) -> CustomGeoApi<'_> {
        CustomGeoApi { client: self }
    }

    pub(crate) async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.require_auth()?;
        let raw = self.inner.http.get(self.url(path)).send().await?;
        let resp = read_api_response::<T>(raw).await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| Error::Api("empty response".into())))
    }

    pub(crate) async fn post<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        self.require_auth()?;
        let raw = self
            .inner
            .http
            .post(self.url(path))
            .json(body)
            .send()
            .await?;
        let resp = read_api_response::<T>(raw).await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| Error::Api("empty response".into())))
    }

    pub(crate) async fn post_empty<B>(&self, path: &str, body: &B) -> Result<()>
    where
        B: serde::Serialize,
    {
        self.require_auth()?;
        let raw = self
            .inner
            .http
            .post(self.url(path))
            .json(body)
            .send()
            .await?;
        let resp = read_api_response::<serde_json::Value>(raw).await?;
        if resp.success {
            Ok(())
        } else {
            Err(Error::Api(resp.msg))
        }
    }

    pub(crate) async fn post_form_empty(&self, path: &str, params: &[(&str, &str)]) -> Result<()> {
        self.require_auth()?;
        let raw = self
            .inner
            .http
            .post(self.url(path))
            .form(params)
            .send()
            .await?;
        let resp = read_api_response::<serde_json::Value>(raw).await?;
        if resp.success {
            Ok(())
        } else {
            Err(Error::Api(resp.msg))
        }
    }

    pub(crate) async fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        self.require_auth()?;
        let bytes = self
            .inner
            .http
            .get(self.url(path))
            .send()
            .await?
            .bytes()
            .await?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn mock_client(server: &MockServer) -> Client {
        let config = ClientConfig::builder()
            .host("127.0.0.1")
            .port(server.address().port())
            .build()
            .unwrap();
        Client::new(config)
    }

    #[tokio::test]
    async fn login_sets_authenticated() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "ok", "obj": null
            })))
            .mount(&server)
            .await;

        let client = mock_client(&server).await;
        assert!(!client.inner.authenticated.load(Ordering::Relaxed));
        client.login("admin", "pass").await.unwrap();
        assert!(client.inner.authenticated.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn login_failure_returns_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": false, "msg": "wrong username or password", "obj": null
            })))
            .mount(&server)
            .await;

        let client = mock_client(&server).await;
        let err = client.login("admin", "wrong").await.unwrap_err();
        assert!(matches!(err, Error::Auth(_)));
    }

    #[tokio::test]
    async fn http_404_returns_endpoint_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": null
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/panel/api/missing"))
            .respond_with(ResponseTemplate::new(404).set_body_string(""))
            .mount(&server)
            .await;

        let client = mock_client(&server).await;
        client.login("admin", "p").await.unwrap();

        let err: Result<serde_json::Value> = client.get("panel/api/missing").await;
        match err {
            Err(Error::EndpointNotFound(p)) => assert!(p.contains("missing")),
            other => panic!("expected EndpointNotFound, got {:?}", other.is_ok()),
        }
    }

    #[tokio::test]
    async fn http_500_with_html_surfaces_status_in_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": null
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/panel/api/boom"))
            .respond_with(
                ResponseTemplate::new(500).set_body_string("<html>Internal Server Error</html>"),
            )
            .mount(&server)
            .await;

        let client = mock_client(&server).await;
        client.login("admin", "p").await.unwrap();

        let err: Result<serde_json::Value> = client.get("panel/api/boom").await;
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("HTTP 500"), "msg = {}", msg);
    }

    #[tokio::test]
    async fn empty_body_returns_api_error_not_panic() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": null
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/panel/api/empty"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let client = mock_client(&server).await;
        client.login("admin", "p").await.unwrap();

        let err: Result<serde_json::Value> = client.get("panel/api/empty").await;
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("empty response body"), "msg = {}", msg);
    }

    #[tokio::test]
    async fn require_auth_fails_when_not_logged_in() {
        let server = MockServer::start().await;
        let client = mock_client(&server).await;
        assert!(matches!(
            client.require_auth(),
            Err(Error::NotAuthenticated)
        ));
    }
}
