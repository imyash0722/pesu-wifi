use crate::config;
use crate::ui::log;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

pub fn get_default_wifi_con() -> String {
    std::env::var("PESU_WIFI_CON").unwrap_or_else(|_| "PESU-EC-Campus".to_string())
}

#[cfg(unix)]
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

#[cfg(unix)]
pub fn get_current_wifi_bssid() -> Option<String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "active,bssid", "dev", "wifi"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.starts_with("yes:") {
            let bssid = line.trim_start_matches("yes:").replace('\\', "");
            if !bssid.trim().is_empty() {
                return Some(bssid.trim().to_uppercase());
            }
        }
    }
    None
}

#[cfg(windows)]
pub fn get_current_wifi_ssid() -> Option<String> {
    if let Some(ssid) = get_current_wifi_ssid_wlanapi() {
        return Some(ssid);
    }
    get_current_wifi_ssid_netsh()
}

#[cfg(windows)]
pub fn get_current_wifi_bssid() -> Option<String> {
    if let Some(bssid) = get_current_wifi_bssid_wlanapi() {
        return Some(bssid);
    }
    get_current_wifi_bssid_netsh()
}

#[cfg(windows)]
fn get_current_wifi_ssid_wlanapi() -> Option<String> {
    use windows_sys::Win32::NetworkManagement::WiFi::*;
    unsafe {
        let mut client_version = 0;
        let mut handle: *mut std::ffi::c_void = std::ptr::null_mut();
        let res = WlanOpenHandle(2, std::ptr::null(), &mut client_version, &mut handle);
        if res != 0 || handle.is_null() {
            return None;
        }

        let mut interface_list = std::ptr::null_mut();
        let res = WlanEnumInterfaces(handle, std::ptr::null_mut(), &mut interface_list);
        if res != 0 || interface_list.is_null() {
            WlanCloseHandle(handle, std::ptr::null_mut());
            return None;
        }

        let mut active_ssid = None;
        let num_items = (*interface_list).dwNumberOfItems;
        for i in 0..num_items {
            let info = (*interface_list).InterfaceInfo[i as usize];
            if info.isState == wlan_interface_state_connected {
                let mut data_size = 0;
                let mut data_ptr = std::ptr::null_mut();
                let mut opcode_value_type = 0;
                let res = WlanQueryInterface(
                    handle,
                    &info.InterfaceGuid,
                    wlan_intf_opcode_current_connection,
                    std::ptr::null_mut(),
                    &mut data_size,
                    &mut data_ptr,
                    &mut opcode_value_type,
                );
                if res == 0 && !data_ptr.is_null() {
                    let conn = data_ptr as *const WLAN_CONNECTION_ATTRIBUTES;
                    let dot11_ssid = (*conn).wlanAssociationAttributes.dot11Ssid;
                    let len = dot11_ssid.uSSIDLength as usize;
                    if len > 0 && len <= 32 {
                        let ssid_bytes = &dot11_ssid.ucSSID[..len];
                        if let Ok(s) = std::str::from_utf8(ssid_bytes) {
                            active_ssid = Some(s.to_string());
                        }
                    }
                    WlanFreeMemory(data_ptr);
                }
                if active_ssid.is_some() {
                    break;
                }
            }
        }

        WlanFreeMemory(interface_list as *mut _);
        WlanCloseHandle(handle, std::ptr::null_mut());
        active_ssid
    }
}

#[cfg(windows)]
fn get_current_wifi_ssid_netsh() -> Option<String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut is_connected = false;
    let mut ssid = None;

    for line in text.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim().to_lowercase();
            let v = v.trim().to_string();
            if k == "state" && v.eq_ignore_ascii_case("connected") {
                is_connected = true;
            } else if k == "ssid" && !v.is_empty() {
                ssid = Some(v);
            }
        }
    }

    if is_connected {
        ssid
    } else {
        None
    }
}

