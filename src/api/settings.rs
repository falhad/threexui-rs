use crate::models::settings::AllSetting;
use crate::{Client, Result};

pub struct SettingsApi<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> SettingsApi<'a> {
    pub async fn get_all(&self) -> Result<AllSetting> {
        self.client
            .post("panel/setting/all", &serde_json::json!({}))
            .await
    }

    pub async fn get_defaults(&self) -> Result<serde_json::Value> {
        self.client
            .post("panel/setting/defaultSettings", &serde_json::json!({}))
            .await
    }

    pub async fn update(&self, settings: &AllSetting) -> Result<()> {
        self.client.post_empty("panel/setting/update", settings).await
    }

    pub async fn update_user(
        &self,
        old_username: &str,
        old_password: &str,
        new_username: &str,
        new_password: &str,
    ) -> Result<()> {
        let body = serde_json::json!({
            "oldUsername": old_username,
            "oldPassword": old_password,
            "newUsername": new_username,
            "newPassword": new_password,
        });
        self.client
            .post_empty("panel/setting/updateUser", &body)
            .await
    }

    pub async fn restart_panel(&self) -> Result<()> {
        self.client
            .post_empty("panel/setting/restartPanel", &serde_json::json!({}))
            .await
    }

    pub async fn default_xray_config(&self) -> Result<serde_json::Value> {
        self.client
            .get("panel/setting/getDefaultJsonConfig")
            .await
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
    async fn get_all_returns_settings() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/setting/all"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": {"webPort": 2053, "tgBotEnable": false}
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let settings = client.settings().get_all().await.unwrap();
        assert_eq!(settings.web_port, 2053);
    }

    #[tokio::test]
    async fn update_user_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/setting/updateUser"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "updated", "obj": null
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        client
            .settings()
            .update_user("admin", "old", "admin", "new123")
            .await
            .unwrap();
    }
}
