//! VoLTE IMS registration state machine.

use serde::Serialize;

/// VoLTE IMS registration phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VolteRegPhase {
    Idle,
    InitialRegister,
    DigestAuthPending,
    Registered,
    Deregistered,
    Failed,
}

impl VolteRegPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::InitialRegister => "initial_register",
            Self::DigestAuthPending => "digest_auth_pending",
            Self::Registered => "registered",
            Self::Deregistered => "deregistered",
            Self::Failed => "failed",
        }
    }
}

/// VoLTE IMS registration state.
pub struct VolteRegisterState {
    pub phase: VolteRegPhase,
    pub sip_uri: String,
    pub realm: String,
    pub registered_expires: Option<u32>,
    pub last_sip_status: Option<u16>,
    pub retry_after: Option<u32>,
    pub last_error: Option<String>,
}

impl VolteRegisterState {
    pub fn new(sip_uri: String, realm: String) -> Self {
        Self {
            phase: VolteRegPhase::Idle,
            sip_uri,
            realm,
            registered_expires: None,
            last_sip_status: None,
            retry_after: None,
            last_error: None,
        }
    }

    pub fn start_register(&mut self) {
        self.phase = VolteRegPhase::InitialRegister;
        self.last_error = None;
    }

    pub fn on_digest_challenge(&mut self) {
        self.phase = VolteRegPhase::DigestAuthPending;
    }

    pub fn on_registered(&mut self, expires: u32) {
        self.phase = VolteRegPhase::Registered;
        self.registered_expires = Some(expires);
        self.last_sip_status = Some(200);
    }

    pub fn on_failed(&mut self, status: u16, reason: Option<String>) {
        self.phase = VolteRegPhase::Failed;
        self.last_sip_status = Some(status);
        self.last_error = reason;
    }

    pub fn deregister(&mut self) {
        self.phase = VolteRegPhase::Deregistered;
    }

    pub fn is_registered(&self) -> bool {
        self.phase == VolteRegPhase::Registered
    }

    pub fn summary(&self) -> VolteRegSummary {
        VolteRegSummary {
            phase: self.phase.as_str(),
            sip_uri: self.sip_uri.clone(),
            realm: self.realm.clone(),
            registered: self.is_registered(),
            registered_expires_seconds: self.registered_expires,
            last_sip_status: self.last_sip_status,
            retry_after_seconds: self.retry_after,
            last_error: self.last_error.clone(),
        }
    }
}

/// Public summary of VoLTE registration state.
#[derive(Debug, Clone, Serialize)]
pub struct VolteRegSummary {
    pub phase: &'static str,
    pub sip_uri: String,
    pub realm: String,
    pub registered: bool,
    pub registered_expires_seconds: Option<u32>,
    pub last_sip_status: Option<u16>,
    pub retry_after_seconds: Option<u32>,
    pub last_error: Option<String>,
}
