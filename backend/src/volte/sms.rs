//! VoLTE SMS module.
//!
//! Routes SMS through IMS (SIP MESSAGE) when VoLTE is active,
//! reusing the VoWiFi SMS codec for GSM7/UCS2 encoding and RP-DU handling.

use serde::Serialize;

use crate::vowifi::sms::{
    self as vowifi_sms, MoSmsSubmission, MtSmsDeliver,
    SmsEncodingError,
};

use super::register::VolteRegisterState;
use super::transport::VolteTransport;

/// VoLTE SMS delivery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VolteSmsState {
    Idle,
    Sending,
    AckReceived,
    Failed,
    Receiving,
    Delivered,
}

impl VolteSmsState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Sending => "sending",
            Self::AckReceived => "ack_received",
            Self::Failed => "failed",
            Self::Receiving => "receiving",
            Self::Delivered => "delivered",
        }
    }
}

/// VoLTE SMS runtime state.
pub struct VolteSmsRuntime {
    state: VolteSmsState,
    last_mo_result: Option<MoSmsResult>,
    last_mt_deliver: Option<MtSmsDeliver>,
    statistics: SmsStatistics,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoSmsResult {
    pub message_id: String,
    pub state: &'static str,
    pub rpdu_status: Option<u8>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SmsStatistics {
    pub mo_sent: u64,
    pub mo_delivered: u64,
    pub mo_failed: u64,
    pub mt_received: u64,
    pub mt_delivered: u64,
}

impl VolteSmsRuntime {
    pub fn new() -> Self {
        Self {
            state: VolteSmsState::Idle,
            last_mo_result: None,
            last_mt_deliver: None,
            statistics: SmsStatistics::default(),
        }
    }

    /// Prepare MO SMS via VoLTE IMS.
    pub fn prepare_mo_sms(
        &mut self,
        recipient: &str,
        text: &str,
        service_center: &str,
    ) -> Result<MoSmsSubmission, SmsEncodingError> {
        self.state = VolteSmsState::Sending;
        vowifi_sms::build_single_part_mo_submission(recipient, text, service_center)
    }

    /// Handle RP-ACK received from network.
    pub fn on_rp_ack(&mut self, message_id: String, rpdu_status: u8) {
        self.state = VolteSmsState::AckReceived;
        self.statistics.mo_delivered += 1;
        self.last_mo_result = Some(MoSmsResult {
            message_id,
            state: "delivered",
            rpdu_status: Some(rpdu_status),
            error: None,
        });
    }

    /// Handle RP-ERROR received from network.
    pub fn on_rp_error(&mut self, message_id: String, rpdu_status: u8, cause: Option<String>) {
        self.state = VolteSmsState::Failed;
        self.statistics.mo_failed += 1;
        self.last_mo_result = Some(MoSmsResult {
            message_id,
            state: "failed",
            rpdu_status: Some(rpdu_status),
            error: cause,
        });
    }

    /// Process incoming MT SMS (SIP MESSAGE body).
    pub fn on_mt_sms_received(&mut self, body: &[u8]) -> Result<MtSmsDeliver, SmsEncodingError> {
        self.state = VolteSmsState::Receiving;
        let deliver = vowifi_sms::parse_mt_rp_data(body)?;
        self.statistics.mt_received += 1;
        self.last_mt_deliver = Some(deliver.clone());
        self.state = VolteSmsState::Delivered;
        Ok(deliver)
    }

    /// Build SIP MESSAGE body for MO SMS.
    pub fn build_sip_message_body(submission: &MoSmsSubmission) -> Result<Vec<u8>, SmsEncodingError> {
        Ok(submission.body.clone())
    }

    /// Build network RP-ACK for incoming MT SMS.
    pub fn build_mt_ack(reference: u8) -> Vec<u8> {
        vowifi_sms::build_network_rp_ack(reference)
    }

    pub fn summary(&self) -> VolteSmsSummary {
        VolteSmsSummary {
            state: self.state.as_str(),
            last_mo: self.last_mo_result.clone(),
            statistics: self.statistics.clone(),
        }
    }

    pub fn is_ready(&self, transport: &VolteTransport, register: &VolteRegisterState) -> bool {
        transport.state == super::transport::VolteTransportState::Connected
            && register.is_registered()
    }
}

/// Public summary of VoLTE SMS state.
#[derive(Debug, Clone, Serialize)]
pub struct VolteSmsSummary {
    pub state: &'static str,
    pub last_mo: Option<MoSmsResult>,
    pub statistics: SmsStatistics,
}
