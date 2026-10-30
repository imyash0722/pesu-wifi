use crate::config;
use crate::ui::log;
use std::process::Command;

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

#[cfg(unix)]
pub fn get_current_wifi_signal() -> Option<i32> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "active,signal", "dev", "wifi"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.starts_with("yes:") {
            let sig_str = line.trim_start_matches("yes:").trim();
            if let Ok(sig) = sig_str.parse::<i32>() {
                return Some(sig);
            }
        }
    }
    None
}

#[cfg(unix)]
pub fn scan_and_find_stronger_campus_ap(
    _current_ssid: &str,
    current_bssid: Option<&str>,
    current_signal: i32,
) -> Option<(String, i32)> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "active,bssid,signal,ssid", "dev", "wifi"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut best_candidate: Option<(String, i32)> = None;

    for line in text.lines() {
        let unescaped = line.replace("\\:", "__COLON__");
        let fields: Vec<&str> = unescaped.split(':').collect();
        if fields.len() >= 4 {
            let bssid = fields[1].replace("__COLON__", ":").trim().to_uppercase();
            let signal = fields[2].trim().parse::<i32>().unwrap_or(0);
            let ssid = fields[3..].join(":").replace("__COLON__", ":").trim().to_string();

            if is_campus_ssid(&ssid) {
                if let Some(curr_b) = current_bssid {
                    if curr_b.eq_ignore_ascii_case(&bssid) {
                        continue;
                    }
                }
                if signal >= current_signal + 15 && signal >= 50 {
                    if let Some((_, best_sig)) = best_candidate {
                        if signal > best_sig {
                            best_candidate = Some((bssid, signal));
                        }
                    } else {
                        best_candidate = Some((bssid, signal));
                    }
                }
            }
        }
    }

    best_candidate
}

#[cfg(unix)]
pub fn steer_to_bssid(ssid: &str, target_bssid: &str) -> bool {
    let output = Command::new("nmcli")
        .args(["dev", "wifi", "connect", ssid, "bssid", target_bssid])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                log(&format!(
                    "⚡ Successfully steered Wi-Fi association to router '{}' on '{}'",
                    target_bssid, ssid
                ));
                true
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                log(&format!(
                    "⚠ nmcli steer to bssid '{}' returned: {}",
                    target_bssid, err.trim()
                ));
                false
            }
        }
        Err(e) => {
            log(&format!("⚠ Failed executing nmcli steer: {}", e));
            false
        }
    }
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
pub fn get_current_wifi_signal() -> Option<i32> {
    if let Some(sig) = get_current_wifi_signal_wlanapi() {
        return Some(sig);
    }
    get_current_wifi_signal_netsh()
}

#[cfg(windows)]
fn get_current_wifi_signal_wlanapi() -> Option<i32> {
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

        let mut active_signal = None;
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
                    let signal = (*conn).wlanAssociationAttributes.wlanSignalQuality as i32;
                    active_signal = Some(signal);
                    WlanFreeMemory(data_ptr);
                    break;
                }
            }
        }

        WlanFreeMemory(interface_list as *mut std::ffi::c_void);
        WlanCloseHandle(handle, std::ptr::null_mut());
        active_signal
    }
}

#[cfg(windows)]
fn get_current_wifi_signal_netsh() -> Option<i32> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("signal") && line.contains(':') {
            let val = line.split(':').nth(1)?.trim().trim_end_matches('%').trim();
            if let Ok(sig) = val.parse::<i32>() {
                return Some(sig);
            }
        }
    }
    None
}

#[cfg(windows)]
pub fn scan_and_find_stronger_campus_ap(
    _current_ssid: &str,
    current_bssid: Option<&str>,
    current_signal: i32,
) -> Option<(String, i32)> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "networks", "mode=bssid"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut current_parsed_ssid = String::new();
    let mut current_parsed_bssid = String::new();
    let mut best_candidate: Option<(String, i32)> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.to_lowercase().starts_with("ssid ") && trimmed.contains(':') {
            if let Some(s) = trimmed.split(':').nth(1) {
                current_parsed_ssid = s.trim().to_string();
            }
        } else if trimmed.to_lowercase().starts_with("bssid ") && trimmed.contains(':') {
            if let Some(b) = trimmed.split(':').nth(1) {
                current_parsed_bssid = b.trim().to_uppercase();
            }
        } else if trimmed.to_lowercase().starts_with("signal") && trimmed.contains(':') {
            if let Some(sig_str) = trimmed.split(':').nth(1) {
                let sig = sig_str.trim().trim_end_matches('%').trim().parse::<i32>().unwrap_or(0);
                if is_campus_ssid(&current_parsed_ssid) && !current_parsed_bssid.is_empty() {
                    if let Some(curr_b) = current_bssid {
                        if curr_b.eq_ignore_ascii_case(&current_parsed_bssid) {
                            continue;
                        }
                    }
                    if sig >= current_signal + 15 && sig >= 50 {
                        if let Some((_, best_sig)) = best_candidate {
                            if sig > best_sig {
                                best_candidate = Some((current_parsed_bssid.clone(), sig));
                            }
                        } else {
                            best_candidate = Some((current_parsed_bssid.clone(), sig));
                        }
                    }
                }
            }
        }
    }

    best_candidate
}

