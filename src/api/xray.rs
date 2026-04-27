use crate::models::xray::{NordAction, OutboundTraffic, WarpAction, XraySetting};
use crate::{Client, Result};

pub struct XrayApi<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> XrayApi<'a> {
    pub async fn get_setting(&self) -> Result<XraySetting> {
        self.client
            .post("panel/xray/", &serde_json::json!({}))
            .await
    }

    pub async fn update_setting(&self, xray_config: &str, test_url: &str) -> Result<()> {
        let params = [("xraySetting", xray_config), ("outboundTestUrl", test_url)];
        self.client
            .post_form_empty("panel/xray/update", &params)
            .await
    }

    pub async fn default_config(&self) -> Result<serde_json::Value> {
        self.client.get("panel/xray/getDefaultJsonConfig").await
    }

    pub async fn outbounds_traffic(&self) -> Result<Vec<OutboundTraffic>> {
        self.client.get("panel/xray/getOutboundsTraffic").await
    }

    pub async fn reset_outbound_traffic(&self, tag: &str) -> Result<()> {
        let body = serde_json::json!({ "tag": tag });
        self.client
            .post_empty("panel/xray/resetOutboundsTraffic", &body)
            .await
    }

    pub async fn test_outbound(
        &self,
        outbound: &serde_json::Value,
        all_outbounds: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let outbound_str = serde_json::to_string(outbound)?;
        let all_str = all_outbounds
            .map(serde_json::to_string)
            .transpose()?
            .unwrap_or_default();
        let params = [
            ("outbound", outbound_str.as_str()),
            ("allOutbounds", all_str.as_str()),
        ];
        self.client.require_auth()?;
        let resp = self
            .client
            .inner
            .http
            .post(self.client.url("panel/xray/testOutbound"))
            .form(&params)
            .send()
            .await?
            .json::<crate::models::common::ApiResponse<serde_json::Value>>()
            .await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| crate::Error::Api("empty response".into())))
    }

    pub async fn xray_result(&self) -> Result<serde_json::Value> {
        self.client.get("panel/xray/getXrayResult").await
    }

    pub async fn warp(&self, action: WarpAction) -> Result<serde_json::Value> {
        let action_str = action.action_str().to_string();
        let path = format!("panel/xray/warp/{}", action_str);

        self.client.require_auth()?;
        let req = self.client.inner.http.post(self.client.url(&path));

        let req = match &action {
            WarpAction::Register {
                private_key,
                public_key,
            } => req.form(&[
                ("privateKey", private_key.as_str()),
                ("publicKey", public_key.as_str()),
            ]),
            WarpAction::SetLicense(license) => req.form(&[("license", license.as_str())]),
            _ => req,
        };

        let resp = req
            .send()
            .await?
            .json::<crate::models::common::ApiResponse<serde_json::Value>>()
            .await?;
        resp.into_result()
            .map(|v| v.unwrap_or(serde_json::Value::Null))
    }

    pub async fn nord(&self, action: NordAction) -> Result<serde_json::Value> {
        let action_str = action.action_str().to_string();
        let path = format!("panel/xray/nord/{}", action_str);

        self.client.require_auth()?;
        let req = self.client.inner.http.post(self.client.url(&path));

        let req = match &action {
            NordAction::Servers { country_id } => req.form(&[("countryId", country_id.as_str())]),
            NordAction::Register { token } => req.form(&[("token", token.as_str())]),
            NordAction::SetKey(key) => req.form(&[("key", key.as_str())]),
            _ => req,
        };

        let resp = req
            .send()
            .await?
            .json::<crate::models::common::ApiResponse<serde_json::Value>>()
            .await?;
        resp.into_result()
            .map(|v| v.unwrap_or(serde_json::Value::Null))
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
    async fn outbounds_traffic_returns_list() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/panel/xray/getOutboundsTraffic"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": [
                    {"id":1,"tag":"direct","up":1024,"down":2048,"total":3072}
                ]
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let traffic = client.xray().outbounds_traffic().await.unwrap();
        assert_eq!(traffic.len(), 1);
        assert_eq!(traffic[0].tag, "direct");
    }

    #[tokio::test]
    async fn warp_data_action() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/xray/warp/data"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": {"account": "test"}
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let result = client.xray().warp(WarpAction::Data).await.unwrap();
        assert!(result.is_object());
    }
}
