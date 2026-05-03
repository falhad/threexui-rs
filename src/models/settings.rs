use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AllSetting {
    // Web server settings
    #[serde(default)]
    pub web_listen: String,
    #[serde(default)]
    pub web_domain: String,
    #[serde(default)]
    pub web_port: i32,
    #[serde(default)]
    pub web_cert_file: String,
    #[serde(default)]
    pub web_key_file: String,
    #[serde(default)]
    pub web_base_path: String,
    #[serde(default)]
    pub session_max_age: i32,
    // UI settings
    #[serde(default)]
    pub page_size: i32,
    #[serde(default)]
    pub expire_diff: i32,
    #[serde(default)]
    pub traffic_diff: i32,
    #[serde(default)]
    pub remark_model: String,
    #[serde(default)]
    pub datepicker: String,
    // Telegram bot settings
    #[serde(default)]
    pub tg_bot_enable: bool,
    #[serde(default)]
    pub tg_bot_token: String,
    #[serde(default)]
    pub tg_bot_proxy: String,
    #[serde(default)]
    pub tg_bot_api_server: String,
    #[serde(default)]
    pub tg_bot_chat_id: String,
    #[serde(default)]
    pub tg_run_time: String,
    #[serde(default)]
    pub tg_bot_backup: bool,
    #[serde(default)]
    pub tg_bot_login_notify: bool,
    #[serde(default)]
    pub tg_cpu: i32,
    #[serde(default)]
    pub tg_lang: String,
    // Security settings
    #[serde(default)]
    pub time_location: String,
    #[serde(default)]
    pub two_factor_enable: bool,
    #[serde(default)]
    pub two_factor_token: String,
    // Subscription settings
    #[serde(default)]
    pub sub_enable: bool,
    #[serde(default)]
    pub sub_json_enable: bool,
    #[serde(default)]
    pub sub_title: String,
    #[serde(default)]
    pub sub_support_url: String,
    #[serde(default)]
    pub sub_profile_url: String,
    #[serde(default)]
    pub sub_announce: String,
    #[serde(default)]
    pub sub_enable_routing: bool,
    #[serde(default)]
    pub sub_routing_rules: String,
    #[serde(default)]
    pub sub_listen: String,
    #[serde(default)]
    pub sub_port: i32,
    #[serde(default)]
    pub sub_path: String,
    #[serde(default)]
    pub sub_domain: String,
    #[serde(default)]
    pub sub_cert_file: String,
    #[serde(default)]
    pub sub_key_file: String,
    #[serde(default)]
    pub sub_updates: i32,
    #[serde(default)]
    pub external_traffic_inform_enable: bool,
    #[serde(default)]
    pub external_traffic_inform_uri: String,
    #[serde(default)]
    pub sub_encrypt: bool,
    #[serde(default)]
    pub sub_show_info: bool,
    #[serde(default)]
    pub sub_uri: String,
    #[serde(default)]
    pub sub_json_path: String,
    #[serde(default)]
    pub sub_json_uri: String,
    #[serde(default)]
    pub sub_clash_enable: bool,
    #[serde(default)]
    pub sub_clash_path: String,
    #[serde(default)]
    pub sub_clash_uri: String,
    #[serde(default)]
    pub sub_json_fragment: String,
    #[serde(default)]
    pub sub_json_noises: String,
    #[serde(default)]
    pub sub_json_mux: String,
    #[serde(default)]
    pub sub_json_rules: String,
    // LDAP settings
    #[serde(default)]
    pub ldap_enable: bool,
    #[serde(default)]
    pub ldap_host: String,
    #[serde(default)]
    pub ldap_port: i32,
    #[serde(default)]
    pub ldap_use_tls: bool,
    #[serde(default)]
    pub ldap_bind_dn: String,
    #[serde(default)]
    pub ldap_password: String,
    #[serde(default)]
    pub ldap_base_dn: String,
    #[serde(default)]
    pub ldap_user_filter: String,
    #[serde(default)]
    pub ldap_user_attr: String,
    #[serde(default)]
    pub ldap_vless_field: String,
    #[serde(default)]
    pub ldap_sync_cron: String,
    #[serde(default)]
    pub ldap_flag_field: String,
    #[serde(default)]
    pub ldap_truthy_values: String,
    #[serde(default)]
    pub ldap_invert_flag: bool,
    #[serde(default)]
    pub ldap_inbound_tags: String,
    #[serde(default)]
    pub ldap_auto_create: bool,
    #[serde(default)]
    pub ldap_auto_delete: bool,
    #[serde(default)]
    #[serde(rename = "ldapDefaultTotalGB")]
    pub ldap_default_total_gb: i32,
    #[serde(default)]
    pub ldap_default_expiry_days: i32,
    #[serde(default)]
    pub ldap_default_limit_ip: i32,
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
