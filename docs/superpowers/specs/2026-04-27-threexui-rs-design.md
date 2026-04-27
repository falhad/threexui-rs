# threexui-rs — Rust SDK Design Spec

**Date:** 2026-04-27  
**Target:** 3x-ui v2.9.3  
**Crate:** `threexui-rs` v2.9.3 on crates.io  

---

## Overview

A general-purpose async Rust SDK for the [3x-ui](https://github.com/MHSanaei/3x-ui) panel API.  
Covers all ~60 endpoints derived directly from the 3x-ui v2.9.3 source code (not docs).  
Designed for automation, CLI tooling, and embedding in larger Rust services.

**Versioning policy:** library version tracks 3x-ui release version exactly.  
`threexui-rs v2.9.3` targets 3x-ui `v2.9.3`. Each new 3x-ui release gets a matching library release.

---

## Goals

- Full coverage of all 3x-ui v2.9.3 API endpoints (~60 total)
- Async-only using `tokio` + `reqwest`
- Ergonomic namespaced accessor pattern: `client.inbounds().list().await?`
- Cross-platform: compiles on Linux, macOS, Windows across x86_64, aarch64, armv7, musl targets
- Publishable to crates.io
- GitHub Actions CI/CD for testing, cross-compilation validation, and release publishing

---

## Non-Goals

- Sync API (out of scope for v1)
- Strongly-typed protocol-specific settings (VMess/VLESS/Trojan settings stay as `serde_json::Value`)
- WebSocket support (panel has a WS endpoint but SDK will use REST only)

---

## Crate Layout

```
threexui-rs/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Re-exports: Client, ClientConfig, Error, all model types
│   ├── client.rs           # Client struct, login/logout, namespace accessors
│   ├── config.rs           # ClientConfig with builder pattern
│   ├── error.rs            # Error enum + Result<T> alias
│   ├── api/
│   │   ├── mod.rs
│   │   ├── inbounds.rs     # InboundsApi — 22 endpoints
│   │   ├── server.rs       # ServerApi — 18 endpoints
│   │   ├── settings.rs     # SettingsApi — 6 endpoints
│   │   ├── xray.rs         # XrayApi — 9 endpoints
│   │   └── custom_geo.rs   # CustomGeoApi — 7 endpoints
│   └── models/
│       ├── mod.rs
│       ├── inbound.rs      # Inbound, Client, ClientTraffic, Protocol
│       ├── server.rs       # ServerStatus, XrayVersion, LogEntry, CpuHistoryPoint
│       ├── settings.rs     # AllSetting
│       ├── xray.rs         # XraySetting, WarpAction, NordAction, OutboundTraffic
│       ├── custom_geo.rs   # CustomGeoResource
│       └── common.rs       # ApiResponse<T> (internal), helper types
├── examples/
│   ├── list_inbounds.rs
│   └── add_client.rs
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

---

## Architecture

### Client

```rust
pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    http: reqwest::Client,       // cookie store enabled
    base_url: String,            // e.g. "http://192.168.1.1:2053/"
    authenticated: AtomicBool,
}
```

The `Client` is cheap to clone (`Arc` inside). All namespace accessors borrow from it:

```rust
impl Client {
    pub fn new(config: ClientConfig) -> Self { ... }
    pub async fn login(&self, username: &str, password: &str) -> Result<()>
    pub async fn login_2fa(&self, username: &str, password: &str, code: &str) -> Result<()>
    pub async fn logout(&self) -> Result<()>
    pub async fn is_two_factor_enabled(&self) -> Result<bool>
    pub async fn backup_to_tgbot(&self) -> Result<()>

    pub fn inbounds(&self) -> InboundsApi<'_>
    pub fn server(&self) -> ServerApi<'_>
    pub fn settings(&self) -> SettingsApi<'_>
    pub fn xray(&self) -> XrayApi<'_>
    pub fn custom_geo(&self) -> CustomGeoApi<'_>
}
```

Each namespace struct holds `&'_ Client` and has no state of its own.

### Config

```rust
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub base_path: String,       // default: "/"
    pub tls: bool,               // default: false (http)
    pub accept_invalid_certs: bool, // default: true (self-signed friendly)
    pub timeout_secs: u64,       // default: 30
}

impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder { ... }
}
```

Usage:
```rust
let config = ClientConfig::builder()
    .host("192.168.1.1")
    .port(2053)
    .base_path("/secret/")   // optional, default "/"
    .tls(true)
    .build()?;
```

### Authentication

