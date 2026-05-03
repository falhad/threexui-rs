//! Production scenarios — exercises real-world flows end-to-end against a live 3x-ui panel.
//!
//! Run: `cargo run --example scenarios -- <host> <port> <user> <pass>`
//!
//! Scenarios covered:
//!   1.  Auth: bad password rejected, good password accepted, double-login safe
//!   2.  Multi-protocol create (vless / vmess / trojan / shadowsocks)
//!   3.  find_client_by_uuid — the production bug pattern (decode after .get())
//!   4.  Bulk create N clients in one inbound
//!   5.  Renew client (extend expiryTime)
//!   6.  Disable + re-enable client
//!   7.  Reset uuid (replace client uuid, keep email)
//!   8.  Data-limit increase
//!   9.  Reset traffic for one client + for whole inbound
//!  10.  Delete depleted clients
//!  11.  Update inbound remark/port
//!  12.  Concurrent reads (parallel list/get/status)
//!  13.  Special characters in email + comment
//!  14.  Client search across all inbounds
//!  15.  Negative-path: get non-existent inbound, delete twice, add duplicate email
//!  16.  Cleanup verification (no leftover inbounds with our prefix)

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use threexui_rs::{Client, ClientConfig, Error, Inbound, InboundClient, Protocol};

const PREFIX: &str = "rs-scenario-";

static PASS: AtomicUsize = AtomicUsize::new(0);
static FAIL: AtomicUsize = AtomicUsize::new(0);

