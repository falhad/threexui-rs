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