3x-ui uses cookie-based session auth. `reqwest::Client` is built with `cookie_store(true)`.  
`login()` POSTs to `/{base_path}login` and the session cookie is automatically stored.  
All subsequent requests carry the cookie automatically.  
`authenticated: AtomicBool` is set to `true` after successful login; methods check it and return `Error::NotAuthenticated` if false.

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
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
```

All 3x-ui responses follow `{"success": bool, "msg": string, "obj": T}`.  
The SDK deserializes this internally via `ApiResponse<T>` and converts `success: false` → `Error::Api(msg)`.  
Callers only ever receive the typed `T` on success.

---

## Models

### Inbound
```rust
pub struct Inbound {
    pub id: i64,
    pub remark: String,
    pub enable: bool,
    pub protocol: Protocol,
    pub port: u16,
    pub listen: String,
    pub up: i64,
    pub down: i64,
    pub total: i64,
    pub all_time: i64,
    pub expiry_time: i64,
    pub traffic_reset: String,
    pub tag: String,
    pub settings: serde_json::Value,
    pub stream_settings: serde_json::Value,
    pub sniffing: serde_json::Value,
    pub client_stats: Vec<ClientTraffic>,
}

pub enum Protocol {
    VMess, VLess, Trojan, Shadowsocks,
    Hysteria, Hysteria2, WireGuard, HTTP, Mixed,
}
```

### Client (inbound client)
```rust
pub struct InboundClient {
    pub id: String,
    pub email: String,
    pub enable: bool,
    pub flow: String,
    pub password: String,
    pub security: String,
    pub limit_ip: i32,
    pub total_gb: i64,
    pub expiry_time: i64,
    pub tg_id: i64,
    pub sub_id: String,
    pub comment: String,
    pub reset: i32,
}
```

### ClientTraffic
```rust
pub struct ClientTraffic {
    pub id: i64,
    pub inbound_id: i64,
    pub email: String,
    pub enable: bool,
    pub up: i64,
    pub down: i64,
    pub all_time: i64,
    pub total: i64,
    pub expiry_time: i64,
    pub reset: i32,
    pub last_online: i64,
}
```

### ServerStatus
```rust
pub struct ServerStatus {
    pub cpu: f64,
    pub mem: MemInfo,
    pub swap: SwapInfo,
    pub disk: DiskInfo,
    pub xray: XrayInfo,
    pub uptime: u64,
    pub loads: Vec<f64>,
    pub tcp_count: i64,
    pub udp_count: i64,
    pub net_io: NetIO,
    pub net_traffic: NetTraffic,
    pub public_ip: PublicIP,
    pub app_stats: AppStats,
}
```

`settings`/`stream_settings`/`sniffing` on `Inbound` remain `serde_json::Value` — these are protocol-specific blobs that the panel itself stores as raw JSON.

---

## API Namespace — Full Endpoint Map

### `client.inbounds()` — 22 methods
| Method | HTTP | Path |
|--------|------|------|
| `list()` | GET | `/inbounds/list` |
| `get(id)` | GET | `/inbounds/get/:id` |
| `add(inbound)` | POST | `/inbounds/add` |
| `update(id, inbound)` | POST | `/inbounds/update/:id` |
| `delete(id)` | POST | `/inbounds/del/:id` |
| `import(inbound)` | POST | `/inbounds/import` |
| `client_traffics_by_email(email)` | GET | `/inbounds/getClientTraffics/:email` |
| `client_traffics_by_id(id)` | GET | `/inbounds/getClientTrafficsById/:id` |
| `client_ips(email)` | POST | `/inbounds/clientIps/:email` |
| `clear_client_ips(email)` | POST | `/inbounds/clearClientIps/:email` |
| `add_client(inbound_id, clients)` | POST | `/inbounds/addClient` |
| `update_client(client_id, client)` | POST | `/inbounds/updateClient/:clientId` |
| `delete_client(inbound_id, client_id)` | POST | `/inbounds/:id/delClient/:clientId` |
| `delete_client_by_email(inbound_id, email)` | POST | `/inbounds/:id/delClientByEmail/:email` |
| `copy_clients(target_id, source_id, emails, flow)` | POST | `/inbounds/:id/copyClients` |
| `reset_client_traffic(inbound_id, email)` | POST | `/inbounds/:id/resetClientTraffic/:email` |
| `reset_all_traffics()` | POST | `/inbounds/resetAllTraffics` |
| `reset_all_client_traffics(inbound_id)` | POST | `/inbounds/resetAllClientTraffics/:id` |
| `delete_depleted_clients(inbound_id)` | POST | `/inbounds/delDepletedClients/:id` |
| `online_clients()` | POST | `/inbounds/onlines` |
| `last_online()` | POST | `/inbounds/lastOnline` |
| `update_client_traffic(email, upload, download)` | POST | `/inbounds/updateClientTraffic/:email` |

### `client.server()` — 18 methods
| Method | HTTP | Path |
|--------|------|------|
| `status()` | GET | `/server/status` |
| `cpu_history(bucket)` | GET | `/server/cpuHistory/:bucket` |
| `xray_versions()` | GET | `/server/getXrayVersion` |
| `config_json()` | GET | `/server/getConfigJson` |
| `download_db()` | GET | `/server/getDb` |
| `new_uuid()` | GET | `/server/getNewUUID` |
| `new_x25519_cert()` | GET | `/server/getNewX25519Cert` |
| `new_mldsa65()` | GET | `/server/getNewmldsa65` |
| `new_mlkem768()` | GET | `/server/getNewmlkem768` |
| `new_vless_enc()` | GET | `/server/getNewVlessEnc` |
| `stop_xray()` | POST | `/server/stopXrayService` |
| `restart_xray()` | POST | `/server/restartXrayService` |
| `install_xray(version)` | POST | `/server/installXray/:version` |
| `update_geofile(filename)` | POST | `/server/updateGeofile[/:fileName]` |
| `logs(count, level, syslog)` | POST | `/server/logs/:count` |
| `xray_logs(count, filter, opts)` | POST | `/server/xraylogs/:count` |
| `import_db(data)` | POST | `/server/importDB` |
| `new_ech_cert(sni)` | POST | `/server/getNewEchCert` |

### `client.settings()` — 6 methods
| Method | HTTP | Path |
|--------|------|------|
| `get_all()` | POST | `/setting/all` |
| `get_defaults()` | POST | `/setting/defaultSettings` |
| `update(settings)` | POST | `/setting/update` |
| `update_user(old_user, old_pass, new_user, new_pass)` | POST | `/setting/updateUser` |
| `restart_panel()` | POST | `/setting/restartPanel` |
| `default_xray_config()` | GET | `/setting/getDefaultJsonConfig` |

### `client.xray()` — 9 methods
| Method | HTTP | Path |
|--------|------|------|
| `get_setting()` | POST | `/xray/` |
| `update_setting(config, test_url)` | POST | `/xray/update` |
| `default_config()` | GET | `/xray/getDefaultJsonConfig` |
| `outbounds_traffic()` | GET | `/xray/getOutboundsTraffic` |
| `reset_outbound_traffic(tag)` | POST | `/xray/resetOutboundsTraffic` |
| `test_outbound(outbound, all_outbounds)` | POST | `/xray/testOutbound` |
| `xray_result()` | GET | `/xray/getXrayResult` |
| `warp(action)` | POST | `/xray/warp/:action` |
| `nord(action)` | POST | `/xray/nord/:action` |

### `client.custom_geo()` — 7 methods
| Method | HTTP | Path |
|--------|------|------|
| `list()` | GET | `/custom-geo/list` |
| `aliases()` | GET | `/custom-geo/aliases` |
| `add(geo)` | POST | `/custom-geo/add` |
| `update(id, geo)` | POST | `/custom-geo/update/:id` |
| `delete(id)` | POST | `/custom-geo/delete/:id` |
| `download(id)` | POST | `/custom-geo/download/:id` |
| `update_all()` | POST | `/custom-geo/update-all` |

---

## CI/CD Workflows

### `ci.yml` — Runs on every push and pull request
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test --all-features`
- Matrix: `[ubuntu-latest, macos-latest, windows-latest]` × `[stable, beta]`

