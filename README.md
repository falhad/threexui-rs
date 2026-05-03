# threexui-rs

An async Rust SDK for the [3x-ui](https://github.com/MHSanaei/3x-ui) panel API. Covers all API endpoints across inbounds, server management, settings, Xray configuration, and custom geo resources.

[![Crates.io](https://img.shields.io/crates/v/threexui-rs)](https://crates.io/crates/threexui-rs)
[![docs.rs](https://docs.rs/threexui-rs/badge.svg)](https://docs.rs/threexui-rs)
[![CI](https://github.com/falhad/threexui-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/falhad/threexui-rs/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Version compatibility

| threexui-rs   | 3x-ui panel(s)         |
|---------------|------------------------|
| 2.9.5         | v2.9.2, v2.9.3         |
| 2.9.4         | v2.9.2, v2.9.3 (yanked: aws-lc-sys CI build break) |
| 2.9.3         | v2.9.3                 |

`2.9.4` is live-tested against both panel versions. The only behavioral
difference: `inbounds.copy_clients` does not exist in v2.9.2 and returns
`Error::EndpointNotFound` — match on it to fall back gracefully.

## Installation

```toml
[dependencies]
threexui-rs = "2.9.5"
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
    println!("Server CPU: {:.1}%  Xray: {}", status.cpu, status.xray.state);

    client.logout().await?;
    Ok(())
}
```

## Configuration

```rust
let config = ClientConfig::builder()
    .host("example.com")
    .port(443)
    .base_path("/panel/")       // optional subpath prefix
    .tls(true)                  // use HTTPS
    .accept_invalid_certs(true) // for self-signed certs
    .timeout_secs(30)
    .build()?;
```

### Outbound proxy (optional)

Route every request through an HTTP, HTTPS, or SOCKS5 proxy:

```rust
let config = ClientConfig::builder()
    .host("panel.example.com").port(2053)
    .proxy("socks5h://127.0.0.1:1080")   // also: http://, https://, socks5://
    .proxy_auth("username", "password")  // optional basic-auth
    .build()?;
```

- `socks5://` resolves DNS locally, `socks5h://` resolves it through the proxy.
- Bad proxy URLs surface as `Error::Config` at `build()` time.
- Use `.no_proxy()` on the builder to clear a previously set proxy.

## Authentication

Standard login:

```rust
client.login("admin", "password").await?;
```

With two-factor authentication:

```rust
client.login_2fa("admin", "password", "123456").await?;
```

Check if 2FA is enabled on the panel:

```rust
let enabled = client.is_two_factor_enabled().await?;
```

## API reference

All methods are async and return `threexui_rs::Result<T>`. The client is cheap to clone (`Arc`-backed) and safe to share across tasks.

### Inbounds — `client.inbounds()`

| Method | Description |
|--------|-------------|
| `list()` | List all inbounds |
| `get(id)` | Get a single inbound by ID |
| `add(inbound)` | Create a new inbound |
| `update(id, inbound)` | Update an existing inbound |
| `delete(id)` | Delete an inbound |
| `import(inbound)` | Import an inbound from another panel |
| `add_client(inbound_id, clients)` | Add clients to an inbound |
| `update_client(client_id, inbound_id, client)` | Update a client |
| `delete_client(inbound_id, client_id)` | Delete a client by ID |
| `delete_client_by_email(inbound_id, email)` | Delete a client by email |
| `copy_clients(target_id, source_id, emails, flow)` | Copy clients between inbounds |
| `client_ips(email)` | Get IP list for a client |
| `clear_client_ips(email)` | Clear recorded IPs for a client |
| `client_traffics_by_email(email)` | Get traffic stats by email |
| `client_traffics_by_id(id)` | Get traffic stats by client ID |
| `reset_client_traffic(inbound_id, email)` | Reset traffic for a client |
| `reset_all_traffics()` | Reset traffic for all inbounds |
| `reset_all_client_traffics(inbound_id)` | Reset all client traffic in an inbound |
| `delete_depleted_clients(inbound_id)` | Remove clients that have exhausted their quota |
| `online_clients()` | List currently online client emails |
| `last_online()` | Map of email → last-seen timestamp |
| `update_client_traffic(email, upload, download)` | Manually adjust client traffic counters |

### Server — `client.server()`

| Method | Description |
|--------|-------------|
| `status()` | CPU, memory, disk, Xray state, network stats |
| `cpu_history(bucket)` | Historical CPU usage |
| `xray_versions()` | Available Xray versions to install |
| `config_json()` | Current Xray config as JSON |
| `download_db()` | Download the panel database as bytes |
| `import_db(data)` | Restore a panel database |
| `new_uuid()` | Generate a new UUID |
| `new_x25519_cert()` | Generate an X25519 keypair |
| `new_mldsa65()` | Generate ML-DSA-65 keys |
| `new_mlkem768()` | Generate ML-KEM-768 keys |
| `new_ech_cert(sni)` | Generate an ECH certificate |
| `new_vless_enc()` | Generate VLESS encryption keys |
| `stop_xray()` | Stop the Xray service |
| `restart_xray()` | Restart the Xray service |
| `install_xray(version)` | Install a specific Xray version |
| `update_geofile(filename)` | Update geo data files |
| `logs(count, level, syslog)` | Retrieve panel log lines |
| `xray_logs(count, filter, ...)` | Retrieve Xray log lines |

### Settings — `client.settings()`

| Method | Description |
|--------|-------------|
| `get_all()` | Get all panel settings |
| `get_defaults()` | Get default settings values |
| `update(settings)` | Save updated settings |
| `update_user(old_user, old_pass, new_user, new_pass)` | Change admin credentials |
| `restart_panel()` | Restart the panel process |
| `default_xray_config()` | Get the default Xray JSON config |

### Xray — `client.xray()`

| Method | Description |
|--------|-------------|
| `get_setting()` | Get current Xray settings |
| `update_setting(config, test_url)` | Update Xray settings |
| `default_config()` | Get default Xray config |
| `outbounds_traffic()` | Get per-outbound traffic stats |
| `reset_outbound_traffic(tag)` | Reset traffic counter for an outbound |
| `test_outbound(outbound, all_outbounds)` | Test an outbound configuration |
| `xray_result()` | Get latest Xray test result |
| `warp(action)` | Manage Cloudflare WARP integration |
| `nord(action)` | Manage NordVPN integration |

### Custom Geo — `client.custom_geo()`

| Method | Description |
|--------|-------------|
| `list()` | List all custom geo resources |
| `aliases()` | List resource aliases |
| `add(geo)` | Add a new geo resource |
| `update(id, geo)` | Update an existing geo resource |
| `delete(id)` | Delete a geo resource |
| `download(id)` | Trigger download for a geo resource |
| `update_all()` | Update all geo resources |

## Error handling

All fallible operations return `threexui_rs::Result<T>`, which is `Result<T, threexui_rs::Error>`:

```rust
use threexui_rs::Error;

match client.inbounds().list().await {
    Ok(inbounds) => { /* ... */ }
    Err(Error::NotAuthenticated) => eprintln!("call login() first"),
    Err(Error::Auth(msg)) => eprintln!("login failed: {}", msg),
    Err(Error::Api(msg)) => eprintln!("panel returned error: {}", msg),
    Err(Error::EndpointNotFound(path)) => eprintln!("endpoint missing on this panel: {}", path),
    Err(Error::Http(e)) => eprintln!("http error: {}", e),
    Err(e) => eprintln!("other error: {}", e),
}
```

`Error::EndpointNotFound` is returned when the panel responds with HTTP 404.
This typically means the panel is older than the lib (e.g. calling
`inbounds.copy_clients` on a v2.9.2 panel). Match on it to fall back.

## Examples

See the [`examples/`](examples/) directory:

- [`list_inbounds.rs`](examples/list_inbounds.rs) — list all inbounds and print server status.
- [`add_client.rs`](examples/add_client.rs) — generate a UUID and add a new VLESS client.
- [`live_test.rs`](examples/live_test.rs) — full smoke test of every public API method.
- [`scenarios.rs`](examples/scenarios.rs) — 16 production scenarios (multi-protocol create, renew, disable/enable, reset-uuid, data-limit, traffic resets, concurrent reads, special characters, negative paths…).
- [`proxy_test.rs`](examples/proxy_test.rs) — end-to-end verification of HTTP / SOCKS5 proxy support.

Run an example (requires a live panel):

```bash
cargo run --example list_inbounds
cargo run --example scenarios -- 127.0.0.1 2053 admin admin
cargo run --example proxy_test -- panel.example.com 2053 admin pw "socks5h://127.0.0.1:1080"
```

## License

MIT — see [LICENSE](LICENSE)
