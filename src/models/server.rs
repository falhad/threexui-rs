use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStat {
    pub current: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayState {
    pub state: String,
    pub error_msg: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetIO {
    pub up: u64,
    pub down: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetTraffic {
    pub sent: u64,
    pub recv: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIP {
    #[serde(rename = "ipv4")]
    pub ipv4: String,
    #[serde(rename = "ipv6")]
    pub ipv6: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStats {
    pub threads: u32,
    pub mem: u64,
    pub uptime: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub cpu: f64,
    pub cpu_cores: i32,
    pub logical_pro: i32,
    pub cpu_speed_mhz: f64,
    pub mem: ResourceStat,
    pub swap: ResourceStat,
    pub disk: ResourceStat,
    pub xray: XrayState,
    pub uptime: u64,
    #[serde(default)]
    pub loads: Vec<f64>,
    pub tcp_count: i64,
    pub udp_count: i64,
    #[serde(rename = "netIO")]
    pub net_io: NetIO,
    pub net_traffic: NetTraffic,
    #[serde(rename = "publicIP")]
    pub public_ip: PublicIP,
    pub app_stats: AppStats,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuHistoryPoint {
    pub t: i64,
    pub cpu: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UuidResponse {
    pub uuid: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct X25519Cert {
    pub private_key: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mldsa65Keys {
    pub seed: String,
    pub verify: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mlkem768Keys {
    pub seed: String,
    pub client: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EchCert {
    pub ech_server_keys: String,
    pub ech_config_list: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VlessAuth {
    pub label: String,
    #[serde(default)]
    pub encryption: String,
    #[serde(default)]
    pub decryption: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VlessEncResult {
    pub auths: Vec<VlessAuth>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_status_deserializes() {
        let raw = r#"{
            "cpu":12.5,"cpuCores":4,"logicalPro":8,"cpuSpeedMhz":3200.0,
            "mem":{"current":2048,"total":8192},
            "swap":{"current":0,"total":4096},
            "disk":{"current":10240,"total":51200},
            "xray":{"state":"running","errorMsg":"","version":"1.8.0"},
            "uptime":3600,"loads":[0.5,0.4,0.3],
            "tcpCount":10,"udpCount":5,
            "netIO":{"up":1024,"down":2048},
            "netTraffic":{"sent":102400,"recv":204800},
            "publicIP":{"ipv4":"1.2.3.4","ipv6":"::1"},
            "appStats":{"threads":12,"mem":65536,"uptime":3600}
        }"#;
        let status: ServerStatus = serde_json::from_str(raw).unwrap();
        assert_eq!(status.cpu, 12.5);
        assert_eq!(status.cpu_cores, 4);
        assert_eq!(status.mem.total, 8192);
    }

    #[test]
    fn uuid_response_deserializes() {
        let raw = r#"{"uuid":"550e8400-e29b-41d4-a716-446655440000"}"#;
        let u: UuidResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(u.uuid, "550e8400-e29b-41d4-a716-446655440000");
    }
}
