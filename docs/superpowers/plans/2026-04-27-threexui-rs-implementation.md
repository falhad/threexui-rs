# threexui-rs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a complete async Rust SDK for the 3x-ui v2.9.3 panel API covering all ~60 endpoints, publishable to crates.io.

**Architecture:** Single `Client` struct backed by `Arc<ClientInner>` holding a `reqwest::Client` with cookie store for session auth. Five namespace accessor structs (`InboundsApi`, `ServerApi`, `SettingsApi`, `XrayApi`, `CustomGeoApi`) each borrow `&Client`. All 3x-ui `{"success", "msg", "obj"}` responses are unwrapped internally.

**Tech Stack:** Rust stable, `tokio` (async runtime), `reqwest` 0.12 (HTTP + cookies + multipart), `serde`/`serde_json` (models), `thiserror` 2 (errors), `wiremock` 0.6 (test mocking).

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Package metadata, dependencies |
| `src/lib.rs` | Public re-exports |
| `src/error.rs` | `Error` enum, `Result<T>` alias |
| `src/config.rs` | `ClientConfig`, `ClientConfigBuilder` |
| `src/client.rs` | `Client`, `ClientInner`, auth methods, namespace accessors |
| `src/models/mod.rs` | Re-exports all model types |
| `src/models/common.rs` | Internal `ApiResponse<T>` |
| `src/models/inbound.rs` | `Inbound`, `Protocol`, `InboundClient`, `ClientTraffic` |
| `src/models/server.rs` | `ServerStatus` and nested types, crypto key structs |
| `src/models/settings.rs` | `AllSetting` |
| `src/models/xray.rs` | `XraySetting`, `OutboundTraffic`, `WarpAction`, `NordAction` |
| `src/models/custom_geo.rs` | `CustomGeoResource`, `CreateCustomGeo` |
| `src/api/mod.rs` | Re-exports, shared HTTP helpers |
| `src/api/inbounds.rs` | `InboundsApi` — 22 methods |
| `src/api/server.rs` | `ServerApi` — 18 methods |
| `src/api/settings.rs` | `SettingsApi` — 6 methods |
| `src/api/xray.rs` | `XrayApi` — 9 methods |
| `src/api/custom_geo.rs` | `CustomGeoApi` — 7 methods |
| `examples/list_inbounds.rs` | Usage example |
| `examples/add_client.rs` | Usage example |
| `.github/workflows/ci.yml` | Test + lint on every push |
| `.github/workflows/release.yml` | Cross-compile + publish on tag |

---

## Task 1: Scaffold Cargo project

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize the library**

```bash
cd /Users/farhad/projects/3x-ui-rs
cargo init --lib --name threexui-rs .
```

Expected: creates `Cargo.toml` and `src/lib.rs`.

- [ ] **Step 2: Replace Cargo.toml with full metadata**

Replace the generated `Cargo.toml` with:

```toml
[package]
name = "threexui-rs"
version = "2.9.3"
edition = "2021"
rust-version = "1.75"
description = "Async Rust SDK for the 3x-ui panel API (targets 3x-ui v2.9.3)"
license = "MIT"
repository = "https://github.com/YOUR_GITHUB_USERNAME/3x-ui-rs"
keywords = ["3x-ui", "xray", "vpn", "proxy", "api-client"]
categories = ["api-bindings", "network-programming"]
readme = "README.md"

[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["json", "cookies", "multipart", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
wiremock = "0.6"
tokio = { version = "1", features = ["rt", "macros"] }
```

- [ ] **Step 3: Write skeleton lib.rs**

```rust
pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod models;

pub use client::Client;
pub use config::ClientConfig;
pub use error::{Error, Result};
```

- [ ] **Step 4: Create placeholder modules so it compiles**

```bash
mkdir -p src/api src/models
touch src/api/mod.rs src/models/mod.rs
touch src/api/inbounds.rs src/api/server.rs src/api/settings.rs src/api/xray.rs src/api/custom_geo.rs
touch src/models/common.rs src/models/inbound.rs src/models/server.rs src/models/settings.rs src/models/xray.rs src/models/custom_geo.rs
touch src/error.rs src/config.rs src/client.rs
```

- [ ] **Step 5: Verify project structure compiles**

```bash
cargo check
```

Expected: no errors (empty modules are valid).

- [ ] **Step 6: Commit**

```bash
git init
echo "target/" > .gitignore
git add .
git commit -m "chore: scaffold threexui-rs library project"
```

---

## Task 2: Error module

**Files:**
- Modify: `src/error.rs`

- [ ] **Step 1: Write the failing test**

Add to `src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not authenticated — call login() first")]
    NotAuthenticated,

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("api error: {0}")]
    Api(String),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid config: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_not_authenticated() {
        let e = Error::NotAuthenticated;
        assert_eq!(e.to_string(), "not authenticated — call login() first");
    }

    #[test]
    fn error_display_api() {
        let e = Error::Api("bad request".to_string());
        assert_eq!(e.to_string(), "api error: bad request");
    }

    #[test]
    fn error_display_config() {
        let e = Error::Config("port cannot be 0".to_string());
        assert_eq!(e.to_string(), "invalid config: port cannot be 0");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test error
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/error.rs
git commit -m "feat: add Error enum and Result alias"
```