### `release.yml` — Triggered by tag push `v*.*.*`
1. `cargo test` (gate)
2. `cargo publish --token $CARGO_TOKEN` → crates.io
3. Cross-compile validation matrix via `cross` tool:
   - `x86_64-unknown-linux-gnu`
   - `x86_64-unknown-linux-musl`
   - `aarch64-unknown-linux-gnu`
   - `aarch64-unknown-linux-musl`
   - `x86_64-apple-darwin`
   - `aarch64-apple-darwin`
   - `x86_64-pc-windows-msvc`
   - `i686-pc-windows-msvc`
   - `armv7-unknown-linux-gnueabihf`
   - `riscv64gc-unknown-linux-gnu`
4. Upload build artifacts to GitHub Release

**Required GitHub secret:** `CARGO_TOKEN` (from crates.io account settings)

---

## Dependencies

```toml
[dependencies]
reqwest   = { version = "0.12", features = ["json", "cookies", "multipart"] }
tokio     = { version = "1", features = ["full"] }
serde     = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"

[dev-dependencies]
tokio     = { version = "1", features = ["full", "test-util"] }
```

---

## Usage Example

```rust
use threexui_rs::{Client, ClientConfig};

#[tokio::main]
async fn main() -> threexui_rs::Result<()> {
    let config = ClientConfig::builder()
        .host("192.168.1.1")
        .port(2053)
        .base_path("/")
        .build()?;

    let client = Client::new(config);
    client.login("admin", "admin123").await?;

    let inbounds = client.inbounds().list().await?;
    for inbound in &inbounds {
        println!("{}: {} (port {})", inbound.id, inbound.remark, inbound.port);
    }

    let status = client.server().status().await?;
    println!("CPU: {}%", status.cpu);

    client.logout().await?;
    Ok(())
}
```
