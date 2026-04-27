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
        let http = reqwest::Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("failed to build reqwest client");

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
        let resp = self
            .inner
            .http
            .get(self.url(path))
            .send()
            .await?
            .json::<ApiResponse<T>>()
            .await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| Error::Api("empty response".into())))
    }

    pub(crate) async fn post<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        self.require_auth()?;
        let resp = self
            .inner
            .http
            .post(self.url(path))
            .json(body)
            .send()
            .await?
            .json::<ApiResponse<T>>()
            .await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| Error::Api("empty response".into())))
    }

    pub(crate) async fn post_empty<B>(&self, path: &str, body: &B) -> Result<()>
    where
        B: serde::Serialize,
    {
        self.require_auth()?;
        let resp = self
            .inner
            .http
            .post(self.url(path))
            .json(body)
            .send()
            .await?
            .json::<ApiResponse<serde_json::Value>>()
            .await?;
        if resp.success {
            Ok(())
        } else {
            Err(Error::Api(resp.msg))
        }
    }

    pub(crate) async fn post_form_empty(&self, path: &str, params: &[(&str, &str)]) -> Result<()> {
        self.require_auth()?;
        let resp = self
            .inner
            .http
            .post(self.url(path))
            .form(params)
            .send()
            .await?
            .json::<ApiResponse<serde_json::Value>>()
            .await?;
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
    async fn require_auth_fails_when_not_logged_in() {
        let server = MockServer::start().await;
        let client = mock_client(&server).await;
        assert!(matches!(
            client.require_auth(),
            Err(Error::NotAuthenticated)
        ));
    }
}
