//! # threexui-rs
//!
//! Async Rust SDK for the [3x-ui](https://github.com/MHSanaei/3x-ui) panel API.
//!
//! This crate targets 3x-ui **v2.9.3**. The library version mirrors the panel version —
//! `threexui-rs v2.9.3` is compatible with 3x-ui `v2.9.3`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use threexui_rs::{Client, ClientConfig};
//!
//! #[tokio::main]
//! async fn main() -> threexui_rs::Result<()> {
//!     let config = ClientConfig::builder()
//!         .host("192.168.1.1")
//!         .port(2053)
//!         .build()?;
//!
//!     let client = Client::new(config);
//!     client.login("admin", "admin123").await?;
//!
//!     let inbounds = client.inbounds().list().await?;
//!     println!("Found {} inbounds", inbounds.len());
//!
//!     client.logout().await?;
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod models;

pub use client::Client;
pub use config::ClientConfig;
pub use error::{Error, Result};

// Inbound models
pub use models::inbound::{ClientTraffic, Inbound, InboundClient, Protocol};

// Server models
pub use models::server::{
    AppStats, CpuHistoryPoint, EchCert, Mldsa65Keys, Mlkem768Keys, NetIO, NetTraffic, PublicIP,
    ResourceStat, ServerStatus, UuidResponse, VlessAuth, VlessEncResult, X25519Cert, XrayState,
};

// Settings
pub use models::settings::AllSetting;

// Xray models
pub use models::xray::{NordAction, OutboundTraffic, WarpAction, XraySetting};

// Custom geo models
pub use models::custom_geo::{CreateCustomGeo, CustomGeoAliases, CustomGeoResource};
