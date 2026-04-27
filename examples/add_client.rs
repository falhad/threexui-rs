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
