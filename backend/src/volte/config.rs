//! VoLTE carrier configuration and policy definitions.

use serde::Serialize;

/// VoLTE-specific carrier configuration.
#[derive(Debug, Clone, Serialize)]
pub struct VolteConfig {
    pub feature_enabled: bool,
    pub ims_apn: String,
    pub proxy_domain: String,
    pub ims_port: u16,
    pub transport_protocol: String,
    pub sms_over_ims: String,
    pub reg_expiry_secs: u32,
    pub pcscf_discovery: String,
    pub static_pcscf: Option<String>,
    pub pdn_type: String,
    pub pdn_cid: u8,
}

impl Default for VolteConfig {
    fn default() -> Self {
        Self {
            feature_enabled: false,
            ims_apn: "ims".to_string(),
            proxy_domain: "ims.mnc001.mcc460.3gppnetwork.org".to_string(),
            ims_port: 5060,
            transport_protocol: "tcp".to_string(),
            sms_over_ims: "ims_prefer".to_string(),
            reg_expiry_secs: 3600,
            pcscf_discovery: "dhcp".to_string(),
            static_pcscf: None,
            pdn_type: "IPv6".to_string(),
            pdn_cid: 3,
        }
    }
}

/// SMS routing policy for VoLTE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SmsRoutePolicy {
    ImsOnly,
    ImsPrefer,
    CsOnly,
}

impl SmsRoutePolicy {
    pub fn from_str(s: &str) -> Self {
        match s {
            "ims_only" => Self::ImsOnly,
            "ims_prefer" => Self::ImsPrefer,
            "cs_fallback" | "cs_only" => Self::CsOnly,
            _ => Self::ImsPrefer,
        }
    }
}
