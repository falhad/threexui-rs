//! Serde robustness — feed weird/real-world payloads at every model and confirm decode.
//! These pin behavior so newer 3x-ui versions don't silently break the decoder.

use threexui_rs::{
    AllSetting, ClientTraffic, CustomGeoAliases, CustomGeoResource, Inbound, InboundClient,
    Protocol, ServerStatus,
};

// ----- Inbound -----

#[test]
fn inbound_with_string_settings() {
    // 3x-ui returns `settings`/`streamSettings`/`sniffing` as JSON-encoded strings.
    let raw = r#"{
        "id":1,"up":0,"down":0,"total":0,"remark":"r","enable":true,
        "expiryTime":0,"listen":"","port":443,"protocol":"vless",
        "settings":"{\"clients\":[]}",
        "streamSettings":"{\"network\":\"tcp\"}",
        "tag":"t","sniffing":"{\"enabled\":false}",
        "clientStats":[]
    }"#;
    let inb: Inbound = serde_json::from_str(raw).unwrap();
    assert_eq!(inb.id, 1);
    assert!(inb.settings.is_string());
}

#[test]
fn inbound_with_object_settings() {
    let raw = r#"{
        "id":2,"up":1,"down":2,"total":0,"remark":"r","enable":true,
        "expiryTime":0,"listen":"0.0.0.0","port":80,"protocol":"vmess",
        "settings":{"clients":[]},
        "streamSettings":{"network":"ws"},
        "tag":"t","sniffing":{"enabled":true}
    }"#;
    let inb: Inbound = serde_json::from_str(raw).unwrap();
    assert!(inb.settings.is_object());
    assert_eq!(inb.protocol, Protocol::VMess);
}

#[test]
fn inbound_with_negative_expiry() {
    // 3x-ui uses negative expiry for "renew on first use" pattern.
    let raw = r#"{
        "id":1,"up":0,"down":0,"total":0,"remark":"r","enable":true,
        "expiryTime":-604800000,"listen":"","port":443,"protocol":"vless",
        "settings":{},"streamSettings":{},"tag":"t","sniffing":{}
    }"#;
    let inb: Inbound = serde_json::from_str(raw).unwrap();
    assert_eq!(inb.expiry_time, -604800000);
}

#[test]
fn inbound_unknown_protocol_falls_back() {
    let raw = r#"{
        "id":1,"up":0,"down":0,"total":0,"remark":"r","enable":true,
        "expiryTime":0,"listen":"","port":443,"protocol":"socks",
        "settings":{},"streamSettings":{},"tag":"t","sniffing":{}
    }"#;
    let inb: Inbound = serde_json::from_str(raw).unwrap();
    assert_eq!(inb.protocol, Protocol::Unknown);
}

#[test]
fn inbound_missing_optional_fields() {
    // Missing trafficReset, lastTrafficResetTime, allTime
    let raw = r#"{
        "id":1,"up":0,"down":0,"total":0,"remark":"r","enable":true,
        "expiryTime":0,"listen":"","port":443,"protocol":"trojan",
        "settings":{},"streamSettings":{},"tag":"t","sniffing":{}
    }"#;
    let inb: Inbound = serde_json::from_str(raw).unwrap();
    assert_eq!(inb.protocol, Protocol::Trojan);
    assert_eq!(inb.all_time, 0);
}

// ----- InboundClient -----

#[test]
fn inbound_client_minimal() {
    let raw = r#"{"id":"u","email":"e","enable":true,"reset":0}"#;
    let c: InboundClient = serde_json::from_str(raw).unwrap();
    assert_eq!(c.email, "e");
    assert_eq!(c.total_gb, 0);
    assert_eq!(c.limit_ip, 0);
    assert_eq!(c.tg_id, 0);
}

#[test]
fn inbound_client_serializes_total_gb_uppercase() {
    let c = InboundClient {
        id: "x".into(),
        email: "e@e".into(),
        enable: true,
        flow: "".into(),
        password: "".into(),
        security: "".into(),
        limit_ip: 0,
        total_gb: 12345,
        expiry_time: 0,
        tg_id: 99,
        sub_id: "".into(),
        comment: "".into(),
        reset: 0,
    };
    let json = serde_json::to_value(&c).unwrap();
    assert!(json.get("totalGB").is_some(), "must serialize as totalGB");
    assert!(
        json.get("totalGb").is_none(),
        "must not serialize as totalGb"
    );
    assert_eq!(json["totalGB"], 12345);
    assert_eq!(json["tgId"], 99);
}

