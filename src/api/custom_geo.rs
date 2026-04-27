use crate::models::custom_geo::{CreateCustomGeo, CustomGeoResource};
use crate::{Client, Result};

pub struct CustomGeoApi<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> CustomGeoApi<'a> {
    pub async fn list(&self) -> Result<Vec<CustomGeoResource>> {
        self.client.get("panel/api/custom-geo/list").await
    }

    pub async fn aliases(&self) -> Result<Vec<String>> {
        self.client.get("panel/api/custom-geo/aliases").await
    }

    pub async fn add(&self, geo: &CreateCustomGeo) -> Result<()> {
        self.client.post_empty("panel/api/custom-geo/add", geo).await
    }

    pub async fn update(&self, id: i64, geo: &CreateCustomGeo) -> Result<()> {
        self.client
            .post_empty(&format!("panel/api/custom-geo/update/{}", id), geo)
            .await
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        self.client
            .post_empty(
                &format!("panel/api/custom-geo/delete/{}", id),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn download(&self, id: i64) -> Result<()> {
        self.client
            .post_empty(
                &format!("panel/api/custom-geo/download/{}", id),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn update_all(&self) -> Result<serde_json::Value> {
        self.client
            .post("panel/api/custom-geo/update-all", &serde_json::json!({}))
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
    async fn list_returns_custom_geos() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/panel/api/custom-geo/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "", "obj": [
                    {"id":1,"type":"geoip","alias":"myip","url":"https://example.com/ip.dat",
                     "localPath":"","lastUpdatedAt":0,"createdAt":0,"updatedAt":0}
                ]
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let list = client.custom_geo().list().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].alias, "myip");
    }

    #[tokio::test]
    async fn add_custom_geo_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/api/custom-geo/add"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "added", "obj": null
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        let geo = CreateCustomGeo {
            geo_type: "geoip".into(),
            alias: "myip".into(),
            url: "https://example.com/ip.dat".into(),
        };
        client.custom_geo().add(&geo).await.unwrap();
    }
}
