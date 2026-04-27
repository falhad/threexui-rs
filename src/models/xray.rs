use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySetting {
    pub xray_setting: serde_json::Value,
    pub inbound_tags: serde_json::Value,
    pub outbound_test_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutboundTraffic {
    pub id: i64,
    pub tag: String,
    pub up: i64,
    pub down: i64,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub enum WarpAction {
    Data,
    Delete,
    Config,
    Register { private_key: String, public_key: String },
    SetLicense(String),
}

impl WarpAction {
    pub fn action_str(&self) -> &str {
        match self {
            WarpAction::Data => "data",
            WarpAction::Delete => "del",
            WarpAction::Config => "config",
            WarpAction::Register { .. } => "reg",
            WarpAction::SetLicense(_) => "license",
        }
    }
}

#[derive(Debug, Clone)]
pub enum NordAction {
    Countries,
    Servers { country_id: String },
    Register { token: String },
    SetKey(String),
    Data,
    Delete,
}

impl NordAction {
    pub fn action_str(&self) -> &str {
        match self {
            NordAction::Countries => "countries",
            NordAction::Servers { .. } => "servers",
            NordAction::Register { .. } => "reg",
            NordAction::SetKey(_) => "setKey",
            NordAction::Data => "data",
            NordAction::Delete => "del",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warp_action_str() {
        assert_eq!(WarpAction::Data.action_str(), "data");
        assert_eq!(WarpAction::Delete.action_str(), "del");
        assert_eq!(WarpAction::Register {
            private_key: "a".into(),
            public_key: "b".into()
        }.action_str(), "reg");
    }

    #[test]
    fn nord_action_str() {
        assert_eq!(NordAction::Countries.action_str(), "countries");
        assert_eq!(NordAction::Servers { country_id: "1".into() }.action_str(), "servers");
    }
}
