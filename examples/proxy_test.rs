//! Live proxy verification — proves the lib actually routes through HTTP / SOCKS5 proxies.
//!
//! Run: `cargo run --example proxy_test -- <panel-host> <panel-port> <user> <pass> <proxy-url>`
//!
//! Proxy-url examples:
//!   http://127.0.0.1:18888
//!   socks5://127.0.0.1:11080
//!   socks5://user:pass@127.0.0.1:11080
//!
//! Tip on macOS: panels reachable on host's `127.0.0.1` are *not* reachable from
//! within proxy containers. Use `host.docker.internal` as the panel host
//! when the proxy is running in Docker.

use threexui_rs::{Client, ClientConfig, Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let host = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "host.docker.internal".into());
    let port: u16 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8787);
    let user = args.get(3).cloned().unwrap_or_else(|| "admin".into());
    let pass = args.get(4).cloned().unwrap_or_else(|| "admin".into());
    let proxy = args
        .get(5)
        .cloned()
        .unwrap_or_else(|| "http://127.0.0.1:18888".into());

    println!("\n========================================================");
    println!(" PROXY TEST  panel=http://{host}:{port}  via {proxy}");
    println!("========================================================\n");

    // 1. Bad-URL validation surfaces as Config error
    let bad = ClientConfig::builder()
        .host(host.clone())
        .port(port)
        .proxy("not a url")
        .build();
    match bad {
        Err(Error::Config(msg)) if msg.contains("invalid proxy url") => {
            println!("  ✓ invalid-url validation: {msg}")
        }
        other => {
            println!(
                "  ✗ invalid-url validation expected Config error, got {:?}",
                other.is_ok()
            );
            std::process::exit(1);
        }
    }

    // 2. Unreachable proxy → connection error (proves proxy is being used)
    let dead = Client::new(
        ClientConfig::builder()
            .host(host.clone())
            .port(port)
            .timeout_secs(3)
            .proxy("http://127.0.0.1:1") // nothing listens on :1
            .build()?,
    );
    match dead.login(&user, &pass).await {
        Err(Error::Http(_)) => println!("  ✓ unreachable proxy → Http error (proxy is in path)"),
        other => {
            println!(
                "  ✗ expected Http error from dead proxy, got {:?}",
                other.is_ok()
            );
            std::process::exit(1);
        }
    }

    // 3. Working proxy → end-to-end calls succeed
    let cfg = ClientConfig::builder()
        .host(host.clone())
        .port(port)
        .timeout_secs(15)
        .proxy(&proxy)
        .build()?;
    let client = Client::new(cfg);

    client.login(&user, &pass).await?;
    println!("  ✓ login via proxy");

    let status = client.server().status().await?;
    println!(
        "  ✓ server.status via proxy (cpu={:.1}% xray={})",
        status.cpu, status.xray.state
    );

    let inbounds = client.inbounds().list().await?;
    println!("  ✓ inbounds.list via proxy ({} inbounds)", inbounds.len());

    let uuid = client.server().new_uuid().await?;
    println!("  ✓ server.new_uuid via proxy ({})", uuid.uuid);

    client.logout().await?;
    println!("  ✓ logout via proxy");

    println!("\n  ALL PROXY CHECKS PASSED\n");
    Ok(())
}
