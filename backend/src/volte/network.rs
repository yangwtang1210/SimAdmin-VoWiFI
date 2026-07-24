//! VoLTE/VoWiFi 网络诊断模块。
//!
//! 支持通过 mmcli/qmicli 诊断 IMS PDN、网络接口、连通性等状态。

use serde::Serialize;
use std::process::Command;

fn run_cmd(cmd: &str) -> String {
    Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

fn mmcli_at(command: &str) -> String {
    run_cmd(&format!("mmcli -m any --command='{}' 2>/dev/null", command))
}

fn qmicli_wds(args: &str) -> String {
    run_cmd(&format!(
        "qmicli -d /dev/wwan0at1 --device-open-qmi --device-open-proxy {} 2>/dev/null",
        args
    ))
}

/// 诊断报告
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticReport {
    pub timestamp: String,
    pub modem: ModemStatus,
    pub network: NetworkStatus,
    pub simadmin: SimadminStatus,
    pub ims_pdn: ImsPdnStatus,
    pub connectivity: ConnectivityStatus,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModemStatus {
    pub available: bool,
    pub pdp_contexts: Vec<String>,
    pub ims_pdp_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkStatus {
    pub wwan0_up: bool,
    pub wwan1_up: bool,
    pub wwan0_ipv6: Option<String>,
    pub wwan1_ipv6: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimadminStatus {
    pub active: bool,
    pub pid: Option<u32>,
    pub last_volte_log: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImsPdnStatus {
    pub ims_apn_active: bool,
    pub ims_feature_enabled: bool,
    pub p_cscf: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectivityStatus {
    pub internet_ipv4: bool,
    pub internet_ipv6: bool,
}

pub fn check_modem() -> ModemStatus {
    let mmcli_l = run_cmd("mmcli -L 2>/dev/null");
    let available = mmcli_l.contains("/org/freedesktop/ModemManager1/Modem/");
    let pdp_raw = mmcli_at("AT+CGDCONT?");
    let pdp_contexts: Vec<String> = pdp_raw
        .lines()
        .filter(|l| l.contains("+CGDCONT:"))
        .map(|s| s.to_string())
        .collect();
    let ims_pdp_enabled = pdp_contexts.iter().any(|l| l.contains("ims"));
    ModemStatus { available, pdp_contexts, ims_pdp_enabled }
}

pub fn check_network() -> NetworkStatus {
    let br = run_cmd("ip -br address show 2>/dev/null");
    let parse_ipv6 = |iface: &str| -> Option<String> {
        br.lines()
            .find(|l| l.starts_with(iface))
            .and_then(|l| l.split_whitespace().find(|f| f.contains(':') && !f.contains("fe80")))
            .map(|s| s.to_string())
    };
    NetworkStatus {
        wwan0_up: br.lines().any(|l| l.starts_with("wwan0")),
        wwan1_up: br.lines().any(|l| l.starts_with("wwan1")),
        wwan0_ipv6: parse_ipv6("wwan0"),
        wwan1_ipv6: parse_ipv6("wwan1"),
    }
}

pub fn check_simadmin() -> SimadminStatus {
    let active = run_cmd("systemctl is-active simadmin 2>/dev/null").trim() == "active";
    let pid_str = run_cmd("systemctl show simadmin --property=MainPID --value 2>/dev/null").trim().to_string();
    let last_log = run_cmd(
        "journalctl -u simadmin -n 50 --no-pager -o short-iso 2>/dev/null \
         | grep -E 'VoLTE|IMS|P-CSCF|REGISTER|runtime failed' | tail -1",
    ).trim().to_string();
    SimadminStatus {
        active,
        pid: pid_str.parse::<u32>().ok(),
        last_volte_log: if last_log.is_empty() { None } else { Some(last_log) },
    }
}

pub fn check_ims_pdn() -> ImsPdnStatus {
    let cgcontrdp = mmcli_at("AT+CGCONTRDP");
    let qcpdpimscfge = mmcli_at("AT$QCPDPIMSCFGE?");
    let ims_feature_enabled = qcpdpimscfge.lines().any(|l| l.contains(",1,1,1") || l.contains(",1,1,0"));
    let qmi_settings = qmicli_wds("--wds-get-current-settings --client-no-release-cid");
    let p_cscf = qmi_settings.lines().find(|l| l.to_lowercase().contains("pcscf")).map(|s| s.trim().to_string());
    ImsPdnStatus {
        ims_apn_active: cgcontrdp.lines().any(|l| l.contains("ims")),
        ims_feature_enabled,
        p_cscf,
    }
}

pub fn check_connectivity() -> ConnectivityStatus {
    ConnectivityStatus {
        internet_ipv4: run_cmd("ping -I wwan0 -c 1 -W 3 8.8.8.8 2>/dev/null").contains("1 received"),
        internet_ipv6: run_cmd("ping -6 -I wwan0 -c 1 -W 3 2001:4860:4860::8888 2>/dev/null").contains("1 received"),
    }
}

pub fn run_full_diagnostic() -> DiagnosticReport {
    let mut errors = Vec::new();
    let modem = check_modem();
    if !modem.available { errors.push("Modem not available".into()); }
    let network = check_network();
    let simadmin = check_simadmin();
    let ims_pdn = check_ims_pdn();
    let connectivity = check_connectivity();
    if !connectivity.internet_ipv4 && !connectivity.internet_ipv6 {
        errors.push("No internet connectivity".into());
    }
    if !ims_pdn.ims_feature_enabled {
        errors.push("IMS PDP feature not enabled".into());
    }
    DiagnosticReport {
        timestamp: String::new(),
        modem, network, simadmin, ims_pdn, connectivity, errors,
    }
}

pub fn cleanup_ims_pdn() {
    run_cmd("mmcli -m any --command='AT+CGACT=0,3' || true");
    run_cmd("mmcli -m any --command='AT$QCPDPIMSCFGE=3,0,0,0' || true");
    run_cmd("ip -6 addr flush dev wwan1 2>/dev/null || true");
    run_cmd("ip xfrm policy flush 2>/dev/null || true");
    run_cmd("ip xfrm state flush 2>/dev/null || true");
}

pub fn setup_ims_pdn_ipv6() {
    cleanup_ims_pdn();
    run_cmd("mmcli -m any --command='AT+CGDCONT=3,\"IPV6\",\"ims\"'");
    run_cmd("mmcli -m any --command='AT$QCPDPIMSCFGE=3,1,1,1'");
}
