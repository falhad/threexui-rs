use serde::{Deserialize, Deserializer, Serialize};

fn null_or_vec<'de, D>(de: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<Vec<String>>::deserialize(de)?.unwrap_or_default())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomGeoResource {
    pub id: i64,
    #[serde(rename = "type")]
    pub geo_type: String,
    pub alias: String,
    pub url: String,
    #[serde(default)]
    pub local_path: String,
    #[serde(default)]
    pub last_updated_at: i64,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCustomGeo {
    #[serde(rename = "type")]
    pub geo_type: String,
    pub alias: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CustomGeoAliases {
    #[serde(default, deserialize_with = "null_or_vec")]
    pub geosite: Vec<String>,
    #[serde(default, deserialize_with = "null_or_vec")]
    pub geoip: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_geo_deserializes() {
        let raw = r#"{"id":1,"type":"geoip","alias":"myip","url":"https://example.com/ip.dat","localPath":"","lastUpdatedAt":0,"createdAt":0,"updatedAt":0}"#;
        let r: CustomGeoResource = serde_json::from_str(raw).unwrap();
        assert_eq!(r.id, 1);
        assert_eq!(r.geo_type, "geoip");
        assert_eq!(r.alias, "myip");
    }
}