---

## Task 3: Config module

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub base_path: String,
    pub tls: bool,
    pub accept_invalid_certs: bool,
    pub timeout_secs: u64,
}

impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }

    pub fn base_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{}://{}:{}{}", scheme, self.host, self.port, self.base_path)
    }
}

#[derive(Debug, Default)]
pub struct ClientConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    base_path: Option<String>,
    tls: bool,
    accept_invalid_certs: bool,
    timeout_secs: Option<u64>,
}

impl ClientConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(normalize_base_path(path.into()));
        self
    }

    pub fn tls(mut self, tls: bool) -> Self {
        self.tls = tls;
        self
    }

    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn build(self) -> crate::Result<ClientConfig> {
        let host = self.host.ok_or_else(|| crate::Error::Config("host is required".into()))?;
        let port = self.port.ok_or_else(|| crate::Error::Config("port is required".into()))?;
        if port == 0 {
            return Err(crate::Error::Config("port cannot be 0".into()));
        }
        Ok(ClientConfig {
            host,
            port,
            base_path: self.base_path.unwrap_or_else(|| "/".to_string()),
            tls: self.tls,
            accept_invalid_certs: self.accept_invalid_certs,
            timeout_secs: self.timeout_secs.unwrap_or(30),
        })
    }
}