#[test]
fn inbound_client_round_trip() {
    let c = InboundClient {
        id: "abc".into(),
        email: "round@trip".into(),
        enable: true,
        flow: "xtls-rprx-vision".into(),
        password: "".into(),
        security: "auto".into(),
        limit_ip: 3,
        total_gb: 1024 * 1024 * 1024,
        expiry_time: 1700000000000,
        tg_id: 12345,
        sub_id: "sub".into(),
        comment: "VIP".into(),
        reset: 0,
    };
    let s = serde_json::to_string(&c).unwrap();
    let back: InboundClient = serde_json::from_str(&s).unwrap();
    assert_eq!(back.email, c.email);
    assert_eq!(back.total_gb, c.total_gb);
    assert_eq!(back.tg_id, c.tg_id);
    assert_eq!(back.flow, c.flow);
    assert_eq!(back.comment, c.comment);
}

#[test]
fn inbound_client_tg_id_negative_string() {
    let raw = r#"{"id":"a","email":"e","enable":true,"limitIp":0,"totalGB":0,"expiryTime":0,"tgId":"-42","reset":0}"#;
    let c: InboundClient = serde_json::from_str(raw).unwrap();
    assert_eq!(c.tg_id, -42);
}

#[test]
fn inbound_client_tg_id_empty_string() {
    let raw = r#"{"id":"a","email":"e","enable":true,"limitIp":0,"totalGB":0,"expiryTime":0,"tgId":"","reset":0}"#;
    let c: InboundClient = serde_json::from_str(raw).unwrap();
    assert_eq!(c.tg_id, 0);
}

#[test]
fn inbound_client_tg_id_missing() {
    let raw =
        r#"{"id":"a","email":"e","enable":true,"limitIp":0,"totalGB":0,"expiryTime":0,"reset":0}"#;
    let c: InboundClient = serde_json::from_str(raw).unwrap();
    assert_eq!(c.tg_id, 0);
}

#[test]
fn inbound_client_tg_id_invalid_string_errors() {
    let raw = r#"{"id":"a","email":"e","enable":true,"limitIp":0,"totalGB":0,"expiryTime":0,"tgId":"not-a-number","reset":0}"#;
    let r: Result<InboundClient, _> = serde_json::from_str(raw);
    assert!(r.is_err());
}

#[test]
fn inbound_client_unknown_fields_ignored() {
    let raw = r#"{
        "id":"a","email":"e","enable":true,
        "limitIp":0,"totalGB":0,"expiryTime":0,"tgId":0,"reset":0,
        "future_field_v3":"whatever",
        "another":42,
        "nested":{"x":1}
    }"#;
    let _c: InboundClient = serde_json::from_str(raw).unwrap();
}

// ----- ClientTraffic -----

#[test]
fn client_traffic_with_uuid_and_last_online() {
    let raw = r#"{
        "id":1,"inboundId":2,"enable":true,"email":"e",
        "uuid":"some-uuid","subId":"s","up":100,"down":200,"allTime":300,
        "expiryTime":0,"total":0,"reset":0,"lastOnline":1700000000
    }"#;
    let t: ClientTraffic = serde_json::from_str(raw).unwrap();
    assert_eq!(t.uuid, "some-uuid");
    assert_eq!(t.last_online, 1700000000);
    assert_eq!(t.inbound_id, 2);
}

#[test]
fn client_traffic_missing_optionals() {
    let raw = r#"{
        "id":1,"inboundId":2,"enable":true,"email":"e",
        "up":0,"down":0,"expiryTime":0,"total":0
    }"#;
    let t: ClientTraffic = serde_json::from_str(raw).unwrap();
    assert_eq!(t.uuid, "");
    assert_eq!(t.sub_id, "");
}

// ----- CustomGeoAliases -----

#[test]
fn custom_geo_aliases_null_object() {
    let raw = r#"{"geosite":null,"geoip":null}"#;
    let a: CustomGeoAliases = serde_json::from_str(raw).unwrap();
    assert!(a.geosite.is_empty());
    assert!(a.geoip.is_empty());
}

#[test]
fn custom_geo_aliases_populated() {
    let raw = r#"{"geosite":["a","b"],"geoip":["c"]}"#;
    let a: CustomGeoAliases = serde_json::from_str(raw).unwrap();
    assert_eq!(a.geosite, vec!["a", "b"]);
    assert_eq!(a.geoip, vec!["c"]);
}

#[test]
fn custom_geo_aliases_empty_object() {
    let raw = r#"{}"#;
    let a: CustomGeoAliases = serde_json::from_str(raw).unwrap();
    assert!(a.geosite.is_empty());
}

