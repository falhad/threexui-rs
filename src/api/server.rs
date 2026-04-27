use crate::models::server::{
    CpuHistoryPoint, EchCert, Mldsa65Keys, Mlkem768Keys, ServerStatus, UuidResponse,
    VlessEncResult, X25519Cert,
};
use crate::{Client, Result};

pub struct ServerApi<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> ServerApi<'a> {
    pub async fn status(&self) -> Result<ServerStatus> {
        self.client.get("panel/api/server/status").await
    }

    pub async fn cpu_history(&self, bucket: u32) -> Result<Vec<CpuHistoryPoint>> {
        self.client
            .get(&format!("panel/api/server/cpuHistory/{}", bucket))
            .await
    }

    pub async fn xray_versions(&self) -> Result<Vec<String>> {
        self.client.get("panel/api/server/getXrayVersion").await
    }

    pub async fn config_json(&self) -> Result<serde_json::Value> {
        self.client.get("panel/api/server/getConfigJson").await
    }

    pub async fn download_db(&self) -> Result<Vec<u8>> {
        self.client.get_bytes("panel/api/server/getDb").await
    }

    pub async fn new_uuid(&self) -> Result<UuidResponse> {
        self.client.get("panel/api/server/getNewUUID").await
    }

    pub async fn new_x25519_cert(&self) -> Result<X25519Cert> {
        self.client.get("panel/api/server/getNewX25519Cert").await
    }

    pub async fn new_mldsa65(&self) -> Result<Mldsa65Keys> {
        self.client.get("panel/api/server/getNewmldsa65").await
    }

    pub async fn new_mlkem768(&self) -> Result<Mlkem768Keys> {
        self.client.get("panel/api/server/getNewmlkem768").await
    }

    pub async fn new_vless_enc(&self) -> Result<VlessEncResult> {
        self.client.get("panel/api/server/getNewVlessEnc").await
    }

    pub async fn stop_xray(&self) -> Result<()> {
        self.client
            .post_empty(
                "panel/api/server/stopXrayService",
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn restart_xray(&self) -> Result<()> {
        self.client
            .post_empty(
                "panel/api/server/restartXrayService",
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn install_xray(&self, version: &str) -> Result<()> {
        self.client
            .post_empty(
                &format!("panel/api/server/installXray/{}", version),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn update_geofile(&self, filename: Option<&str>) -> Result<()> {
        let path = match filename {
            Some(name) => format!("panel/api/server/updateGeofile/{}", name),
            None => "panel/api/server/updateGeofile".to_string(),
        };
        self.client.post_empty(&path, &serde_json::json!({})).await
    }

    pub async fn logs(&self, count: u32, level: &str, syslog: &str) -> Result<Vec<String>> {
        let params = [("level", level), ("syslog", syslog)];
        let path = format!("panel/api/server/logs/{}", count);
        self.client.require_auth()?;
        let resp = self
            .client
            .inner
            .http
            .post(self.client.url(&path))
            .form(&params)
            .send()
            .await?
            .json::<crate::models::common::ApiResponse<Vec<String>>>()
            .await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| crate::Error::Api("empty response".into())))
    }

    pub async fn xray_logs(
        &self,
        count: u32,
        filter: &str,
        show_direct: bool,
        show_blocked: bool,
        show_proxy: bool,
    ) -> Result<Vec<String>> {
        let params = [
            ("filter", filter.to_string()),
            ("showDirect", show_direct.to_string()),
            ("showBlocked", show_blocked.to_string()),
            ("showProxy", show_proxy.to_string()),
        ];
        let path = format!("panel/api/server/xraylogs/{}", count);
        self.client.require_auth()?;
        let resp = self
            .client
            .inner
            .http
            .post(self.client.url(&path))
            .form(&params)
            .send()
            .await?
            .json::<crate::models::common::ApiResponse<Vec<String>>>()
            .await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| crate::Error::Api("empty response".into())))
    }

    pub async fn import_db(&self, data: Vec<u8>) -> Result<()> {
        self.client.require_auth()?;
        let part = reqwest::multipart::Part::bytes(data).file_name("x-ui.db");
        let form = reqwest::multipart::Form::new().part("db", part);
        let resp = self
            .client
            .inner
            .http
            .post(self.client.url("panel/api/server/importDB"))
            .multipart(form)
            .send()
            .await?
            .json::<crate::models::common::ApiResponse<serde_json::Value>>()
            .await?;
        if resp.success {
            Ok(())
        } else {
            Err(crate::Error::Api(resp.msg))
        }
    }

    pub async fn new_ech_cert(&self, sni: &str) -> Result<EchCert> {
        let params = [("sni", sni)];
        self.client.require_auth()?;
        let resp = self
            .client
            .inner
            .http
            .post(self.client.url("panel/api/server/getNewEchCert"))
            .form(&params)
            .send()
            .await?
            .json::<crate::models::common::ApiResponse<EchCert>>()
            .await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| crate::Error::Api("empty response".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClientConfig;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn auth_client(server: &MockServer) -> Client {
        Mock::given(method("POST"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": null
            })))
            .mount(server)
            .await;
        let config = ClientConfig::builder()
            .host("127.0.0.1")
            .port(server.address().port())
            .build()
            .unwrap();
        let client = Client::new(config);
        client.login("admin", "pass").await.unwrap();
        client
    }

    #[tokio::test]
    async fn status_returns_server_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/panel/api/server/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": {
                    "cpu":5.0,"cpuCores":4,"logicalPro":8,"cpuSpeedMhz":3200.0,
                    "mem":{"current":1024,"total":8192},
                    "swap":{"current":0,"total":0},
                    "disk":{"current":10240,"total":102400},
                    "xray":{"state":"running","errorMsg":"","version":"1.8.0"},
                    "uptime":7200,"loads":[0.1,0.2,0.3],
                    "tcpCount":5,"udpCount":2,
                    "netIO":{"up":512,"down":1024},
                    "netTraffic":{"sent":51200,"recv":102400},
                    "publicIP":{"ipv4":"1.2.3.4","ipv6":"::1"},
                    "appStats":{"threads":8,"mem":32768,"uptime":7200}
                }
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let status = client.server().status().await.unwrap();
        assert_eq!(status.cpu, 5.0);
        assert_eq!(status.xray.state, "running");
    }

    #[tokio::test]
    async fn new_uuid_returns_uuid() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/panel/api/server/getNewUUID"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": {"uuid": "abc-123"}
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let resp = client.server().new_uuid().await.unwrap();
        assert_eq!(resp.uuid, "abc-123");
    }

    #[tokio::test]
    async fn restart_xray_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/api/server/restartXrayService"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "restarted", "obj": null
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        client.server().restart_xray().await.unwrap();
    }
}
