use crate::config;
use crate::ui::log;
use std::collections::HashSet;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

pub fn get_default_wifi_con() -> String {
    std::env::var("PESU_WIFI_CON").unwrap_or_else(|_| "PESU-EC-Campus".to_string())
}

pub fn get_current_wifi_ssid() -> Option<String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "name,type", "connection", "show", "--active"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains(":802-11-wireless") {
            if let Some(name) = line.split(':').next() {
                return Some(name.trim().to_string());
            }
        }
    }
    None
}

pub fn is_campus_ssid(ssid: &str) -> bool {
    let cfg = config::load_config();
    let preferred = cfg.preferred_ssid.unwrap_or_else(get_default_wifi_con);
    if ssid == preferred {
        return true;
    }
    let s = ssid.to_uppercase();
    s.contains("PESU")
        || s.contains("PES-WIFI")
        || s.contains("PES_WIFI")
        || s.contains("PESUNIVERSITY")
}

pub fn get_active_wifi_ssid() -> String {
    if let Some(curr) = get_current_wifi_ssid() {
        return curr;
    }
    let cfg = config::load_config();
    cfg.preferred_ssid.unwrap_or_else(get_default_wifi_con)
}

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub in_use: bool,
    pub signal: String,
    pub security: String,
    pub bars: String,
    pub saved: bool,
}

pub fn scan_wifi_networks() -> Vec<WifiNetwork> {
    let mut lines = Vec::new();
    let res = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "IN-USE,SSID,SIGNAL,SECURITY,BARS",
            "dev",
            "wifi",
            "list",
            "--rescan",
            "no",
        ])
        .output();

    if let Ok(out) = res {
        let stdout = String::from_utf8_lossy(&out.stdout);
        lines.extend(stdout.lines().map(|s| s.to_string()));
    }

    if lines.is_empty() {
        let res2 = Command::new("nmcli")
            .args([
                "-t",
                "-f",
                "IN-USE,SSID,SIGNAL,SECURITY,BARS",
                "dev",
                "wifi",
                "list",
                "--rescan",
                "yes",
            ])
            .output();
        if let Ok(out) = res2 {
            let stdout = String::from_utf8_lossy(&out.stdout);
            lines.extend(stdout.lines().map(|s| s.to_string()));
        }
    }

    let mut saved_conns = HashSet::new();
    if let Ok(out) = Command::new("nmcli")
        .args(["-t", "-f", "NAME,TYPE", "connection", "show"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 && parts[1] == "802-11-wireless" {
                saved_conns.insert(parts[0].trim().to_string());
            }
        }
    }

    let mut seen = HashSet::new();
    let mut list = Vec::new();

    for line in lines {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            let in_use = parts[0].trim() == "*";
            let ssid = parts[1].trim().to_string();
            let signal = parts[2].trim().to_string();
            let security = if parts.len() > 3 {
                parts[3].trim().to_string()
            } else {
                String::new()
            };
            let bars = if parts.len() > 4 {
                parts[4].trim().to_string()
            } else {
                String::new()
            };

            if !ssid.is_empty() && !seen.contains(&ssid) {
                seen.insert(ssid.clone());
                list.push(WifiNetwork {
                    in_use,
                    signal,
                    security: if security.is_empty() {
                        "Open".to_string()
                    } else {
                        security
                    },
                    bars,
                    saved: saved_conns.contains(&ssid),
                    ssid,
                });
            }
        }
    }

    for s in saved_conns {
        if !seen.contains(&s) {
            seen.insert(s.clone());
            list.push(WifiNetwork {
                ssid: s,
                in_use: false,
                signal: "??".to_string(),
                security: String::new(),
                bars: String::new(),
                saved: true,
            });
        }
    }

    list
}

pub fn connect_wifi(ssid: &str) -> Result<(), String> {
    let res = Command::new("nmcli")
        .args(["connection", "up", ssid])
        .output();

    let mut ok = false;
    let mut err_msg = String::new();

    if let Ok(out) = res {
        if out.status.success() {
            ok = true;
        } else {
            err_msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        }
    }

    if !ok {
        let res2 = Command::new("nmcli")
            .args(["dev", "wifi", "connect", ssid])
            .output();
        if let Ok(out) = res2 {
            if out.status.success() {
                ok = true;
            } else {
                let msg2 = String::from_utf8_lossy(&out.stderr).trim().to_string();
                if !msg2.is_empty() {
                    err_msg = msg2;
                }
            }
        }
    }

    if ok {
        let mut cfg = config::load_config();
        cfg.preferred_ssid = Some(ssid.to_string());
        let _ = config::save_config(&cfg);
        Ok(())
    } else {
        Err(if !err_msg.is_empty() {
            err_msg
        } else {
            format!("Failed to connect to '{}'", ssid)
        })
    }
}

pub fn heal_network(tier: u8) {
    let wifi_con = get_current_wifi_ssid().unwrap_or_else(get_active_wifi_ssid);
    if !is_campus_ssid(&wifi_con) {
        log(&format!(
            "[Self-Healing] Skipped: '{}' is not a PESU campus network.",
            wifi_con
        ));
        return;
    }

    match tier {
        1 => {
            log(&format!("[Self-Healing L1] Reconnecting to '{}'...", wifi_con));
            let _ = Command::new("nmcli")
                .args(["connection", "up", &wifi_con])
                .output();
        }
        2 => {
            log("[Self-Healing L2] Cycling Wi-Fi radio...");
            let _ = Command::new("nmcli").args(["radio", "wifi", "off"]).output();
            sleep(Duration::from_secs(2));
            let _ = Command::new("nmcli").args(["radio", "wifi", "on"]).output();
            sleep(Duration::from_secs(5));
            let _ = Command::new("nmcli")
                .args(["connection", "up", &wifi_con])
                .output();
        }
        3 => {
            log("[Self-Healing L3] Re-engaging Wi-Fi device interface...");
            let is_root = Command::new("id").arg("-u").output().map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0").unwrap_or(false);

            if is_root {
                let _ = Command::new("systemctl")
                    .args(["restart", "NetworkManager"])
                    .output();
                sleep(Duration::from_secs(6));
            } else if let Ok(out) = Command::new("nmcli")
                .args(["-t", "-f", "DEVICE,TYPE", "dev"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if line.contains(":wifi") {
                        if let Some(dev_name) = line.split(':').next() {
                            let _ = Command::new("nmcli")
                                .args(["device", "disconnect", dev_name])
                                .output();
                            sleep(Duration::from_secs(2));
                            let _ = Command::new("nmcli")
                                .args(["device", "connect", dev_name])
                                .output();
                            break;
                        }
                    }
                }
            }
            let _ = Command::new("nmcli")
                .args(["connection", "up", &wifi_con])
                .output();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_campus_ssid() {
        assert!(is_campus_ssid("PESU-EC-Campus"));
        assert!(is_campus_ssid("PESU-RR-Campus"));
        assert!(is_campus_ssid("PES-WIFI"));
        assert!(is_campus_ssid("PES_WIFI"));
        assert!(is_campus_ssid("pesuniversity-guest"));
        assert!(!is_campus_ssid("Home_Network"));
        assert!(!is_campus_ssid("Space"));
        assert!(!is_campus_ssid(""));
    }
}