#[test]
fn custom_geo_resource_full() {
    let raw = r#"{
        "id":7,"type":"geosite","alias":"my","url":"https://x/y.dat",
        "localPath":"/tmp/y.dat","lastUpdatedAt":1,"createdAt":2,"updatedAt":3
    }"#;
    let r: CustomGeoResource = serde_json::from_str(raw).unwrap();
    assert_eq!(r.id, 7);
    assert_eq!(r.geo_type, "geosite");
}

// ----- ServerStatus -----

#[test]
fn server_status_with_zero_traffic() {
    let raw = r#"{
        "cpu":0.0,"cpuCores":2,"logicalPro":2,"cpuSpeedMhz":2400.0,
        "mem":{"current":0,"total":0},
        "swap":{"current":0,"total":0},
        "disk":{"current":0,"total":0},
        "xray":{"state":"stopped","errorMsg":"","version":""},
        "uptime":0,"loads":[0.0,0.0,0.0],
        "tcpCount":0,"udpCount":0,
        "netIO":{"up":0,"down":0},
        "netTraffic":{"sent":0,"recv":0},
        "publicIP":{"ipv4":"","ipv6":""},
        "appStats":{"threads":0,"mem":0,"uptime":0}
    }"#;
    let s: ServerStatus = serde_json::from_str(raw).unwrap();
    assert_eq!(s.xray.state, "stopped");
}

// ----- AllSetting -----

#[test]
fn all_setting_with_ldap_section() {
    let raw = r#"{
        "webPort":2053,"ldapEnable":true,"ldapHost":"ldap.example.com",
        "ldapPort":389,"ldapUseTLS":true,"ldapAutoCreate":true,
        "ldapDefaultTotalGB":50
    }"#;
    let s: AllSetting = serde_json::from_str(raw).unwrap();
    assert_eq!(s.web_port, 2053);
    assert!(s.ldap_enable);
    assert_eq!(s.ldap_host, "ldap.example.com");
    assert_eq!(s.ldap_default_total_gb, 50);
}

#[test]
fn all_setting_with_extra_unknown_fields() {
    let raw = r#"{"webPort":443,"futureFlag":true,"newSetting":"value"}"#;
    let s: AllSetting = serde_json::from_str(raw).unwrap();
    assert_eq!(s.web_port, 443);
}

// ----- Real-world prod payload -----

#[test]
fn real_world_settings_clients_decode() {
    // Verbatim slice (4 clients) from a production 3x-ui /panel/api/inbounds/get/1 response.
    let settings_json = r#"{"clients":[
        {"comment":"","created_at":1777667608000,"email":"client-alpha","enable":true,
         "expiryTime":0,"id":"33f83d80-aa43-40d5-b1a6-79dcf1e2663e",
         "limitIp":0,"reset":0,"security":"","subId":"","tgId":0,"totalGB":0,
         "updated_at":1777667608000},
        {"comment":"@falhawd","created_at":1777668617000,"email":"u77313385_599","enable":true,
         "expiryTime":-604800000,"flow":"","id":"e8a255e3-1ac7-4d1f-b0b6-a2c65eefea12",
         "limitIp":1,"reset":0,"subId":"u77313385_599","tgId":"77313385",
         "totalGB":1073741824,"updated_at":1777668617000},
        {"comment":"AdminTrial","created_at":1777715925000,"email":"atrial_x","enable":true,
         "expiryTime":-86400000,"flow":"","id":"dabc9f4b-87a1-4347-b29d-d7b112c3a991",
         "limitIp":1,"reset":0,"subId":"atrial_x","tgId":"77313385",
         "totalGB":104857600,"updated_at":1777715925000},
        {"comment":"Disabled","created_at":0,"email":"off","enable":false,
         "expiryTime":0,"id":"00000000-0000-0000-0000-000000000000",
         "limitIp":0,"reset":3,"subId":"","tgId":null,"totalGB":0,
         "updated_at":0}
    ]}"#;
    #[derive(serde::Deserialize)]
    struct Wrap {
        clients: Vec<InboundClient>,
    }
    let w: Wrap = serde_json::from_str(settings_json).unwrap();
    assert_eq!(w.clients.len(), 4);
    assert_eq!(w.clients[1].tg_id, 77313385);
    assert_eq!(w.clients[1].total_gb, 1073741824);
    assert!(!w.clients[3].enable);
    assert_eq!(w.clients[3].tg_id, 0);
    assert_eq!(w.clients[3].reset, 3);
}
