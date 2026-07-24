//! VoLTE IMS transport layer.

use serde::Serialize;

/// Transport state for VoLTE IMS connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VolteTransportState {
    Idle,
    PdnConnected,
    PscfDiscovered,
    SipReady,
    Connected,
    Failed,
}

impl VolteTransportState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::PdnConnected => "pdn_connected",
            Self::PscfDiscovered => "pcscf_discovered",
            Self::SipReady => "sip_ready",
            Self::Connected => "connected",
            Self::Failed => "failed",
        }
    }
}

/// Summary of VoLTE transport status.
#[derive(Debug, Clone, Serialize)]
pub struct VolteTransportSummary {
    pub state: &'static str,
    pub pdn_active: bool,
    pub pcscf_resolved: bool,
    pub pcscf_address: Option<String>,
    pub ims_port: u16,
    pub transport_protocol: &'static str,
    pub last_error: Option<String>,
}

/// VoLTE IMS transport manager.
pub struct VolteTransport {
    pub(crate) state: VolteTransportState,
    pub(crate) pdn_active: bool,
    pub(crate) pcscf_address: Option<String>,
    pub(crate) ims_port: u16,
    pub(crate) transport_protocol: String,
    pub(crate) last_error: Option<String>,
}

impl VolteTransport {
    pub fn new(ims_port: u16, transport_protocol: String) -> Self {
        Self {
            state: VolteTransportState::Idle,
            pdn_active: false,
            pcscf_address: None,
            ims_port,
            transport_protocol,
            last_error: None,
        }
    }

    pub fn on_pdn_connected(&mut self) {
        self.pdn_active = true;
        self.state = VolteTransportState::PdnConnected;
    }

    pub fn on_pcscf_discovered(&mut self, address: String) {
        self.pcscf_address = Some(address);
        self.state = VolteTransportState::PscfDiscovered;
    }

    pub fn on_sip_ready(&mut self) {
        self.state = VolteTransportState::SipReady;
    }

    pub fn on_connected(&mut self) {
        self.state = VolteTransportState::Connected;
    }

    pub fn on_failed(&mut self, error: String) {
        self.last_error = Some(error);
        self.state = VolteTransportState::Failed;
    }

    pub fn reset(&mut self) {
        self.state = VolteTransportState::Idle;
        self.pdn_active = false;
        self.pcscf_address = None;
        self.last_error = None;
    }

    pub fn summary(&self) -> VolteTransportSummary {
        VolteTransportSummary {
            state: self.state.as_str(),
            pdn_active: self.pdn_active,
            pcscf_resolved: self.pcscf_address.is_some(),
            pcscf_address: self.pcscf_address.clone(),
            ims_port: self.ims_port,
            transport_protocol: match self.transport_protocol.as_str() {
                "tls" => "tls",
                "udp" => "udp",
                _ => "tcp",
            },
            last_error: self.last_error.clone(),
        }
    }
}
