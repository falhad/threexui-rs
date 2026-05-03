use crate::models::inbound::Inbound;
use crate::{Client, Result};

pub struct InboundsApi<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> InboundsApi<'a> {
    pub async fn list(&self) -> Result<Vec<Inbound>> {
        self.client.get("panel/api/inbounds/list").await
    }

    pub async fn get(&self, id: i64) -> Result<Inbound> {
        self.client
            .get(&format!("panel/api/inbounds/get/{}", id))
            .await
    }

    pub async fn add(&self, inbound: &Inbound) -> Result<Inbound> {
        self.client.post("panel/api/inbounds/add", inbound).await
    }

    pub async fn update(&self, id: i64, inbound: &Inbound) -> Result<Inbound> {
        self.client
            .post(&format!("panel/api/inbounds/update/{}", id), inbound)
            .await
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        self.client
            .post_empty(
                &format!("panel/api/inbounds/del/{}", id),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn import(&self, inbound: &Inbound) -> Result<Inbound> {
        let data_str = serde_json::to_string(inbound)?;
        self.client.require_auth()?;
        let raw = self
            .client
            .inner
            .http
            .post(self.client.url("panel/api/inbounds/import"))
            .form(&[("data", data_str.as_str())])
            .send()
            .await?;
        let resp = crate::client::read_api_response::<Inbound>(raw).await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| crate::Error::Api("empty response".into())))
    }

    pub async fn add_client(&self, inbound_id: i64, clients: &[serde_json::Value]) -> Result<()> {
        let settings = serde_json::json!({ "clients": clients }).to_string();
        let body = serde_json::json!({ "id": inbound_id, "settings": settings });
        self.client
            .post_empty("panel/api/inbounds/addClient", &body)
            .await
    }

    pub async fn update_client(
        &self,
        client_id: &str,
        inbound_id: i64,
        client: &serde_json::Value,
    ) -> Result<()> {
        let settings = serde_json::json!({ "clients": [client] }).to_string();
        let body = serde_json::json!({ "id": inbound_id, "settings": settings });
        self.client
            .post_empty(
                &format!("panel/api/inbounds/updateClient/{}", client_id),
                &body,
            )
            .await
    }

    pub async fn delete_client(&self, inbound_id: i64, client_id: &str) -> Result<()> {
        self.client
            .post_empty(
                &format!("panel/api/inbounds/{}/delClient/{}", inbound_id, client_id),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn delete_client_by_email(&self, inbound_id: i64, email: &str) -> Result<()> {
        self.client
            .post_empty(
                &format!(
                    "panel/api/inbounds/{}/delClientByEmail/{}",
                    inbound_id, email
                ),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn copy_clients(
        &self,
        target_inbound_id: i64,
        source_inbound_id: i64,
        client_emails: &[String],
        flow: &str,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "sourceInboundId": source_inbound_id,
            "clientEmails": client_emails,
            "flow": flow,
        });
        self.client
            .post(
                &format!("panel/api/inbounds/{}/copyClients", target_inbound_id),
                &body,
            )
            .await
    }

    pub async fn client_ips(&self, email: &str) -> Result<serde_json::Value> {
        self.client
            .post(
                &format!("panel/api/inbounds/clientIps/{}", email),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn clear_client_ips(&self, email: &str) -> Result<()> {
        self.client
            .post_empty(
                &format!("panel/api/inbounds/clearClientIps/{}", email),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn client_traffics_by_email(
        &self,
        email: &str,
    ) -> Result<crate::models::inbound::ClientTraffic> {
        self.client
            .get(&format!("panel/api/inbounds/getClientTraffics/{}", email))
            .await
    }

    pub async fn client_traffics_by_id(
        &self,
        id: &str,
    ) -> Result<Vec<crate::models::inbound::ClientTraffic>> {
        self.client
            .get(&format!("panel/api/inbounds/getClientTrafficsById/{}", id))
            .await
    }

    pub async fn reset_client_traffic(&self, inbound_id: i64, email: &str) -> Result<()> {
        self.client
            .post_empty(
                &format!(
                    "panel/api/inbounds/{}/resetClientTraffic/{}",
                    inbound_id, email
                ),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn reset_all_traffics(&self) -> Result<()> {
        self.client
            .post_empty(
                "panel/api/inbounds/resetAllTraffics",
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn reset_all_client_traffics(&self, inbound_id: i64) -> Result<()> {
        self.client
            .post_empty(
                &format!("panel/api/inbounds/resetAllClientTraffics/{}", inbound_id),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn delete_depleted_clients(&self, inbound_id: i64) -> Result<()> {
        self.client
            .post_empty(
                &format!("panel/api/inbounds/delDepletedClients/{}", inbound_id),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn online_clients(&self) -> Result<Vec<String>> {
        self.client
            .post("panel/api/inbounds/onlines", &serde_json::json!({}))
            .await
    }

    pub async fn last_online(&self) -> Result<std::collections::HashMap<String, i64>> {
        self.client
            .post("panel/api/inbounds/lastOnline", &serde_json::json!({}))
            .await
    }

    pub async fn update_client_traffic(
        &self,
        email: &str,
        upload: i64,
        download: i64,
    ) -> Result<()> {
        let body = serde_json::json!({ "upload": upload, "download": download });
        self.client
            .post_empty(
                &format!("panel/api/inbounds/updateClientTraffic/{}", email),
                &body,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClientConfig;
    use crate::models::inbound::Protocol;
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
    async fn list_returns_inbounds() {
        let server = MockServer::start().await;
        let inbound_json = serde_json::json!([{
            "id":1,"up":0,"down":0,"total":0,"remark":"test","enable":true,
            "expiryTime":0,"listen":"","port":443,"protocol":"vless",
            "settings":{},"streamSettings":{},"tag":"inbound-443",
            "sniffing":{},"clientStats":[]
        }]);
        Mock::given(method("GET"))
            .and(path("/panel/api/inbounds/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": inbound_json
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let inbounds = client.inbounds().list().await.unwrap();
        assert_eq!(inbounds.len(), 1);
        assert_eq!(inbounds[0].id, 1);
        assert_eq!(inbounds[0].protocol, Protocol::VLess);
    }

    #[tokio::test]
    async fn get_returns_single_inbound() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/panel/api/inbounds/get/5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": {
                    "id":5,"up":0,"down":0,"total":0,"remark":"my-inbound","enable":true,
                    "expiryTime":0,"listen":"","port":8080,"protocol":"vmess",
                    "settings":{},"streamSettings":{},"tag":"inbound-8080",
                    "sniffing":{},"clientStats":[]
                }
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let inbound = client.inbounds().get(5).await.unwrap();
        assert_eq!(inbound.id, 5);
        assert_eq!(inbound.remark, "my-inbound");
    }

    #[tokio::test]
    async fn delete_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/api/inbounds/del/3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "deleted", "obj": 3
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        client.inbounds().delete(3).await.unwrap();
    }

    #[tokio::test]
    async fn add_client_sends_correct_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/api/inbounds/addClient"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "client added", "obj": null
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let new_client = serde_json::json!({"email": "user@example.com", "enable": true});
        client
            .inbounds()
            .add_client(1, &[new_client])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_client_by_email_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/panel/api/inbounds/2/delClientByEmail/user@example.com",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "deleted", "obj": null
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        client
            .inbounds()
            .delete_client_by_email(2, "user@example.com")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn online_clients_returns_email_list() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/api/inbounds/onlines"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": ["user1@example.com", "user2@example.com"]
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let online = client.inbounds().online_clients().await.unwrap();
        assert_eq!(online.len(), 2);
        assert!(online.contains(&"user1@example.com".to_string()));
    }

    #[tokio::test]
    async fn reset_all_traffics_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/api/inbounds/resetAllTraffics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "reset", "obj": null
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        client.inbounds().reset_all_traffics().await.unwrap();
    }
}
