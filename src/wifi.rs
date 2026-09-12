use crate::config;
use crate::ui::log;
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