fn normalize_base_path(mut path: String) -> String {
    if !path.starts_with('/') {
        path.insert(0, '/');
    }
    if !path.ends_with('/') {
        path.push('/');
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_minimal_config() {
        let cfg = ClientConfig::builder()
            .host("192.168.1.1")
            .port(2053)
            .build()
            .unwrap();
        assert_eq!(cfg.host, "192.168.1.1");
        assert_eq!(cfg.port, 2053);
        assert_eq!(cfg.base_path, "/");
        assert!(!cfg.tls);
        assert_eq!(cfg.timeout_secs, 30);
    }

    #[test]
    fn build_full_config() {
        let cfg = ClientConfig::builder()
            .host("example.com")
            .port(443)
            .base_path("secret")
            .tls(true)
            .accept_invalid_certs(true)
            .timeout_secs(60)
            .build()
            .unwrap();
        assert_eq!(cfg.base_path, "/secret/");
        assert!(cfg.tls);
        assert!(cfg.accept_invalid_certs);
        assert_eq!(cfg.timeout_secs, 60);
    }

    #[test]
    fn base_url_http() {
        let cfg = ClientConfig::builder()
            .host("192.168.1.1")
            .port(2053)
            .build()
            .unwrap();
        assert_eq!(cfg.base_url(), "http://192.168.1.1:2053/");
    }

    #[test]
    fn base_url_https_with_path() {
        let cfg = ClientConfig::builder()
            .host("example.com")
            .port(443)
            .base_path("/secret/")
            .tls(true)
            .build()
            .unwrap();
        assert_eq!(cfg.base_url(), "https://example.com:443/secret/");
    }

    #[test]
    fn missing_host_errors() {
        let err = ClientConfig::builder().port(2053).build().unwrap_err();
        assert!(err.to_string().contains("host is required"));
    }

    #[test]
    fn port_zero_errors() {
        let err = ClientConfig::builder()
            .host("localhost")
            .port(0)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("port cannot be 0"));
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test config
```

Expected: 6 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/config.rs
git commit -m "feat: add ClientConfig with builder"
```

---

## Task 4: Common models (ApiResponse)

**Files:**
- Modify: `src/models/common.rs`

- [ ] **Step 1: Write ApiResponse**

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub success: bool,
    pub msg: String,
    pub obj: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn into_result(self) -> crate::Result<Option<T>> {
        if self.success {
            Ok(self.obj)
        } else {
            Err(crate::Error::Api(self.msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response_ok() {
        let raw = r#"{"success":true,"msg":"","obj":42}"#;
        let resp: ApiResponse<i32> = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.into_result().unwrap(), Some(42));
    }

    #[test]
    fn failed_response_is_err() {
        let raw = r#"{"success":false,"msg":"bad input","obj":null}"#;
        let resp: ApiResponse<i32> = serde_json::from_str(raw).unwrap();
        let err = resp.into_result().unwrap_err();
        assert!(err.to_string().contains("bad input"));
    }
}
```

- [ ] **Step 2: Update `src/models/mod.rs` to expose common**

```rust
pub(crate) mod common;
pub mod inbound;
pub mod server;
pub mod settings;
pub mod xray;
pub mod custom_geo;
```

- [ ] **Step 3: Run tests**

```bash
cargo test models::common
```

Expected: 2 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/models/
git commit -m "feat: add ApiResponse<T> internal model"
```

---

## Task 5: Inbound models

**Files:**
- Modify: `src/models/inbound.rs`

- [ ] **Step 1: Write the full inbound model**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
```

- [ ] **Step 2: Run tests**

```bash
cargo test models::inbound
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/models/inbound.rs
git commit -m "feat: add Inbound, Protocol, InboundClient, ClientTraffic models"
```

---

## Task 6: Server models

**Files:**
- Modify: `src/models/server.rs`

- [ ] **Step 1: Write server status and crypto key models**

```rust
use serde::{Deserialize, Serialize};

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
```

- [ ] **Step 2: Run tests**

```bash
cargo test models::server
```

Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/models/server.rs
git commit -m "feat: add ServerStatus and crypto key response models"
```

---

## Task 7: Settings model

**Files:**
- Modify: `src/models/settings.rs`

- [ ] **Step 1: Write AllSetting with all fields from entity.go**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AllSetting {
    // Web server settings
    #[serde(default)] pub web_listen: String,
    #[serde(default)] pub web_domain: String,
    #[serde(default)] pub web_port: i32,
    #[serde(default)] pub web_cert_file: String,
    #[serde(default)] pub web_key_file: String,
    #[serde(default)] pub web_base_path: String,
    #[serde(default)] pub session_max_age: i32,
    // UI settings
    #[serde(default)] pub page_size: i32,
    #[serde(default)] pub expire_diff: i32,
    #[serde(default)] pub traffic_diff: i32,
    #[serde(default)] pub remark_model: String,
    #[serde(default)] pub datepicker: String,
    // Telegram bot settings
    #[serde(default)] pub tg_bot_enable: bool,
    #[serde(default)] pub tg_bot_token: String,
    #[serde(default)] pub tg_bot_proxy: String,
    #[serde(default)] pub tg_bot_api_server: String,
    #[serde(default)] pub tg_bot_chat_id: String,
    #[serde(default)] pub tg_run_time: String,
    #[serde(default)] pub tg_bot_backup: bool,
    #[serde(default)] pub tg_bot_login_notify: bool,
    #[serde(default)] pub tg_cpu: i32,
    #[serde(default)] pub tg_lang: String,
    // Security settings
    #[serde(default)] pub time_location: String,
    #[serde(default)] pub two_factor_enable: bool,
    #[serde(default)] pub two_factor_token: String,
    // Subscription settings
    #[serde(default)] pub sub_enable: bool,
    #[serde(default)] pub sub_json_enable: bool,
    #[serde(default)] pub sub_title: String,
    #[serde(default)] pub sub_support_url: String,
    #[serde(default)] pub sub_profile_url: String,
    #[serde(default)] pub sub_announce: String,
    #[serde(default)] pub sub_enable_routing: bool,
    #[serde(default)] pub sub_routing_rules: String,
    #[serde(default)] pub sub_listen: String,
    #[serde(default)] pub sub_port: i32,
    #[serde(default)] pub sub_path: String,
    #[serde(default)] pub sub_domain: String,
    #[serde(default)] pub sub_cert_file: String,
    #[serde(default)] pub sub_key_file: String,
    #[serde(default)] pub sub_updates: i32,
    #[serde(default)] pub external_traffic_inform_enable: bool,
    #[serde(default)] pub external_traffic_inform_uri: String,
    #[serde(default)] pub sub_encrypt: bool,
    #[serde(default)] pub sub_show_info: bool,
    #[serde(default)] pub sub_uri: String,
    #[serde(default)] pub sub_json_path: String,
    #[serde(default)] pub sub_json_uri: String,
    #[serde(default)] pub sub_clash_enable: bool,
    #[serde(default)] pub sub_clash_path: String,
    #[serde(default)] pub sub_clash_uri: String,
    #[serde(default)] pub sub_json_fragment: String,
    #[serde(default)] pub sub_json_noises: String,
    #[serde(default)] pub sub_json_mux: String,
    #[serde(default)] pub sub_json_rules: String,
    // LDAP settings
    #[serde(default)] pub ldap_enable: bool,
    #[serde(default)] pub ldap_host: String,
    #[serde(default)] pub ldap_port: i32,
    #[serde(default)] pub ldap_use_tls: bool,
    #[serde(default)] pub ldap_bind_dn: String,
    #[serde(default)] pub ldap_password: String,
    #[serde(default)] pub ldap_base_dn: String,
    #[serde(default)] pub ldap_user_filter: String,
    #[serde(default)] pub ldap_user_attr: String,
    #[serde(default)] pub ldap_vless_field: String,
    #[serde(default)] pub ldap_sync_cron: String,
    #[serde(default)] pub ldap_flag_field: String,
    #[serde(default)] pub ldap_truthy_values: String,
    #[serde(default)] pub ldap_invert_flag: bool,
    #[serde(default)] pub ldap_inbound_tags: String,
    #[serde(default)] pub ldap_auto_create: bool,
    #[serde(default)] pub ldap_auto_delete: bool,
    #[serde(default)] pub ldap_default_total_gb: i32,
    #[serde(default)] pub ldap_default_expiry_days: i32,
    #[serde(default)] pub ldap_default_limit_ip: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_setting_deserializes_partial() {
        let raw = r#"{"webPort":2053,"tgBotEnable":false,"subEnable":true}"#;
        let s: AllSetting = serde_json::from_str(raw).unwrap();
        assert_eq!(s.web_port, 2053);
        assert!(!s.tg_bot_enable);
        assert!(s.sub_enable);
    }

    #[test]
    fn all_setting_empty_defaults() {
        let s: AllSetting = serde_json::from_str("{}").unwrap();
        assert_eq!(s.web_port, 0);
        assert_eq!(s.sub_title, "");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test models::settings
```

Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/models/settings.rs
git commit -m "feat: add AllSetting model with all fields"
```

---

## Task 8: Xray and CustomGeo models

**Files:**
- Modify: `src/models/xray.rs`
- Modify: `src/models/custom_geo.rs`

- [ ] **Step 1: Write xray models**

```rust
// src/models/xray.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySetting {
    pub xray_setting: serde_json::Value,
    pub inbound_tags: serde_json::Value,
    pub outbound_test_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutboundTraffic {
    pub id: i64,
    pub tag: String,
    pub up: i64,
    pub down: i64,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub enum WarpAction {
    Data,
    Delete,
    Config,
    Register { private_key: String, public_key: String },
    SetLicense(String),
}

impl WarpAction {
    pub fn action_str(&self) -> &str {
        match self {
            WarpAction::Data => "data",
            WarpAction::Delete => "del",
            WarpAction::Config => "config",
            WarpAction::Register { .. } => "reg",
            WarpAction::SetLicense(_) => "license",
        }
    }
}

#[derive(Debug, Clone)]
pub enum NordAction {
    Countries,
    Servers { country_id: String },
    Register { token: String },
    SetKey(String),
    Data,
    Delete,
}

impl NordAction {
    pub fn action_str(&self) -> &str {
        match self {
            NordAction::Countries => "countries",
            NordAction::Servers { .. } => "servers",
            NordAction::Register { .. } => "reg",
            NordAction::SetKey(_) => "setKey",
            NordAction::Data => "data",
            NordAction::Delete => "del",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warp_action_str() {
        assert_eq!(WarpAction::Data.action_str(), "data");
        assert_eq!(WarpAction::Delete.action_str(), "del");
        assert_eq!(WarpAction::Register {
            private_key: "a".into(),
            public_key: "b".into()
        }.action_str(), "reg");
    }

    #[test]
    fn nord_action_str() {
        assert_eq!(NordAction::Countries.action_str(), "countries");
        assert_eq!(NordAction::Servers { country_id: "1".into() }.action_str(), "servers");
    }
}
```

- [ ] **Step 2: Write custom_geo model**

```rust
// src/models/custom_geo.rs
use serde::{Deserialize, Serialize};

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
```

- [ ] **Step 3: Run all model tests**

```bash
cargo test models
```

Expected: all model tests pass.

- [ ] **Step 4: Update lib.rs to re-export models**

```rust
// src/lib.rs
pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod models;

pub use client::Client;
pub use config::ClientConfig;
pub use error::{Error, Result};

pub use models::inbound::{ClientTraffic, Inbound, InboundClient, Protocol};
pub use models::server::{
    AppStats, CpuHistoryPoint, EchCert, Mldsa65Keys, Mlkem768Keys, NetIO, NetTraffic, PublicIP,
    ResourceStat, ServerStatus, UuidResponse, VlessAuth, VlessEncResult, X25519Cert, XrayState,
};
pub use models::settings::AllSetting;
pub use models::xray::{NordAction, OutboundTraffic, WarpAction, XraySetting};
pub use models::custom_geo::{CreateCustomGeo, CustomGeoResource};
```

- [ ] **Step 5: Commit**

```bash
git add src/models/ src/lib.rs
git commit -m "feat: add Xray, CustomGeo models and wire lib.rs exports"
```

---

## Task 9: Client core (authentication)

**Files:**
- Modify: `src/client.rs`

- [ ] **Step 1: Write failing tests for login/logout**

```rust
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
use crate::{Error};

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

    async fn login_inner(&self, username: &str, password: &str, two_factor: Option<&str>) -> Result<()> {
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
        let _ = self
            .inner
            .http
            .get(self.url("logout"))
            .send()
            .await?;
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
        assert!(matches!(client.require_auth(), Err(Error::NotAuthenticated)));
    }
}
```

- [ ] **Step 2: Add stub API structs so it compiles**

Add to `src/api/mod.rs`:

```rust
pub mod custom_geo;
pub mod inbounds;
pub mod server;
pub mod settings;
pub mod xray;
```

Add stubs to each api file, e.g. `src/api/inbounds.rs`:

```rust
use crate::Client;

pub struct InboundsApi<'a> {
    pub(crate) client: &'a Client,
}
```

Repeat the same stub pattern for `ServerApi`, `SettingsApi`, `XrayApi`, `CustomGeoApi` in their respective files.

- [ ] **Step 3: Run tests**

```bash
cargo test client
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/client.rs src/api/
git commit -m "feat: add Client with login/logout and namespace accessors"
```

---

## Task 10: API helpers

**Files:**
- Modify: `src/api/mod.rs`

Add shared HTTP helper methods to `Client` in `src/client.rs` (these are `pub(crate)` and used by all API modules):

- [ ] **Step 1: Add HTTP helpers to client.rs**

Append these methods to the `impl Client` block in `src/client.rs`:

```rust
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
```

- [ ] **Step 2: Verify compile**

```bash
cargo check
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/client.rs
git commit -m "feat: add shared HTTP helpers to Client (get, post, post_empty, get_bytes)"
```

---

## Task 11: InboundsApi — CRUD (list, get, add, update, delete, import)

**Files:**
- Modify: `src/api/inbounds.rs`

- [ ] **Step 1: Write failing tests for CRUD operations**

Replace `src/api/inbounds.rs`:

```rust
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
        self.client.get(&format!("panel/api/inbounds/get/{}", id)).await
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
            .post_empty(&format!("panel/api/inbounds/del/{}", id), &serde_json::json!({}))
            .await
    }

    pub async fn import(&self, inbound: &Inbound) -> Result<Inbound> {
        // The handler reads c.PostForm("data") so this must be form-encoded,
        // not a JSON body — the data field value is a JSON string.
        let data_str = serde_json::to_string(inbound)?;
        self.client.require_auth()?;
        let resp = self
            .client
            .inner
            .http
            .post(self.client.url("panel/api/inbounds/import"))
            .form(&[("data", data_str.as_str())])
            .send()
            .await?
            .json::<crate::models::common::ApiResponse<Inbound>>()
            .await?;
        resp.into_result()
            .and_then(|v| v.ok_or_else(|| crate::Error::Api("empty response".into())))
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
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test api::inbounds
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/api/inbounds.rs
git commit -m "feat: add InboundsApi CRUD methods (list, get, add, update, delete, import)"
```

---

## Task 12: InboundsApi — Client management

**Files:**
- Modify: `src/api/inbounds.rs`

- [ ] **Step 1: Add client management methods**

Append to `impl<'a> InboundsApi<'a>` in `src/api/inbounds.rs`:

```rust
    /// Adds new clients to an inbound. `clients` is a slice of client objects
    /// (format depends on protocol — see 3x-ui docs for schema per protocol).
    pub async fn add_client(
        &self,
        inbound_id: i64,
        clients: &[serde_json::Value],
    ) -> Result<()> {
        let settings = serde_json::json!({ "clients": clients }).to_string();
        let body = serde_json::json!({ "id": inbound_id, "settings": settings });
        self.client.post_empty("panel/api/inbounds/addClient", &body).await
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
            .post_empty(&format!("panel/api/inbounds/updateClient/{}", client_id), &body)
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
                &format!("panel/api/inbounds/{}/delClientByEmail/{}", inbound_id, email),
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
            .post(&format!("panel/api/inbounds/{}/copyClients", target_inbound_id), &body)
            .await
    }

    pub async fn client_ips(&self, email: &str) -> Result<serde_json::Value> {
        self.client
            .post(&format!("panel/api/inbounds/clientIps/{}", email), &serde_json::json!({}))
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
```

- [ ] **Step 2: Add tests for client management**

Append to the `#[cfg(test)]` block in `src/api/inbounds.rs`:

```rust
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
        client.inbounds().add_client(1, &[new_client]).await.unwrap();
    }

    #[tokio::test]
    async fn delete_client_by_email_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/panel/api/inbounds/2/delClientByEmail/user@example.com"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true, "msg": "deleted", "obj": null
            })))
            .mount(&server)
            .await;

        let client = auth_client(&server).await;
        client.inbounds().delete_client_by_email(2, "user@example.com").await.unwrap();
    }
```

- [ ] **Step 3: Run tests**

```bash
cargo test api::inbounds
```

Expected: 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/api/inbounds.rs
git commit -m "feat: add InboundsApi client management (add, update, delete, copy, ips)"
```

---

## Task 13: InboundsApi — Traffic management and online status

**Files:**
- Modify: `src/api/inbounds.rs`

- [ ] **Step 1: Add traffic and status methods**

Append to `impl<'a> InboundsApi<'a>`:

```rust
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
                &format!("panel/api/inbounds/{}/resetClientTraffic/{}", inbound_id, email),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn reset_all_traffics(&self) -> Result<()> {
        self.client
            .post_empty("panel/api/inbounds/resetAllTraffics", &serde_json::json!({}))
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
```

- [ ] **Step 2: Add tests for traffic methods**

Append to the `#[cfg(test)]` block:

```rust
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
```

- [ ] **Step 3: Run all inbound tests**

```bash
cargo test api::inbounds
```

Expected: 7 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/api/inbounds.rs
git commit -m "feat: add InboundsApi traffic management and online status methods"
```

---

## Task 14: ServerApi (all 18 methods)

**Files:**
- Modify: `src/api/server.rs`

- [ ] **Step 1: Write ServerApi with all 18 methods**

```rust
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
            .post_empty("panel/api/server/stopXrayService", &serde_json::json!({}))
            .await
    }

    pub async fn restart_xray(&self) -> Result<()> {
        self.client
            .post_empty("panel/api/server/restartXrayService", &serde_json::json!({}))
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
```

- [ ] **Step 2: Run tests**

```bash
cargo test api::server
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/api/server.rs
git commit -m "feat: add ServerApi with all 18 methods"
```

---

## Task 15: SettingsApi (all 6 methods)

**Files:**
- Modify: `src/api/settings.rs`

- [ ] **Step 1: Write SettingsApi**

```rust
use crate::models::settings::AllSetting;
use crate::{Client, Result};

pub struct SettingsApi<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> SettingsApi<'a> {
    pub async fn get_all(&self) -> Result<AllSetting> {
        self.client.post("panel/setting/all", &serde_json::json!({})).await
    }

    pub async fn get_defaults(&self) -> Result<serde_json::Value> {
        self.client.post("panel/setting/defaultSettings", &serde_json::json!({})).await
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
        self.client.post_empty("panel/setting/updateUser", &body).await
    }

    pub async fn restart_panel(&self) -> Result<()> {
        self.client
            .post_empty("panel/setting/restartPanel", &serde_json::json!({}))
            .await
    }

    pub async fn default_xray_config(&self) -> Result<serde_json::Value> {
        self.client.get("panel/setting/getDefaultJsonConfig").await
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
```

- [ ] **Step 2: Run tests**

```bash
cargo test api::settings
```

Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/api/settings.rs
git commit -m "feat: add SettingsApi with all 6 methods"
```

---

## Task 16: XrayApi (all 9 methods)

**Files:**
- Modify: `src/api/xray.rs`

- [ ] **Step 1: Write XrayApi**

```rust
use crate::models::xray::{NordAction, OutboundTraffic, WarpAction, XraySetting};
use crate::{Client, Result};

pub struct XrayApi<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> XrayApi<'a> {
    pub async fn get_setting(&self) -> Result<XraySetting> {
        self.client.post("panel/xray/", &serde_json::json!({})).await
    }

    pub async fn update_setting(&self, xray_config: &str, test_url: &str) -> Result<()> {
        let params = [("xraySetting", xray_config), ("outboundTestUrl", test_url)];
        self.client.post_form_empty("panel/xray/update", &params).await
    }

    pub async fn default_config(&self) -> Result<serde_json::Value> {
        self.client.get("panel/xray/getDefaultJsonConfig").await
    }

    pub async fn outbounds_traffic(&self) -> Result<Vec<OutboundTraffic>> {
        self.client.get("panel/xray/getOutboundsTraffic").await
    }

    pub async fn reset_outbound_traffic(&self, tag: &str) -> Result<()> {
        let body = serde_json::json!({ "tag": tag });
        self.client.post_empty("panel/xray/resetOutboundsTraffic", &body).await
    }

    pub async fn test_outbound(
        &self,
        outbound: &serde_json::Value,
        all_outbounds: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let outbound_str = serde_json::to_string(outbound)?;
        let all_str = all_outbounds
            .map(|v| serde_json::to_string(v))
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
        let req = self.client.inner.http.post(
            self.client.url(&path)
        );

        let req = match &action {
            WarpAction::Register { private_key, public_key } => req.form(&[
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
        resp.into_result().map(|v| v.unwrap_or(serde_json::Value::Null))
    }

    pub async fn nord(&self, action: NordAction) -> Result<serde_json::Value> {
        let action_str = action.action_str().to_string();
        let path = format!("panel/xray/nord/{}", action_str);

        self.client.require_auth()?;
        let req = self.client.inner.http.post(
            self.client.url(&path)
        );

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
        resp.into_result().map(|v| v.unwrap_or(serde_json::Value::Null))
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
```

- [ ] **Step 2: Run tests**

```bash
cargo test api::xray
```

Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/api/xray.rs
git commit -m "feat: add XrayApi with all 9 methods (warp, nord, outbound, config)"
```

---

## Task 17: CustomGeoApi (all 7 methods)

**Files:**
- Modify: `src/api/custom_geo.rs`

- [ ] **Step 1: Write CustomGeoApi**

```rust
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
```

- [ ] **Step 2: Run all tests**

```bash
cargo test
```

Expected: all tests pass (no failures).

- [ ] **Step 3: Commit**

```bash
git add src/api/custom_geo.rs
git commit -m "feat: add CustomGeoApi with all 7 methods"
```

---

## Task 18: Final lib.rs + full compile check

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Finalize lib.rs with complete re-exports**

```rust
//! # threexui-rs
//!
//! Async Rust SDK for the [3x-ui](https://github.com/MHSanaei/3x-ui) panel API.
//!
//! This crate targets 3x-ui **v2.9.3**. The library version mirrors the panel version —
//! `threexui-rs v2.9.3` is compatible with 3x-ui `v2.9.3`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use threexui_rs::{Client, ClientConfig};
//!
//! #[tokio::main]
//! async fn main() -> threexui_rs::Result<()> {
//!     let config = ClientConfig::builder()
//!         .host("192.168.1.1")
//!         .port(2053)
//!         .build()?;
//!
//!     let client = Client::new(config);
//!     client.login("admin", "admin123").await?;
//!
//!     let inbounds = client.inbounds().list().await?;
//!     println!("Found {} inbounds", inbounds.len());
//!
//!     client.logout().await?;
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod models;

pub use client::Client;
pub use config::ClientConfig;
pub use error::{Error, Result};

// Inbound models
pub use models::inbound::{ClientTraffic, Inbound, InboundClient, Protocol};

// Server models
pub use models::server::{
    AppStats, CpuHistoryPoint, EchCert, Mldsa65Keys, Mlkem768Keys, NetIO, NetTraffic, PublicIP,
    ResourceStat, ServerStatus, UuidResponse, VlessAuth, VlessEncResult, X25519Cert, XrayState,
};

// Settings
pub use models::settings::AllSetting;

// Xray models
pub use models::xray::{NordAction, OutboundTraffic, WarpAction, XraySetting};

// Custom geo models
pub use models::custom_geo::{CreateCustomGeo, CustomGeoResource};
```

- [ ] **Step 2: Run full test suite**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 3: Run clippy**

```bash
cargo clippy -- -D warnings
```

Fix any warnings before continuing.

- [ ] **Step 4: Run fmt check**

```bash
cargo fmt --check
```

Run `cargo fmt` if there are formatting issues.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: finalize lib.rs public API and doc comment"
```

---

## Task 19: Examples

**Files:**
- Create: `examples/list_inbounds.rs`
- Create: `examples/add_client.rs`

- [ ] **Step 1: Write list_inbounds example**

```rust
// examples/list_inbounds.rs
use threexui_rs::{Client, ClientConfig};

#[tokio::main]
async fn main() -> threexui_rs::Result<()> {
    let config = ClientConfig::builder()
        .host("192.168.1.1")
        .port(2053)
        .build()?;

    let client = Client::new(config);
    client.login("admin", "admin123").await?;

    let inbounds = client.inbounds().list().await?;
    for inbound in &inbounds {
        println!(
            "[{}] {} — {:?} port {} (enabled: {})",
            inbound.id, inbound.remark, inbound.protocol, inbound.port, inbound.enable
        );
        for stat in &inbound.client_stats {
            println!("  client: {} up={} down={}", stat.email, stat.up, stat.down);
        }
    }

    let status = client.server().status().await?;
    println!("\nServer CPU: {:.1}%  Xray: {}", status.cpu, status.xray.state);

    client.logout().await?;
    Ok(())
}
```

- [ ] **Step 2: Write add_client example**

```rust
// examples/add_client.rs
use threexui_rs::{Client, ClientConfig};

#[tokio::main]
async fn main() -> threexui_rs::Result<()> {
    let config = ClientConfig::builder()
        .host("192.168.1.1")
        .port(2053)
        .build()?;

    let client = Client::new(config);
    client.login("admin", "admin123").await?;

    // Generate a fresh UUID for the new VLESS client
    let uuid_resp = client.server().new_uuid().await?;
    println!("New UUID: {}", uuid_resp.uuid);

    // Build the client object as raw JSON (format is protocol-specific)
    let new_client = serde_json::json!({
        "id": uuid_resp.uuid,
        "email": "newuser@example.com",
        "enable": true,
        "flow": "",
        "limitIp": 0,
        "totalGB": 10,
        "expiryTime": 0,
        "tgId": 0,
        "subId": "",
        "comment": "",
        "reset": 0
    });

    // Add client to inbound with ID 1
    client.inbounds().add_client(1, &[new_client]).await?;
    println!("Client added successfully");

    client.logout().await?;
    Ok(())
}
```

- [ ] **Step 3: Verify examples compile**

```bash
cargo build --examples
```

Expected: both examples compile (they won't run without a live server, that's fine).

- [ ] **Step 4: Commit**

```bash
git add examples/
git commit -m "docs: add list_inbounds and add_client usage examples"
```

---

## Task 20: CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Create CI workflow**

```bash
mkdir -p .github/workflows
```

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: ["**"]
  pull_request:
    branches: ["**"]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test (${{ matrix.os }} / ${{ matrix.toolchain }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        toolchain: [stable, beta]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust ${{ matrix.toolchain }}
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.toolchain }}
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-${{ matrix.toolchain }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: cargo fmt check
        run: cargo fmt --all --check

      - name: cargo clippy
        run: cargo clippy --all-features -- -D warnings

      - name: cargo test
        run: cargo test --all-features
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add CI workflow (test, clippy, fmt on all OS + stable/beta)"
```

---

## Task 21: Release workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Create release workflow**

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - "v*.*.*"

env:
  CARGO_TERM_COLOR: always

jobs:
  test-gate:
    name: Test gate before publish
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features

  publish:
    name: Publish to crates.io
    runs-on: ubuntu-latest
    needs: test-gate
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Publish
        run: cargo publish --token ${{ secrets.CARGO_TOKEN }}

  cross-compile:
    name: Cross-compile check (${{ matrix.target }})
    runs-on: ${{ matrix.runner }}
    needs: test-gate
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            runner: ubuntu-latest
          - target: x86_64-unknown-linux-musl
            runner: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            runner: ubuntu-latest
          - target: aarch64-unknown-linux-musl
            runner: ubuntu-latest
          - target: armv7-unknown-linux-gnueabihf
            runner: ubuntu-latest
          - target: riscv64gc-unknown-linux-gnu
            runner: ubuntu-latest
          - target: x86_64-apple-darwin
            runner: macos-latest
          - target: aarch64-apple-darwin
            runner: macos-latest
          - target: x86_64-pc-windows-msvc
            runner: windows-latest
          - target: i686-pc-windows-msvc
            runner: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross (Linux non-native targets)
        if: runner.os == 'Linux'
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build (cross — Linux)
        if: runner.os == 'Linux'
        run: cross build --target ${{ matrix.target }} --release

      - name: Build (cargo — macOS / Windows)
        if: runner.os != 'Linux'
        run: cargo build --target ${{ matrix.target }} --release

  create-release:
    name: Create GitHub Release
    runs-on: ubuntu-latest
    needs: [publish, cross-compile]
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - name: Create release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          body: |
            ## threexui-rs ${{ github.ref_name }}

            Rust SDK targeting **3x-ui ${{ github.ref_name }}**.

            Install via Cargo:
            ```toml
            [dependencies]
            threexui-rs = "${{ github.ref_name }}"
            ```
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add release workflow with crates.io publish and cross-compile matrix"
```

---

## Task 22: Cargo metadata and README

**Files:**
- Modify: `Cargo.toml` (already has metadata — add `readme` and `documentation` fields)
- Create: `README.md`
- Create: `LICENSE`

- [ ] **Step 1: Add final Cargo.toml fields**

Add these lines to the `[package]` section of `Cargo.toml`:

```toml
documentation = "https://docs.rs/threexui-rs"
exclude = ["/docs", "/.github"]
```

- [ ] **Step 2: Create LICENSE (MIT)**

```
MIT License

Copyright (c) 2026 Farhad

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 3: Create README.md**

```markdown
# threexui-rs

Async Rust SDK for the [3x-ui](https://github.com/MHSanaei/3x-ui) panel API.

[![Crates.io](https://img.shields.io/crates/v/threexui-rs)](https://crates.io/crates/threexui-rs)
[![docs.rs](https://docs.rs/threexui-rs/badge.svg)](https://docs.rs/threexui-rs)
[![CI](https://github.com/YOUR_USERNAME/3x-ui-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/YOUR_USERNAME/3x-ui-rs/actions)

## Version compatibility

| threexui-rs | 3x-ui panel |
|-------------|-------------|
| 2.9.3       | v2.9.3      |

## Installation

```toml
[dependencies]
threexui-rs = "2.9.3"
tokio = { version = "1", features = ["full"] }
```

## Quick start

```rust
use threexui_rs::{Client, ClientConfig};

#[tokio::main]
async fn main() -> threexui_rs::Result<()> {
    let config = ClientConfig::builder()
        .host("192.168.1.1")
        .port(2053)
        .build()?;

    let client = Client::new(config);
    client.login("admin", "admin123").await?;

    let inbounds = client.inbounds().list().await?;
    println!("Found {} inbounds", inbounds.len());

    let status = client.server().status().await?;
    println!("CPU: {:.1}%", status.cpu);

    client.logout().await?;
    Ok(())
}
```

## API coverage

| Namespace | Methods | Description |
|-----------|---------|-------------|
| `client.inbounds()` | 22 | Inbound CRUD, client management, traffic |
| `client.server()` | 18 | Status, Xray control, key generation, logs |
| `client.settings()` | 6 | Panel configuration, user management |
| `client.xray()` | 9 | Xray config, Warp, NordVPN, outbounds |
| `client.custom_geo()` | 7 | Custom geo resource management |

## Publishing a new version for a new 3x-ui release

1. Update `version` in `Cargo.toml` to match the new 3x-ui version
2. Update `Cargo.lock`: `cargo update`
3. Commit and push a tag: `git tag v2.9.4 && git push origin v2.9.4`
4. The release workflow publishes to crates.io automatically

## License

MIT
```

- [ ] **Step 4: Final cargo check and test**

```bash
cargo test
cargo clippy -- -D warnings
```

Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml LICENSE README.md
git commit -m "docs: add README, LICENSE, and finalize Cargo.toml metadata"
```

---

## Post-implementation checklist

- [ ] Replace `YOUR_GITHUB_USERNAME` in `Cargo.toml` and `README.md` with actual GitHub username
- [ ] Add `CARGO_TOKEN` secret in GitHub repository settings (from crates.io account → API Tokens)
- [ ] Push to GitHub: `git remote add origin https://github.com/YOUR_USERNAME/3x-ui-rs && git push -u origin main`
- [ ] Test against a real 3x-ui instance with `cargo run --example list_inbounds`
- [ ] When ready: `git tag v2.9.3 && git push origin v2.9.3` to trigger release workflow
