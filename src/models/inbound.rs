use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_null_default<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::<T>::deserialize(de)?;
    Ok(opt.unwrap_or_default())
}

/// Accept i64 from JSON number, string, or null.
/// 3x-ui sometimes serializes `tgId` as a string (e.g. `"77313385"`).
fn deserialize_flex_i64<'de, D>(de: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(de)?;
    match v {
        serde_json::Value::Null => Ok(0),
        serde_json::Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .ok_or_else(|| D::Error::custom("number out of range for i64")),
        serde_json::Value::String(s) => {
            if s.is_empty() {
                Ok(0)
            } else {
                s.parse::<i64>().map_err(D::Error::custom)
            }
        }
        other => Err(D::Error::custom(format!(
            "expected i64-compatible value, got {}",
            other
        ))),
    }
}

fn serialize_i64<S>(v: &i64, ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    ser.serialize_i64(*v)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    VMess,
    VLess,
    Trojan,
    Shadowsocks,
    Hysteria,
    Hysteria2,
    WireGuard,
    HTTP,
    Mixed,
    #[serde(other)]
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientTraffic {
    pub id: i64,
    pub inbound_id: i64,
    pub enable: bool,
    pub email: String,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub sub_id: String,
    pub up: i64,
    pub down: i64,
    #[serde(default)]
    pub all_time: i64,
    pub expiry_time: i64,
    pub total: i64,
    #[serde(default)]
    pub reset: i32,
    #[serde(default)]
    pub last_online: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Inbound {
    #[serde(default)]
    pub id: i64,
    pub up: i64,
    pub down: i64,
    pub total: i64,
    #[serde(default)]
    pub all_time: i64,
    pub remark: String,
    pub enable: bool,
    pub expiry_time: i64,
    #[serde(default)]
    pub traffic_reset: String,
    #[serde(default)]
    pub last_traffic_reset_time: i64,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub client_stats: Vec<ClientTraffic>,
    pub listen: String,
    pub port: u16,
    pub protocol: Protocol,
    pub settings: serde_json::Value,
    pub stream_settings: serde_json::Value,
    pub tag: String,
    pub sniffing: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InboundClient {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub email: String,
    pub enable: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub flow: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub password: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub security: String,
    #[serde(default)]
    pub limit_ip: i32,
    #[serde(default, rename = "totalGB")]
    pub total_gb: i64,
    #[serde(default)]
    pub expiry_time: i64,
    #[serde(
        default,
        deserialize_with = "deserialize_flex_i64",
        serialize_with = "serialize_i64"
    )]
    pub tg_id: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sub_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub comment: String,
    pub reset: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_deserializes() {
        let p: Protocol = serde_json::from_str(r#""vmess""#).unwrap();
        assert_eq!(p, Protocol::VMess);
        let p: Protocol = serde_json::from_str(r#""vless""#).unwrap();
        assert_eq!(p, Protocol::VLess);
    }

    #[test]
    fn protocol_unknown_variant() {
        let p: Protocol = serde_json::from_str(r#""socks5""#).unwrap();
        assert_eq!(p, Protocol::Unknown);
    }

    #[test]
    fn inbound_with_null_client_stats() {
        // 3x-ui /panel/api/inbounds/get/{id} returns clientStats: null
        let raw = r#"{
            "id":1,"up":0,"down":0,"total":0,"remark":"x","enable":true,
            "expiryTime":0,"listen":"","port":80,"protocol":"vless",
            "settings":"{}","streamSettings":"{}","tag":"i","sniffing":"{}",
            "clientStats":null
        }"#;
        let inb: Inbound = serde_json::from_str(raw).unwrap();
        assert!(inb.client_stats.is_empty());
    }

    #[test]
    fn inbound_client_with_newer_fields() {
        // Real 3x-ui client payload: totalGB (uppercase GB), tgId as string,
        // plus unknown fields comment/created_at/updated_at.
        let raw = r#"{
            "id":"abc","email":"u@example.com","enable":true,"flow":"",
            "limitIp":1,"totalGB":1073741824,"expiryTime":-604800000,
            "tgId":"77313385","subId":"x","comment":"hi",
            "created_at":1777667608000,"updated_at":1777667608000,
            "reset":0
        }"#;
        let c: InboundClient = serde_json::from_str(raw).unwrap();
        assert_eq!(c.total_gb, 1073741824);
        assert_eq!(c.tg_id, 77313385);
        assert_eq!(c.comment, "hi");
    }

    #[test]
    fn inbound_client_tg_id_as_int_or_null() {
        let raw_int = r#"{"id":"a","email":"e","enable":true,"limitIp":0,"totalGB":0,"expiryTime":0,"tgId":42,"reset":0}"#;
        let c: InboundClient = serde_json::from_str(raw_int).unwrap();
        assert_eq!(c.tg_id, 42);

        let raw_null = r#"{"id":"a","email":"e","enable":true,"limitIp":0,"totalGB":0,"expiryTime":0,"tgId":null,"reset":0}"#;
        let c: InboundClient = serde_json::from_str(raw_null).unwrap();
        assert_eq!(c.tg_id, 0);
    }

    #[test]
    fn inbound_deserializes() {
        let raw = r#"{
            "id":1,"up":0,"down":0,"total":0,"remark":"test",
            "enable":true,"expiryTime":0,"listen":"","port":443,
            "protocol":"vless","settings":{},"streamSettings":{},
            "tag":"inbound-443","sniffing":{},"clientStats":[]
        }"#;
        let inbound: Inbound = serde_json::from_str(raw).unwrap();
        assert_eq!(inbound.id, 1);
        assert_eq!(inbound.protocol, Protocol::VLess);
        assert_eq!(inbound.port, 443);
    }
}
