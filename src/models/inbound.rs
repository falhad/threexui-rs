use serde::{Deserialize, Serialize};

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
    #[serde(default)]
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
    pub limit_ip: i32,
    pub total_gb: i64,
    pub expiry_time: i64,
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