macro_rules! step {
    ($name:expr) => {
        println!("\n--- {} ---", $name);
    };
}
macro_rules! ok {
    ($label:expr) => {{
        println!("  ✓ {}", $label);
        PASS.fetch_add(1, Ordering::Relaxed);
    }};
}
macro_rules! err {
    ($label:expr, $e:expr) => {{
        println!("  ✗ {}: {}", $label, $e);
        FAIL.fetch_add(1, Ordering::Relaxed);
    }};
}
macro_rules! must {
    ($label:expr, $expr:expr) => {{
        match $expr {
            Ok(v) => {
                ok!($label);
                Some(v)
            }
            Err(e) => {
                err!($label, e);
                None
            }
        }
    }};
}
macro_rules! must_unit {
    ($label:expr, $expr:expr) => {{
        match $expr {
            Ok(()) => {
                ok!($label);
                true
            }
            Err(e) => {
                err!($label, e);
                false
            }
        }
    }};
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn build_inbound(
    port: u16,
    protocol: Protocol,
    remark: &str,
    settings_clients: serde_json::Value,
) -> Inbound {
    let settings_obj = match protocol {
        Protocol::VLess | Protocol::VMess => {
            serde_json::json!({"clients": settings_clients, "decryption":"none","fallbacks":[]})
        }
        Protocol::Trojan => {
            serde_json::json!({"clients": settings_clients, "fallbacks":[]})
        }
        Protocol::Shadowsocks => {
            // shadowsocks needs method + password at top-level — keep simple
            serde_json::json!({
                "method":"chacha20-ietf-poly1305",
                "password":"abcd1234efgh",
                "network":"tcp,udp",
                "clients": settings_clients
            })
        }
        _ => serde_json::json!({"clients": settings_clients}),
    };
    let stream = serde_json::json!({
        "network":"tcp","security":"none",
        "tcpSettings":{"header":{"type":"none"}}
    });
    Inbound {
        remark: remark.into(),
        enable: true,
        listen: "".into(),
        port,
        protocol,
        settings: serde_json::Value::String(settings_obj.to_string()),
        stream_settings: serde_json::Value::String(stream.to_string()),
        sniffing: serde_json::Value::String(
            serde_json::json!({"enabled":false,"destOverride":["http","tls"]}).to_string(),
        ),
        tag: format!("inbound-{}", port),
        ..Default::default()
    }
}

fn make_client(uuid: &str, email: &str) -> serde_json::Value {
    serde_json::json!({
        "id": uuid, "email": email, "enable": true, "flow": "",
        "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
        "subId": "", "comment": "", "reset": 0
    })
}

async fn cleanup(client: &Client) {
    if let Ok(list) = client.inbounds().list().await {
        for inb in list.iter().filter(|i| i.remark.starts_with(PREFIX)) {
            let _ = client.inbounds().delete(inb.id).await;
        }
    }
}

#[tokio::main]
async fn main() -> threexui_rs::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let host = args.get(1).cloned().unwrap_or_else(|| "127.0.0.1".into());
    let port: u16 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8787);
    let user = args.get(3).cloned().unwrap_or_else(|| "admin".into());
    let pass = args.get(4).cloned().unwrap_or_else(|| "admin".into());

    println!("=========================================================");
    println!(" SCENARIO TEST  -> http://{host}:{port}  ({user})");
    println!("=========================================================");

    // ------------------------------------------------------------------
    step!("1. Auth flows");
    let cfg = ClientConfig::builder()
        .host(host.clone())
        .port(port)
        .timeout_secs(15)
        .build()?;

    // Bad password → must fail
    let bad = Client::new(cfg.clone());
    match bad.login(&user, "definitely-not-the-password-zzz").await {
        Err(Error::Auth(_)) => ok!("login(wrong) returns Auth error"),
        Err(e) => err!("login(wrong) wrong error variant", e),
        Ok(_) => err!("login(wrong) unexpectedly succeeded", "should have failed"),
    }
    // Calling protected endpoint after auth failure → NotAuthenticated
    match bad.inbounds().list().await {
        Err(Error::NotAuthenticated) => ok!("post-failed-login returns NotAuthenticated"),
        other => err!(
            "post-failed-login wrong result",
            format!("{:?}", other.is_ok())
        ),
    }

    let client = Client::new(cfg);
    must_unit!("login(correct)", client.login(&user, &pass).await);
    // Double-login should still leave session valid
    must_unit!("login(again, idempotent)", client.login(&user, &pass).await);

    // Pre-cleanup any leftovers from a prior aborted run
    cleanup(&client).await;

    // ------------------------------------------------------------------
    step!("2. Multi-protocol inbound creation");
    let base_port: u16 = 31000 + ((now_ms() as u64 % 2000) as u16);
    let protos: &[(Protocol, &str)] = &[
        (Protocol::VLess, "vless"),
        (Protocol::VMess, "vmess"),
        (Protocol::Trojan, "trojan"),
        (Protocol::Shadowsocks, "ss"),
    ];

    let mut created: Vec<(i64, Protocol)> = Vec::new();
    for (i, (proto, label)) in protos.iter().enumerate() {
        let p = base_port + i as u16;
        let uuid = client.server().new_uuid().await?.uuid;
        let email = format!("scn-{}-{}", label, &uuid[..6]);
        let pwd_field = if matches!(proto, Protocol::Trojan) {
            // trojan client needs `password`
            serde_json::json!([{
                "id": "", "email": email, "enable": true, "flow": "",
                "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
                "subId": "", "comment": "", "reset": 0,
                "password": uuid.clone()
            }])
        } else if matches!(proto, Protocol::Shadowsocks) {
            // ss client form
            serde_json::json!([{
                "method":"chacha20-ietf-poly1305", "password": uuid.clone(),
                "email": email, "limitIp":0, "totalGB":0, "expiryTime":0,
                "subId":"", "comment":"", "reset":0
            }])
        } else {
            serde_json::json!([make_client(&uuid, &email)])
        };
        let inb = build_inbound(
            p,
            proto.clone(),
            &format!("{}{}-{}", PREFIX, label, p),
            pwd_field,
        );
        match client.inbounds().add(&inb).await {
            Ok(added) => {
                ok!(format!("create {} on :{}", label, p));
                created.push((added.id, proto.clone()));
            }
            Err(e) => err!(format!("create {} on :{}", label, p), e),
        }
    }

    // ------------------------------------------------------------------
    step!("3. find_client_by_uuid (the production bug pattern)");
    // For each created inbound, .get() it then parse settings.clients into Vec<InboundClient>
    // and confirm we can find each created client by uuid.
    let mut sample_uuid: Option<String> = None;
    let mut sample_email: Option<String> = None;
    let mut sample_inbound_id: Option<i64> = None;
    for (id, proto) in &created {
        match client.inbounds().get(*id).await {
            Ok(inb) => {
                ok!(format!("inbounds.get({}) decoded ({:?})", id, proto));
                if matches!(proto, Protocol::VLess | Protocol::VMess) {
                    let settings_str = match &inb.settings {
                        serde_json::Value::String(s) => s.clone(),
                        v => v.to_string(),
                    };
                    let parsed: serde_json::Value =
                        serde_json::from_str(&settings_str).unwrap_or(serde_json::json!({}));
                    let arr = parsed.get("clients").cloned().unwrap_or_default();
                    match serde_json::from_value::<Vec<InboundClient>>(arr) {
                        Ok(cs) => {
                            ok!(format!("  parse {} clients via InboundClient", cs.len()));
                            if let Some(c) = cs.first() {
                                sample_uuid = Some(c.id.clone());
                                sample_email = Some(c.email.clone());
                                sample_inbound_id = Some(*id);
                            }
                        }
                        Err(e) => err!("  Vec<InboundClient> parse", e),
                    }
                }
            }
            Err(e) => err!(format!("inbounds.get({})", id), e),
        }
    }

    // ------------------------------------------------------------------
    step!("4. Bulk create N clients in one inbound");
    let bulk_inbound_id = created
        .iter()
        .find(|(_, p)| matches!(p, Protocol::VLess))
        .map(|(i, _)| *i);
    if let Some(id) = bulk_inbound_id {
        let n = 10;
        let mut emails: Vec<String> = Vec::with_capacity(n);
        for i in 0..n {
            let uuid = client.server().new_uuid().await?.uuid;
            let email = format!("bulk-{:02}-{}", i, &uuid[..6]);
            emails.push(email.clone());
            if let Err(e) = client
                .inbounds()
                .add_client(id, &[make_client(&uuid, &email)])
                .await
            {
                err!(format!("bulk add #{}", i), e);
            }
        }
        ok!(format!("bulk added {} clients", n));

        // Verify they all show up
        match client.inbounds().get(id).await {
            Ok(inb) => {
                let s = inb.settings.as_str().map(String::from).unwrap_or_default();
                let parsed: serde_json::Value = serde_json::from_str(&s).unwrap_or_default();
                let cs = parsed.get("clients").cloned().unwrap_or_default();
                let cs: Vec<InboundClient> = serde_json::from_value(cs).unwrap_or_default();
                let found = emails
                    .iter()
                    .filter(|e| cs.iter().any(|c| &c.email == *e))
                    .count();
                if found == n {
                    ok!(format!("verified {} bulk clients present", found));
                } else {
                    err!(
                        "bulk verification",
                        format!("only {} / {} present", found, n)
                    );
                }
            }
            Err(e) => err!("bulk verify .get", e),
        }
    }

    // ------------------------------------------------------------------
    step!("5. Renew client (extend expiryTime)");
    if let (Some(uuid), Some(_email), Some(iid)) = (&sample_uuid, &sample_email, sample_inbound_id)
    {
        let new_expiry = now_ms() + 30 * 24 * 60 * 60 * 1000; // +30 days
        let updated = serde_json::json!({
            "id": uuid, "email": sample_email.as_ref().unwrap(),
            "enable": true, "flow":"",
            "limitIp": 0, "totalGB": 0, "expiryTime": new_expiry,
            "tgId": 0, "subId":"", "comment":"renewed", "reset": 0
        });
        must_unit!(
            "update_client (extend expiry)",
            client.inbounds().update_client(uuid, iid, &updated).await
        );

        // Verify
        if let Ok(inb) = client.inbounds().get(iid).await {
            let s = inb.settings.as_str().map(String::from).unwrap_or_default();
            let parsed: serde_json::Value = serde_json::from_str(&s).unwrap_or_default();
            let cs: Vec<InboundClient> =
                serde_json::from_value(parsed.get("clients").cloned().unwrap_or_default())
                    .unwrap_or_default();
            if let Some(c) = cs.iter().find(|c| &c.id == uuid) {
                if c.expiry_time == new_expiry && c.comment == "renewed" {
                    ok!("renew verified in settings");
                } else {
                    err!(
                        "renew verify",
                        format!(
                            "expected expiry={}, got={}, comment={}",
                            new_expiry, c.expiry_time, c.comment
                        )
                    );
                }
            } else {
                err!("renew verify", "client uuid not found post-update");
            }
        }
    }

    // ------------------------------------------------------------------
    step!("6. Disable + re-enable client");
    if let (Some(uuid), Some(email), Some(iid)) = (&sample_uuid, &sample_email, sample_inbound_id) {
        let disabled = serde_json::json!({
            "id": uuid, "email": email, "enable": false, "flow":"",
            "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
            "subId":"", "comment":"disabled", "reset": 0
        });
        must_unit!(
            "disable client",
            client.inbounds().update_client(uuid, iid, &disabled).await
        );
        let enabled = serde_json::json!({
            "id": uuid, "email": email, "enable": true, "flow":"",
            "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
            "subId":"", "comment":"enabled-again", "reset": 0
        });
        must_unit!(
            "re-enable client",
            client.inbounds().update_client(uuid, iid, &enabled).await
        );
    }

    // ------------------------------------------------------------------
    step!("7. Reset uuid (replace client uuid, keep email)");
    if let (Some(old_uuid), Some(email), Some(iid)) =
        (sample_uuid.clone(), sample_email.clone(), sample_inbound_id)
    {
        let new_uuid = client.server().new_uuid().await?.uuid;
        let replaced = serde_json::json!({
            "id": new_uuid, "email": email, "enable": true, "flow":"",
            "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
            "subId":"", "comment":"reset-uuid", "reset": 0
        });
        // Pass *old* uuid as path param (3x-ui matches existing client by it)
        must_unit!(
            "update_client (reset uuid)",
            client
                .inbounds()
                .update_client(&old_uuid, iid, &replaced)
                .await
        );

        // Verify new uuid is present, old gone
        if let Ok(inb) = client.inbounds().get(iid).await {
            let s = inb.settings.as_str().map(String::from).unwrap_or_default();
            let parsed: serde_json::Value = serde_json::from_str(&s).unwrap_or_default();
            let cs: Vec<InboundClient> =
                serde_json::from_value(parsed.get("clients").cloned().unwrap_or_default())
                    .unwrap_or_default();
            let has_new = cs.iter().any(|c| c.id == new_uuid);
            let has_old = cs.iter().any(|c| c.id == old_uuid);
            if has_new && !has_old {
                ok!("uuid replaced in settings");
            } else {
                err!(
                    "uuid reset verify",
                    format!("has_new={}, has_old={}", has_new, has_old)
                );
            }
            // Update sample_uuid to the new one for downstream steps
            sample_uuid = Some(new_uuid);
        }
    }

    // ------------------------------------------------------------------
    step!("8. Data-limit increase");
    if let (Some(uuid), Some(email), Some(iid)) = (&sample_uuid, &sample_email, sample_inbound_id) {
        let new_total = 5_368_709_120i64; // 5 GiB
        let upd = serde_json::json!({
            "id": uuid, "email": email, "enable": true, "flow":"",
            "limitIp": 0, "totalGB": new_total, "expiryTime": 0, "tgId": 0,
            "subId":"", "comment":"5gb", "reset": 0
        });
        must_unit!(
            "update totalGB",
            client.inbounds().update_client(uuid, iid, &upd).await
        );
        if let Ok(inb) = client.inbounds().get(iid).await {
            let s = inb.settings.as_str().map(String::from).unwrap_or_default();
            let parsed: serde_json::Value = serde_json::from_str(&s).unwrap_or_default();
            let cs: Vec<InboundClient> =
                serde_json::from_value(parsed.get("clients").cloned().unwrap_or_default())
                    .unwrap_or_default();
            if let Some(c) = cs.iter().find(|c| &c.id == uuid) {
                if c.total_gb == new_total {
                    ok!("totalGB persisted");
                } else {
                    err!(
                        "totalGB verify",
                        format!("got {}, want {}", c.total_gb, new_total)
                    );
                }
            }
        }
    }

    // ------------------------------------------------------------------
    step!("9. Reset traffic (per-client + whole inbound)");
    if let (Some(email), Some(iid)) = (&sample_email, sample_inbound_id) {
        // First push some fake traffic
        must_unit!(
            "update_client_traffic (set up/down)",
            client
                .inbounds()
                .update_client_traffic(email, 1024 * 1024, 2 * 1024 * 1024)
                .await
        );
        must_unit!(
            "reset_client_traffic",
            client.inbounds().reset_client_traffic(iid, email).await
        );
        must_unit!(
            "reset_all_client_traffics",
            client.inbounds().reset_all_client_traffics(iid).await
        );
    }

    // ------------------------------------------------------------------
    step!("10. delete_depleted_clients");
    if let Some(iid) = sample_inbound_id {
        must_unit!(
            "delete_depleted_clients",
            client.inbounds().delete_depleted_clients(iid).await
        );
    }

    // ------------------------------------------------------------------
    step!("11. Update inbound (remark + port)");
    if let Some(iid) = sample_inbound_id {
        if let Ok(mut inb) = client.inbounds().get(iid).await {
            let new_port = inb.port + 100;
            inb.remark = format!("{}{}", PREFIX, "renamed");
            inb.port = new_port;
            inb.tag = format!("inbound-{}", new_port);
            must!(
                "inbounds.update (remark+port)",
                client.inbounds().update(iid, &inb).await
            );
            if let Ok(reread) = client.inbounds().get(iid).await {
                if reread.port == new_port && reread.remark.contains("renamed") {
                    ok!("update persisted");
                } else {
                    err!(
                        "update verify",
                        format!("port={} remark={}", reread.port, reread.remark)
                    );
                }
            }
        }
    }

    // ------------------------------------------------------------------
    step!("12. Concurrent reads (parallel list/get/status)");
    let c1 = client.clone();
    let c2 = client.clone();
    let c3 = client.clone();
    let id_for_get = sample_inbound_id.unwrap_or(1);
    let f1 = async move { c1.inbounds().list().await };
    let f2 = async move { c2.server().status().await };
    let f3 = async move { c3.inbounds().get(id_for_get).await };
    let (a, b, c) = tokio::join!(f1, f2, f3);
    if a.is_ok() && b.is_ok() && c.is_ok() {
        ok!("3 parallel reads all succeeded");
    } else {
        err!(
            "parallel reads",
            format!("list={} status={} get={}", a.is_ok(), b.is_ok(), c.is_ok())
        );
    }

    // ------------------------------------------------------------------
    step!("13. Special-character email + comment");
    if let Some(iid) = sample_inbound_id {
        let uuid = client.server().new_uuid().await?.uuid;
        // 3x-ui email field is fairly permissive; test common edge chars.
        let weird_email = format!("test+tag.{}@sub.example.co", &uuid[..6]);
        let comment = "VIP — café 🚀 \"quoted\" \\back/slash";
        let payload = serde_json::json!({
            "id": uuid, "email": weird_email, "enable": true, "flow": "",
            "limitIp": 0, "totalGB": 0, "expiryTime": 0, "tgId": 0,
            "subId": "", "comment": comment, "reset": 0
        });
        if must_unit!(
            "add client (special chars)",
            client.inbounds().add_client(iid, &[payload]).await
        ) {
            if let Ok(inb) = client.inbounds().get(iid).await {
                let s = inb.settings.as_str().map(String::from).unwrap_or_default();
                let parsed: serde_json::Value = serde_json::from_str(&s).unwrap_or_default();
                let cs: Vec<InboundClient> =
                    serde_json::from_value(parsed.get("clients").cloned().unwrap_or_default())
                        .unwrap_or_default();
                if cs
                    .iter()
                    .any(|c| c.email == weird_email && c.comment == comment)
                {
                    ok!("special chars preserved round-trip");
                } else {
                    err!("special chars verify", "not preserved");
                }
            }
        }
    }

    // ------------------------------------------------------------------
    step!("14. Cross-inbound client search (find by uuid scan)");
    if let Some(target_uuid) = sample_uuid.clone() {
        let list = client.inbounds().list().await.unwrap_or_default();
        let mut found_in: Option<i64> = None;
        for inb in &list {
            let s = inb.settings.as_str().map(String::from).unwrap_or_default();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Ok(cs) = serde_json::from_value::<Vec<InboundClient>>(
                    parsed.get("clients").cloned().unwrap_or_default(),
                ) {
                    if cs.iter().any(|c| c.id == target_uuid) {
                        found_in = Some(inb.id);
                        break;
                    }
                }
            }
        }
        match found_in {
            Some(id) => ok!(format!("found uuid in inbound {}", id)),
            None => err!("uuid scan", "uuid not found in any inbound"),
        }
    }

    // ------------------------------------------------------------------
    step!("15. Negative paths");
    // 15a. get non-existent inbound
    match client.inbounds().get(999_999).await {
        Err(_) => ok!("get(999_999) returns error"),
        Ok(_) => err!("get(999_999) should error", "got Ok"),
    }
    // 15b. delete the same inbound twice
    if let Some(iid) = sample_inbound_id {
        let _ = client.inbounds().delete(iid).await; // first
        match client.inbounds().delete(iid).await {
            Err(_) => ok!("double-delete returns error"),
            Ok(()) => err!("double-delete should error", "got Ok"),
        }
        // wipe from created list since it's gone
    }
    // 15c. duplicate email in add_client
    let live = client.inbounds().list().await.unwrap_or_default();
    if let Some(any_inb) = live.iter().find(|i| i.remark.starts_with(PREFIX)) {
        let s = any_inb
            .settings
            .as_str()
            .map(String::from)
            .unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&s).unwrap_or_default();
        let cs: Vec<InboundClient> =
            serde_json::from_value(parsed.get("clients").cloned().unwrap_or_default())
                .unwrap_or_default();
        if let Some(existing) = cs.first() {
            let dup = make_client(&client.server().new_uuid().await?.uuid, &existing.email);
            match client.inbounds().add_client(any_inb.id, &[dup]).await {
                Err(_) => ok!("duplicate email rejected"),
                Ok(()) => {
                    // Some panel versions may allow it — report but don't fail.
                    println!("  • duplicate email accepted by panel (version dependent)");
                }
            }
        }
    }

    // ------------------------------------------------------------------
    step!("16. Cleanup");
    cleanup(&client).await;
    let leftover = client
        .inbounds()
        .list()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|i| i.remark.starts_with(PREFIX))
        .count();
    if leftover == 0 {
        ok!("no leftover scenario inbounds");
    } else {
        err!("cleanup", format!("{} leftovers", leftover));
    }

    must_unit!("logout", client.logout().await);

    let p = PASS.load(Ordering::Relaxed);
    let f = FAIL.load(Ordering::Relaxed);
    println!("\n===========================================");
    println!(" SCENARIOS  pass={}  fail={}  total={}", p, f, p + f);
    println!("===========================================\n");
    Ok(())
}
