//! VoLTE (Voice over LTE) SMS takeover module.
//!
//! Provides IMS-based SMS over LTE, reusing VoWiFi SMS codec layer
//! while replacing the transport (LTE PDN instead of ePDG tunnel).

pub mod config;
pub mod network;
pub mod register;
pub mod sms;
pub mod transport;
