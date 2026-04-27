# threexui-rs

Async Rust SDK for the [3x-ui](https://github.com/MHSanaei/3x-ui) panel API.

[![Crates.io](https://img.shields.io/crates/v/threexui-rs)](https://crates.io/crates/threexui-rs)
[![docs.rs](https://docs.rs/threexui-rs/badge.svg)](https://docs.rs/threexui-rs)
[![CI](https://github.com/falhad/threexui-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/falhad/threexui-rs/actions)

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
| `client.xray()` | 9 | Xray config, outbound traffic, warp/nord |
| `client.custom_geo()` | 7 | Custom GeoIP/GeoSite resources |

## Configuration

```rust
let config = ClientConfig::builder()
    .host("example.com")
    .port(443)
    .base_path("/panel/")   // if panel is behind a subpath
    .tls(true)
    .accept_invalid_certs(false)
    .timeout_secs(30)
    .build()?;
```

## Two-factor authentication

```rust
client.login_2fa("admin", "password", "123456").await?;
```

## Crates.io publishing

Before publishing, update the repository URL in `Cargo.toml` to your actual GitHub username.

## License

MIT