#[cfg(windows)]
fn get_current_wifi_bssid_wlanapi() -> Option<String> {
    use windows_sys::Win32::NetworkManagement::WiFi::*;
    unsafe {
        let mut client_version = 0;
        let mut handle: *mut std::ffi::c_void = std::ptr::null_mut();
        let res = WlanOpenHandle(2, std::ptr::null(), &mut client_version, &mut handle);
        if res != 0 || handle.is_null() {
            return None;
        }

        let mut interface_list = std::ptr::null_mut();
        let res = WlanEnumInterfaces(handle, std::ptr::null_mut(), &mut interface_list);
        if res != 0 || interface_list.is_null() {
            WlanCloseHandle(handle, std::ptr::null_mut());
            return None;
        }

        let mut active_bssid = None;
        let num_items = (*interface_list).dwNumberOfItems;
        for i in 0..num_items {
            let info = (*interface_list).InterfaceInfo[i as usize];
            if info.isState == wlan_interface_state_connected {
                let mut data_size = 0;
                let mut data_ptr = std::ptr::null_mut();
                let mut opcode_value_type = 0;
                let res = WlanQueryInterface(
                    handle,
                    &info.InterfaceGuid,
                    wlan_intf_opcode_current_connection,
                    std::ptr::null_mut(),
                    &mut data_size,
                    &mut data_ptr,
                    &mut opcode_value_type,
                );
                if res == 0 && !data_ptr.is_null() {
                    let conn = data_ptr as *const WLAN_CONNECTION_ATTRIBUTES;
                    let b = (*conn).wlanAssociationAttributes.dot11Bssid;
                    let bssid_str = format!(
                        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        b[0], b[1], b[2], b[3], b[4], b[5]
                    );
                    if bssid_str != "00:00:00:00:00:00" {
                        active_bssid = Some(bssid_str);
                    }
                    WlanFreeMemory(data_ptr);
                }
                if active_bssid.is_some() {
                    break;
                }
            }
        }

        WlanFreeMemory(interface_list as *mut _);
        WlanCloseHandle(handle, std::ptr::null_mut());
        active_bssid
    }
}

#[cfg(windows)]
fn get_current_wifi_bssid_netsh() -> Option<String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim().to_lowercase();
            let v = v.trim().to_string();
            if k == "bssid" && !v.is_empty() {
                return Some(v.to_uppercase());
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

#[cfg(unix)]
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

#[cfg(windows)]
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
            let _ = Command::new("netsh")
                .args(["wlan", "connect", &format!("name={}", wifi_con)])
                .output();
        }
        2 => {
            log("[Self-Healing L2] Cycling Wi-Fi connection...");
            let _ = Command::new("netsh").args(["wlan", "disconnect"]).output();
            sleep(Duration::from_secs(2));
            let _ = Command::new("netsh")
                .args(["wlan", "connect", &format!("name={}", wifi_con)])
                .output();
        }
        3 => {
            log("[Self-Healing L3] Re-engaging Wi-Fi network interface...");
            let _ = Command::new("netsh")
                .args(["interface", "set", "interface", "Wi-Fi", "disable"])
                .output();
            sleep(Duration::from_secs(2));
            let _ = Command::new("netsh")
                .args(["interface", "set", "interface", "Wi-Fi", "enable"])
                .output();
            sleep(Duration::from_secs(4));
            let _ = Command::new("netsh")
                .args(["wlan", "connect", &format!("name={}", wifi_con)])
                .output();
        }
        _ => {}
    }
}

#[cfg(unix)]
pub fn disable_wifi_powersave(ssid: &str) {
    // 1. Check if powersave is already disabled on this connection
    let needs_powersave = Command::new("nmcli")
        .args(["-f", "802-11-wireless.powersave", "connection", "show", ssid])
        .output()
        .map(|o| !String::from_utf8_lossy(&o.stdout).contains("2 (disable)"))
        .unwrap_or(true);

    if needs_powersave {
        let _ = Command::new("nmcli")
            .args(["connection", "modify", ssid, "802-11-wireless.powersave", "2"])
            .output();
    }

    // 2. Ensure Wi-Fi radio is on only if currently disabled
    let radio_disabled = Command::new("nmcli")
        .args(["radio", "wifi"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() != "enabled")
        .unwrap_or(false);

    if radio_disabled {
        let _ = Command::new("nmcli")
            .args(["radio", "wifi", "on"])
            .output();
    }
}

#[cfg(windows)]
pub fn disable_wifi_powersave(_ssid: &str) {
    // 1. Set Wireless Adapter Power Saving Mode to Maximum Performance (0) for AC and DC
    let _ = Command::new("powercfg")
        .args([
            "/SETACVALUEINDEX",
            "SCHEME_CURRENT",
            "19cbb8fa-5279-450e-9fac-8a3d5fedd0c1",
            "12bbebe6-58d6-4636-95bb-3217ef867c1a",
            "0",
        ])
        .output();
    let _ = Command::new("powercfg")
        .args([
            "/SETDCVALUEINDEX",
            "SCHEME_CURRENT",
            "19cbb8fa-5279-450e-9fac-8a3d5fedd0c1",
            "12bbebe6-58d6-4636-95bb-3217ef867c1a",
            "0",
        ])
        .output();
    let _ = Command::new("powercfg")
        .args(["/SETACTIVE", "SCHEME_CURRENT"])
        .output();

    // 2. Disable device sleep on disconnect on physical Wi-Fi adapters
    let script = "Get-NetAdapter -Physical -ErrorAction SilentlyContinue | \
                  Where-Object { $_.InterfaceDescription -like '*Wi-Fi*' -or $_.MediaType -eq 'Native 802.11' } | \
                  ForEach-Object { Set-NetAdapterPowerManagement -Name $_.Name -DeviceSleepOnDisconnect Disabled -WakeOnMagicPacket Disabled -ErrorAction SilentlyContinue }";
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", script])
        .output();
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
