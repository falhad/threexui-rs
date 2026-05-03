//! Live integration smoke-test — exercises every public API method against a running 3x-ui panel.
//!
//! Run: `cargo run --example live_test -- <host> <port> <user> <pass>`
//! Default: 127.0.0.1 8787 admin admin

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use threexui_rs::{Client, ClientConfig, Inbound, Protocol};

static PASS: AtomicUsize = AtomicUsize::new(0);
static FAIL: AtomicUsize = AtomicUsize::new(0);
static SKIP: AtomicUsize = AtomicUsize::new(0);

macro_rules! check {
    ($label:expr, $expr:expr) => {{
        let t = Instant::now();
        match $expr {
            Ok(v) => {
                println!("  PASS  {:<40}  ({:?})", $label, t.elapsed());
                PASS.fetch_add(1, Ordering::Relaxed);
                Some(v)
            }
            Err(e) => {
                println!("  FAIL  {:<40}  ({:?})  -> {}", $label, t.elapsed(), e);
                FAIL.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }};
}

macro_rules! check_unit {
    ($label:expr, $expr:expr) => {{
        let t = Instant::now();
        match $expr {
            Ok(()) => {
                println!("  PASS  {:<40}  ({:?})", $label, t.elapsed());
                PASS.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(e) => {
                println!("  FAIL  {:<40}  ({:?})  -> {}", $label, t.elapsed(), e);
                FAIL.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }};
}

#[tokio::main]
async fn main() -> threexui_rs::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let host = args.get(1).cloned().unwrap_or_else(|| "127.0.0.1".into());
    let port: u16 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8787);
    let user = args.get(3).cloned().unwrap_or_else(|| "admin".into());
    let pass = args.get(4).cloned().unwrap_or_else(|| "admin".into());

    println!("\n=========================================================");
    println!(" 3x-ui-rs LIVE TEST  -> http://{host}:{port}  ({user})");
    println!("=========================================================\n");

    let cfg = ClientConfig::builder()
        .host(host.clone())
        .port(port)
        .timeout_secs(20)
        .build()?;
    let client = Client::new(cfg);

    println!("[auth]");
    if !check_unit!("login", client.login(&user, &pass).await) {
        eprintln!("  Cannot proceed without auth.");
        return Ok(());
    }
    check!(
        "is_two_factor_enabled",
        client.is_two_factor_enabled().await
    );

    println!("\n[server]");
    let status = check!("server.status", client.server().status().await);
    if let Some(s) = &status {
        println!(
            "        cpu={:.1}% xray={} v={} uptime={}s",
            s.cpu, s.xray.state, s.xray.version, s.uptime
        );
    }
    check!(
        "server.cpu_history(60)",
        client.server().cpu_history(60).await
    );
    check!(
        "server.xray_versions",
        client.server().xray_versions().await
    );
    check!("server.config_json", client.server().config_json().await);
    let new_uuid = check!("server.new_uuid", client.server().new_uuid().await);
    check!(
        "server.new_x25519_cert",
        client.server().new_x25519_cert().await
    );
    check!("server.new_mldsa65", client.server().new_mldsa65().await);
    check!("server.new_mlkem768", client.server().new_mlkem768().await);
    check!(
        "server.new_vless_enc",
        client.server().new_vless_enc().await
    );
    check!(
        "server.new_ech_cert(google.com)",
        client.server().new_ech_cert("google.com").await
    );
    check!(
        "server.logs(50,info,)",
        client.server().logs(50, "info", "").await
    );
    check!(
        "server.xray_logs(50,'',f,f,f)",
        client.server().xray_logs(50, "", false, false, false).await
    );
    let db_bytes = check!("server.download_db", client.server().download_db().await);
    if let Some(b) = &db_bytes {
        println!("        db size = {} bytes", b.len());
    }

    println!("\n[settings]");
    let all = check!("settings.get_all", client.settings().get_all().await);
    if let Some(s) = &all {
        println!(
            "        web_port={} sub_enable={} tg_bot_enable={}",
            s.web_port, s.sub_enable, s.tg_bot_enable
        );
    }
    check!(
        "settings.get_defaults",
        client.settings().get_defaults().await
    );
    check!(
        "settings.default_xray_config",
        client.settings().default_xray_config().await
    );

    println!("\n[xray]");
    check!("xray.get_setting", client.xray().get_setting().await);
    check!("xray.default_config", client.xray().default_config().await);
    check!(
        "xray.outbounds_traffic",
        client.xray().outbounds_traffic().await
    );
    check!("xray.xray_result", client.xray().xray_result().await);
    check!(
        "xray.warp(Data)",
        client.xray().warp(threexui_rs::WarpAction::Data).await
    );
    check!(
        "xray.nord(Countries)",
        client.xray().nord(threexui_rs::NordAction::Countries).await
    );

    println!("\n[custom_geo]");
    check!("custom_geo.list", client.custom_geo().list().await);
    check!("custom_geo.aliases", client.custom_geo().aliases().await);

    println!("\n[inbounds — read]");
    let inbounds_before = check!("inbounds.list", client.inbounds().list().await);
    let initial_len = inbounds_before.as_ref().map(|v| v.len()).unwrap_or(0);
    println!("        existing inbounds = {}", initial_len);

    println!("\n[inbounds — create + lifecycle]");
    let (uuid, free_port) = (
        new_uuid
            .as_ref()
            .map(|u| u.uuid.clone())
            .unwrap_or_else(|| "fallback-uuid".into()),
        pick_free_port(initial_len),
    );

    let test_email = format!("test_{}@example.com", uuid.split('-').next().unwrap_or("e"));
    let remark = format!("threexui-rs-livetest-{}", free_port);

    let settings_str = serde_json::json!({
        "clients": [{
            "id": uuid,
            "email": test_email,
            "enable": true,
            "flow": "",
            "limitIp": 0,
            "totalGB": 0,
            "expiryTime": 0,
            "tgId": 0,
            "subId": "",
            "comment": "",
            "reset": 0
        }],
        "decryption": "none",
        "fallbacks": []
    })
    .to_string();
    let stream_str = serde_json::json!({
        "network": "tcp",
        "security": "none",
        "tcpSettings": {"header": {"type": "none"}}
    })
    .to_string();
    let sniffing_str = serde_json::json!({
        "enabled": false,
        "destOverride": ["http","tls","quic","fakedns"]
    })
    .to_string();

    let inb = Inbound {
        remark: remark.clone(),
        enable: true,
        listen: "".into(),
        port: free_port,
        protocol: Protocol::VLess,
        settings: serde_json::Value::String(settings_str),
        stream_settings: serde_json::Value::String(stream_str),
        sniffing: serde_json::Value::String(sniffing_str),
        tag: format!("inbound-{}", free_port),
        ..Default::default()
    };
    let added = check!("inbounds.add", client.inbounds().add(&inb).await);

    let inbound_id = added.as_ref().map(|i| i.id);
    if let Some(id) = inbound_id {
        println!("        new inbound id = {}", id);
        check!("inbounds.get", client.inbounds().get(id).await);

        // update remark
        if let Some(mut updated) = added.clone() {
            updated.remark = format!("{}-upd", remark);
            check!(
                "inbounds.update",
                client.inbounds().update(id, &updated).await
            );
        }

        // add second client
        let uuid2 = client
            .server()
            .new_uuid()
            .await
            .ok()
            .map(|u| u.uuid)
            .unwrap_or_default();
        let email2 = format!("second_{}@example.com", &uuid2[..8.min(uuid2.len())]);
        let new_client = serde_json::json!({
            "id": uuid2,
            "email": email2,
            "enable": true,
            "flow": "",
            "limitIp": 0,
            "totalGB": 0,
            "expiryTime": 0,
            "tgId": 0,
            "subId": "",
            "comment": "",
            "reset": 0
        });
        check_unit!(
            "inbounds.add_client",
            client
                .inbounds()
                .add_client(id, std::slice::from_ref(&new_client))
                .await
        );

        // update_client
        let mut updated_client = new_client.clone();
        updated_client["limitIp"] = serde_json::json!(2);
        check_unit!(
            "inbounds.update_client",
            client
                .inbounds()
                .update_client(&uuid2, id, &updated_client)
                .await
        );

        // client_traffics_by_email
        check!(
            "inbounds.client_traffics_by_email",
            client
                .inbounds()
                .client_traffics_by_email(&test_email)
                .await
        );
        check!(
            "inbounds.client_traffics_by_id",
            client.inbounds().client_traffics_by_id(&uuid).await
        );

        // client_ips
        check!(
            "inbounds.client_ips",
            client.inbounds().client_ips(&test_email).await
        );
        check_unit!(
            "inbounds.clear_client_ips",
            client.inbounds().clear_client_ips(&test_email).await
        );

        // online + last_online
        check!(
            "inbounds.online_clients",
            client.inbounds().online_clients().await
        );
        check!(
            "inbounds.last_online",
            client.inbounds().last_online().await
        );

        // update_client_traffic
        check_unit!(
            "inbounds.update_client_traffic",
            client
                .inbounds()
                .update_client_traffic(&test_email, 1024, 2048)
                .await
        );

        // reset_client_traffic
        check_unit!(
            "inbounds.reset_client_traffic",
            client
                .inbounds()
                .reset_client_traffic(id, &test_email)
                .await
        );

        // reset_all_client_traffics
        check_unit!(
            "inbounds.reset_all_client_traffics",
            client.inbounds().reset_all_client_traffics(id).await
        );

        // delete_depleted_clients
        check_unit!(
            "inbounds.delete_depleted_clients",
            client.inbounds().delete_depleted_clients(id).await
        );

        // add a 3rd client so we can delete one without leaving the inbound empty
        let uuid3 = client
            .server()
            .new_uuid()
            .await
            .ok()
            .map(|u| u.uuid)
            .unwrap_or_default();
        let email3 = format!("third_{}@example.com", &uuid3[..8.min(uuid3.len())]);
        let third = serde_json::json!({
            "id": uuid3, "email": email3, "enable": true, "flow": "",
            "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
            "subId": "", "comment": "", "reset": 0
        });
        let _ = client.inbounds().add_client(id, &[third]).await;

        // delete_client (by uuid) — first client
        check_unit!(
            "inbounds.delete_client",
            client.inbounds().delete_client(id, &uuid).await
        );
        // delete_client_by_email — second
        check_unit!(
            "inbounds.delete_client_by_email",
            client.inbounds().delete_client_by_email(id, &email2).await
        );

        // import — fresh uuid + email + port
        let new_uuid_imp = client
            .server()
            .new_uuid()
            .await
            .ok()
            .map(|u| u.uuid)
            .unwrap_or_default();
        let imp_email = format!(
            "import_{}@example.com",
            &new_uuid_imp[..8.min(new_uuid_imp.len())]
        );
        let imp_settings = serde_json::json!({
            "clients": [{
                "id": new_uuid_imp, "email": imp_email, "enable": true, "flow": "",
                "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
                "subId": "", "comment": "", "reset": 0
            }],
            "decryption": "none", "fallbacks": []
        })
        .to_string();
        let mut imp = inb.clone();
        imp.port = free_port + 1;
        imp.tag = format!("inbound-{}", imp.port);
        imp.remark = format!("{}-import", remark);
        imp.settings = serde_json::Value::String(imp_settings);
        let imported = check!("inbounds.import", client.inbounds().import(&imp).await);
        if let Some(i2) = imported {
            check_unit!(
                "inbounds.delete(imported)",
                client.inbounds().delete(i2.id).await
            );
        }

        // copy_clients — create a sibling inbound, copy email3 over, delete sibling
        let mut sibling = inb.clone();
        sibling.port = free_port + 2;
        sibling.tag = format!("inbound-{}", sibling.port);
        sibling.remark = format!("{}-sibling", remark);
        // give sibling a unique uuid+email so copy_clients has a clean target
        let sib_uuid = client
            .server()
            .new_uuid()
            .await
            .ok()
            .map(|u| u.uuid)
            .unwrap_or_default();
        let sib_email = format!("sib_{}@example.com", &sib_uuid[..8.min(sib_uuid.len())]);
        let sib_settings = serde_json::json!({
            "clients": [{
                "id": sib_uuid, "email": sib_email, "enable": true, "flow": "",
                "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
                "subId": "", "comment": "", "reset": 0
            }],
            "decryption": "none", "fallbacks": []
        })
        .to_string();
        sibling.settings = serde_json::Value::String(sib_settings);
        if let Ok(sib) = client.inbounds().add(&sibling).await {
            check!(
                "inbounds.copy_clients(diff)",
                client
                    .inbounds()
                    .copy_clients(id, sib.id, std::slice::from_ref(&email3), "")
                    .await
            );
            let _ = client.inbounds().delete(sib.id).await;
        }

        // reset_all_traffics
        check_unit!(
            "inbounds.reset_all_traffics",
            client.inbounds().reset_all_traffics().await
        );

        // delete the test inbound at end
        check_unit!("inbounds.delete", client.inbounds().delete(id).await);
    }

    println!("\n[misc]");
    check_unit!(
        "backup_to_tgbot (best-effort)",
        client.backup_to_tgbot().await
    );

    println!("\n[teardown]");
    check_unit!("logout", client.logout().await);

    let p = PASS.load(Ordering::Relaxed);
    let f = FAIL.load(Ordering::Relaxed);
    let s = SKIP.load(Ordering::Relaxed);
    println!(
        "\n----- RESULT  pass={}  fail={}  skip={}  total={} -----\n",
        p,
        f,
        s,
        p + f + s
    );

    Ok(())
}

fn pick_free_port(seed: usize) -> u16 {
    // pick a high port unlikely to collide; vary slightly with seed
    30000_u16.saturating_add((seed as u16) % 1000)
}