#[cfg(windows)]
pub fn steer_to_bssid(ssid: &str, _target_bssid: &str) -> bool {
    let res = Command::new("netsh")
        .args(["wlan", "connect", &format!("name={}", ssid), &format!("ssid={}", ssid)])
        .output();
    match res {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
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
        || s.contains("AMAATRA")
        || s.contains("FOODCOURT")
        || s.contains("PES SOUTH CAFE")
        || s.contains("PESU-CIE")
}

pub fn get_active_wifi_ssid() -> String {
    if let Some(curr) = get_current_wifi_ssid() {
        return curr;
    }
    let cfg = config::load_config();
    cfg.preferred_ssid.unwrap_or_else(get_default_wifi_con)
}

#[cfg(unix)]
#[allow(dead_code)]
pub fn is_vpn_active() -> bool {
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("tun")
                || name.starts_with("tap")
                || name.starts_with("wg")
                || name.starts_with("tailscale")
                || name.starts_with("proton")
                || name.starts_with("wireguard")
            {
                let operstate = std::fs::read_to_string(entry.path().join("operstate"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if operstate != "down" {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn is_vpn_active() -> bool {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object { ($_.InterfaceDescription -like '*WireGuard*' -or $_.InterfaceDescription -like '*Wintun*' -or $_.InterfaceDescription -like '*TAP*' -or $_.InterfaceDescription -like '*Tailscale*' -or $_.InterfaceAlias -like '*Tailscale*') -and $_.Status -eq 'Up' } | Measure-Object).Count",
        ])
        .output()
        .ok();
    if let Some(out) = output {
        let count_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if let Ok(count) = count_str.parse::<usize>() {
            return count > 0;
        }
    }
    false
}

#[cfg(unix)]
pub fn heal_network(_tier: u8) {
    let wifi_con = get_current_wifi_ssid().unwrap_or_else(get_active_wifi_ssid);
    if !is_campus_ssid(&wifi_con) {
        log(&format!(
            "[Self-Healing] Skipped: '{}' is not a PESU campus network.",
            wifi_con
        ));
        return;
    }

    log(&format!(
        "[Self-Healing] Refreshing persistent routing rules & host routes on '{}' (zero-disconnect, no port switching)...",
        wifi_con
    ));
    ensure_vpn_bypass_routes(&wifi_con);
    if let Some(iface) = crate::portal::get_wifi_interface_name() {
        let _ = Command::new("nmcli").args(["device", "reapply", &iface]).output();
    }
}

#[cfg(windows)]
pub fn heal_network(_tier: u8) {
    let wifi_con = get_current_wifi_ssid().unwrap_or_else(get_active_wifi_ssid);
    if !is_campus_ssid(&wifi_con) {
        log(&format!(
            "[Self-Healing] Skipped: '{}' is not a PESU campus network.",
            wifi_con
        ));
        return;
    }

    log(&format!(
        "[Self-Healing] Refreshing persistent routing rules & host routes on '{}' (zero-disconnect, no port switching)...",
        wifi_con
    ));
    ensure_vpn_bypass_routes(&wifi_con);
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

#[cfg(unix)]
pub fn ensure_vpn_bypass_routes(ssid: &str) {
    if !is_campus_ssid(ssid) {
        return;
    }

    let gw = match crate::portal::get_default_gateway_ip() {
        Some(ip) => ip,
        None => return,
    };

    let portal_ips = ["192.168.254.1/32", "172.16.1.1/32", "192.168.1.1/32", "10.14.142.1/32"];

    let existing_routes = Command::new("nmcli")
        .args(["-g", "ipv4.routes", "connection", "show", ssid])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut routes_modified = false;

    // Purge any stale static gateway routes from the persistent NetworkManager profile.
    // Hardcoded gateway IPs (e.g. 10.1.0.1) break when roaming across campus subnets (e.g. to 10.3.0.0/20).
    if !existing_routes.trim().is_empty() {
        let mut has_portal_route = false;
        for target in portal_ips {
            let ip_target = target.trim_end_matches("/32");
            if existing_routes.contains(ip_target) {
                has_portal_route = true;
                break;
            }
        }
        if has_portal_route {
            let res = Command::new("nmcli")
                .args(["connection", "modify", ssid, "ipv4.routes", ""])
                .output();
            if let Ok(out) = res {
                if out.status.success() {
                    routes_modified = true;
                    log(&format!(
                        "⚡ Purged stale static gateway routes from connection profile '{}' (prevents subnet roaming lockouts)",
                        ssid
                    ));
                }
            }
        }
    }

    // Direct NetworkManager routing rules for Tailscale / WireGuard compatibility:
    let existing_nm_rules = Command::new("nmcli")
        .args(["-g", "ipv4.routing-rules", "connection", "show", ssid])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    for target in portal_ips {
        let ip_target = target.trim_end_matches("/32");
        if !existing_nm_rules.contains(ip_target) {
            let rule_entry = format!("priority 5200 to {} table main", ip_target);
            let res = Command::new("nmcli")
                .args(["connection", "modify", ssid, "+ipv4.routing-rules", &rule_entry])
                .output();
            if let Ok(out) = res {
                if out.status.success() {
                    routes_modified = true;
                    log(&format!(
                        "⚡ Configured VPN bypass policy rule for {} (priority 5200 -> main) on '{}'",
                        ip_target, ssid
                    ));
                }
            }
        }
    }

    // Direct kernel route insertion & live reapply without dropping connection:
    let iface_opt = crate::portal::get_wifi_interface_name();
    if let Some(ref iface) = iface_opt {
        // 1. Instantly inject kernel routes into main table (zero overhead, no reconnect needed)
        for target in portal_ips {
            let ip_target = target.trim_end_matches("/32");
            let _ = Command::new("ip")
                .args(["route", "replace", ip_target, "via", &gw, "dev", iface])
                .output();
        }

        // 2. Policy routing rule for Tailscale / WireGuard compatibility:
        // Tailscale uses priority 5270 (lookup 52). Add rule at priority 5200 to look up main table for portal IPs!
        let existing_rules = Command::new("ip")
            .args(["rule", "show"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        for target in portal_ips {
            let ip_target = target.trim_end_matches("/32");
            if !existing_rules.contains(&format!("to {}", ip_target)) {
                let _ = Command::new("ip")
                    .args(["rule", "add", "to", ip_target, "lookup", "main", "pref", "5200"])
                    .output();
            }
        }

        // 3. If NetworkManager profile was modified, reapply live without disconnecting Wi-Fi
        if routes_modified {
            let _ = Command::new("nmcli")
                .args(["device", "reapply", iface])
                .output();
            log(&format!("⚡ Dynamically reapplied routes to live device '{}' (0ms downtime)", iface));
        }
    }
}

#[cfg(windows)]
pub fn ensure_vpn_bypass_routes(ssid: &str) {
    if !is_campus_ssid(ssid) {
        return;
    }

    let gw = match crate::portal::get_default_gateway_ip() {
        Some(ip) => ip,
        None => return,
    };

    let targets = ["192.168.254.1", "172.16.1.1", "192.168.1.1", "10.14.142.1"];
    for target in targets {
        let _ = Command::new("route")
            .args(["add", target, "mask", "255.255.255.255", &gw, "metric", "1"])
            .output();
    }

    let script = format!(
        r#"
        $adapter = Get-NetAdapter -Physical -ErrorAction SilentlyContinue | Where-Object {{ $_.MediaType -eq 'Native 802.11' -or $_.InterfaceDescription -like '*Wi-Fi*' -or $_.Name -like '*Wi-Fi*' }} | Select-Object -First 1
        if ($adapter) {{
            $idx = $adapter.ifIndex
            $targets = @('192.168.254.1/32', '172.16.1.1/32', '192.168.1.1/32', '10.14.142.1/32')
            foreach ($tgt in $targets) {{
                $existing = Get-NetRoute -DestinationPrefix $tgt -InterfaceIndex $idx -ErrorAction SilentlyContinue
                if (-not $existing) {{
                    New-NetRoute -DestinationPrefix $tgt -InterfaceIndex $idx -NextHop '{gw}' -RouteMetric 1 -PolicyStore ActiveStore -ErrorAction SilentlyContinue | Out-Null
                }}
            }}
        }}
        "#,
        gw = gw
    );

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
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
        assert!(is_campus_ssid("AMAATRA_HOSTEL"));
        assert!(is_campus_ssid("AMAATRA-HOSTEL"));
        assert!(is_campus_ssid("Foodcourt"));
        assert!(is_campus_ssid("pes south cafe"));
        assert!(is_campus_ssid("PESU-CIE"));
        assert!(!is_campus_ssid("Home_Network"));
        assert!(!is_campus_ssid("Space"));
        assert!(!is_campus_ssid(""));
    }

    #[test]
    fn test_ensure_vpn_bypass_routes_non_campus() {
        // Non-campus networks must safely do nothing
        ensure_vpn_bypass_routes("Home_Network");
        ensure_vpn_bypass_routes("");
    }

    #[test]
    fn test_is_vpn_active() {
        let _ = is_vpn_active();
    }

    #[test]
    fn test_get_current_wifi_signal() {
        let _ = get_current_wifi_signal();
    }

    #[test]
    fn test_scan_and_find_stronger_campus_ap() {
        let _ = scan_and_find_stronger_campus_ap("PESU-EC-Campus", None, 30);
    }
}
