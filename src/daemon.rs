use crate::config;
use crate::portal;
use crate::ui::{color, log, print_err, print_info, print_ok, print_warn, RED, YELLOW};
#[cfg(unix)]
use crate::ui::DIM;
use crate::wifi;
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

pub const KEEP_ALIVE_INTERVAL: u64 = 180;

pub fn get_keep_alive_interval() -> u64 {
    if let Ok(env_interval) = std::env::var("PESU_KEEPALIVE_INTERVAL") {
        if let Ok(v) = env_interval.parse() {
            return v;
        }
    }
    let cfg = config::load_config();
    if let Some(interval) = cfg.keep_alive_interval {
        if interval > 0 {
            return interval;
        }
    }
    KEEP_ALIVE_INTERVAL
}

pub fn calculate_jittered_interval(base_interval: u64) -> u64 {
    // ±5s anti-storm jitter using millisecond timestamp modulo
    let jitter = (portal::get_timestamp() % 11) as i64 - 5;
    (base_interval as i64 + jitter).max(10) as u64
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsEvent {
    NetworkChange(String),
    Timeout,
}

pub fn get_lock_file_path() -> std::path::PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime_dir.is_empty() {
            return std::path::PathBuf::from(runtime_dir).join("pesu_wifi_daemon.lock");
        }
    }
    config::get_config_dir().join("daemon.lock")
}

pub fn acquire_daemon_lock() -> File {
    let lock_path = get_lock_file_path();
    if let Some(parent) = lock_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to open lock file ({}): {}", lock_path.display(), e);
            std::process::exit(1);
        });

    if file.try_lock_exclusive().is_err() {
        print_warn("Another instance of pesu-wifi daemon is already running. Exiting.");
        std::process::exit(0);
    }

    // Write current PID to lock file
    use std::io::Write;
    let _ = file.set_len(0);
    let _ = writeln!(file, "{}", std::process::id());
    let _ = file.flush();

    file
}

pub fn is_daemon_running() -> (bool, Option<u32>) {
    let lock_path = get_lock_file_path();
    if let Ok(mut file) = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
    {
        if file.try_lock_exclusive().is_ok() {
            let _ = file.unlock();
            return (false, None);
        } else {
            // Lock is held; try reading PID from lockfile
            use std::io::Read;
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Ok(pid) = content.trim().parse::<u32>() {
                    return (true, Some(pid));
                }
            }
        }
    }

    // Lock was held: fallback to process table query
    #[cfg(unix)]
    if let Ok(out) = Command::new("pgrep")
        .args(["-f", "pesu[-_]wifi.*daemon"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let current_pid = std::process::id();
        for line in stdout.lines() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                if pid != current_pid {
                    return (true, Some(pid));
                }
            }
        }
    }

    #[cfg(windows)]
    if let Ok(out) = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Process -Name pesu-wifi -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let current_pid = std::process::id();
        for line in stdout.lines() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                if pid != current_pid {
                    return (true, Some(pid));
                }
            }
        }
    }

    (true, None)
}

#[cfg(unix)]
pub fn notify_desktop(title: &str, message: &str, urgency: &str) {
    let cfg = config::load_config();
    if !cfg.notifications {
        return;
    }
    if let Ok(env_val) = std::env::var("PESU_NOTIFICATIONS") {
        if env_val == "0" || env_val.eq_ignore_ascii_case("false") {
            return;
        }
    }
    let _ = Command::new("notify-send")
        .args(["-a", "PESU WiFi", "-i", "network-wireless", "-u", urgency, title, message])
        .output();
}

#[cfg(windows)]
pub fn notify_desktop(title: &str, message: &str, _urgency: &str) {
    let cfg = config::load_config();
    if !cfg.notifications {
        return;
    }
    if let Ok(env_val) = std::env::var("PESU_NOTIFICATIONS") {
        if env_val == "0" || env_val.eq_ignore_ascii_case("false") {
            return;
        }
    }
    let script = format!(
        "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
        $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
        $xml = [xml]$template.GetXml(); \
        $texts = $xml.GetElementsByTagName('text'); \
        $texts[0].AppendChild($xml.CreateTextNode('{}')) > $null; \
        $texts[1].AppendChild($xml.CreateTextNode('{}')) > $null; \
        $toast = [Windows.UI.Notifications.ToastNotification]::new($template); \
        [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('PESU WiFi').Show($toast);",
        title.replace('\'', "''"),
        message.replace('\'', "''")
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .output();
}

struct SleepInhibitor {
    #[cfg(unix)]
    child: Option<std::process::Child>,
    #[cfg(windows)]
    active: bool,
}

impl SleepInhibitor {
    fn new() -> Self {
        Self {
            #[cfg(unix)]
            child: None,
            #[cfg(windows)]
            active: false,
        }
    }

    fn activate(&mut self) {
        #[cfg(unix)]
        {
            if self.child.is_none() {
                if let Ok(child) = Command::new("systemd-inhibit")
                    .args([
                        "--what=idle:sleep",
                        "--who=PESU WiFi",
                        "--why=Continuous campus Wi-Fi keepalive and auto-login",
                        "--mode=block",
                        "sleep",
                        "infinity",
                    ])
                    .spawn()
                {
                    self.child = Some(child);
                    log("⚡ Power inhibitor active: system sleep & Wi-Fi power saving prevented.");
                }
            }
        }

        #[cfg(windows)]
        {
            if !self.active {
                unsafe {
                    windows_sys::Win32::System::Power::SetThreadExecutionState(
                        windows_sys::Win32::System::Power::ES_CONTINUOUS
                            | windows_sys::Win32::System::Power::ES_SYSTEM_REQUIRED
                            | windows_sys::Win32::System::Power::ES_AWAYMODE_REQUIRED,
                    );
                }
                self.active = true;
                log("⚡ Power inhibitor active: system sleep & Wi-Fi power saving prevented.");
            }
        }
    }

    fn deactivate(&mut self) {
        #[cfg(unix)]
        {
            if let Some(mut child) = self.child.take() {
                let _ = child.kill();
                let _ = child.wait();
                log("⚡ Power inhibitor released: normal system power profile restored.");
            }
        }

        #[cfg(windows)]
        {
            if self.active {
                unsafe {
                    windows_sys::Win32::System::Power::SetThreadExecutionState(
                        windows_sys::Win32::System::Power::ES_CONTINUOUS,
                    );
                }
                self.active = false;
                log("⚡ Power inhibitor released: normal system power profile restored.");
            }
        }
    }
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) {
        self.deactivate();
    }
}

#[cfg(unix)]
pub struct OsNetworkListener {
    fd: std::os::unix::io::RawFd,
}

#[cfg(unix)]
impl OsNetworkListener {
    pub fn new() -> Result<Self, String> {
        let fd = unsafe {
            libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                libc::NETLINK_ROUTE,
            )
        };
        if fd < 0 {
            return Err(format!(
                "Failed to open Netlink route socket: {}",
                std::io::Error::last_os_error()
            ));
        }

        let mut sa: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        sa.nl_family = libc::AF_NETLINK as libc::sa_family_t;
        // Multicast groups:
        // RTMGRP_LINK (0x01): Interface up/down, carrier on/off
        // RTMGRP_IPV4_IFADDR (0x10): IPv4 address assigned / renewed via DHCP
        // RTMGRP_NOTIFY (0x02): Routing / link notifications
        sa.nl_groups = (libc::RTMGRP_LINK | libc::RTMGRP_IPV4_IFADDR | libc::RTMGRP_NOTIFY) as u32;

        let ret = unsafe {
            libc::bind(
                fd,
                &sa as *const libc::sockaddr_nl as *const libc::sockaddr,
                std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
            )
        };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            unsafe { libc::close(fd); }
            return Err(format!("Failed to bind Netlink socket to multicast groups: {}", err));
        }

        Ok(Self { fd })
    }

    pub fn wait_event(&self, timeout: Option<Duration>) -> Result<OsEvent, String> {
        let mut pfd = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let timeout_ms = match timeout {
            Some(d) => d.as_millis().min(i32::MAX as u128) as libc::c_int,
            None => -1,
        };

        let ret = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                return Ok(OsEvent::Timeout);
            }
            return Err(format!("Netlink poll error: {}", err));
        } else if ret == 0 {
            return Ok(OsEvent::Timeout);
        }

        let mut buf = [0u8; 4096];
        let n = unsafe {
            libc::recv(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0)
        };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(OsEvent::Timeout);
            }
            return Err(format!("Netlink recv error: {}", err));
        }

        // Debounce: wait 250ms and drain any buffered burst messages from the same state transition
        sleep(Duration::from_millis(250));
        while unsafe {
            libc::recv(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                libc::MSG_DONTWAIT,
            )
        } > 0 {}

        Ok(OsEvent::NetworkChange("Netlink (rtnetlink link/addr event)".to_string()))
    }
}

#[cfg(unix)]
impl Drop for OsNetworkListener {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

#[cfg(windows)]
pub struct OsNetworkListener {
    rx: std::sync::mpsc::Receiver<String>,
    tx_ptr: *mut std::sync::mpsc::SyncSender<String>,
    handle: *mut std::ffi::c_void,
}

#[cfg(windows)]
unsafe impl Send for OsNetworkListener {}

#[cfg(windows)]
unsafe extern "system" fn wlan_notification_callback(
    pdata: *mut windows_sys::Win32::NetworkManagement::WiFi::L2_NOTIFICATION_DATA,
    pcontext: *mut core::ffi::c_void,
) {
    if pdata.is_null() || pcontext.is_null() {
        return;
    }
    let sender = &*(pcontext as *const std::sync::mpsc::SyncSender<String>);
    let code = (*pdata).NotificationCode;
    let source = (*pdata).NotificationSource;

    let desc = match (source, code) {
        (windows_sys::Win32::NetworkManagement::WiFi::WLAN_NOTIFICATION_SOURCE_ACM, 10) => "ACM:ConnectionComplete",
        (windows_sys::Win32::NetworkManagement::WiFi::WLAN_NOTIFICATION_SOURCE_ACM, 21) => "ACM:Disconnected",
        (windows_sys::Win32::NetworkManagement::WiFi::WLAN_NOTIFICATION_SOURCE_ACM, 13) => "ACM:InterfaceArrival",
        (windows_sys::Win32::NetworkManagement::WiFi::WLAN_NOTIFICATION_SOURCE_ACM, 14) => "ACM:InterfaceRemoval",
        (windows_sys::Win32::NetworkManagement::WiFi::WLAN_NOTIFICATION_SOURCE_MSM, 4) => "MSM:Connected",
        (windows_sys::Win32::NetworkManagement::WiFi::WLAN_NOTIFICATION_SOURCE_MSM, 10) => "MSM:Disconnected",
        (windows_sys::Win32::NetworkManagement::WiFi::WLAN_NOTIFICATION_SOURCE_MSM, 5) => "MSM:RoamingStart",
        (windows_sys::Win32::NetworkManagement::WiFi::WLAN_NOTIFICATION_SOURCE_MSM, 6) => "MSM:RoamingEnd",
        _ => "WlanNotification",
    };
    let _ = sender.try_send(desc.to_string());
}

#[cfg(windows)]
impl OsNetworkListener {
    pub fn new() -> Result<Self, String> {
        use windows_sys::Win32::NetworkManagement::WiFi::*;
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(64);
        let tx_ptr = Box::into_raw(Box::new(tx));

        unsafe {
            let mut client_version = 0;
            let mut handle = std::ptr::null_mut();
            let res = WlanOpenHandle(2, std::ptr::null(), &mut client_version, &mut handle);
            if res != 0 || handle.is_null() {
                let _ = Box::from_raw(tx_ptr);
                return Err(format!("WlanOpenHandle failed with error code {}", res));
            }

            let mut prev_source = 0;
            let reg_res = WlanRegisterNotification(
                handle,
                WLAN_NOTIFICATION_SOURCE_ALL,
                1,
                Some(wlan_notification_callback),
                tx_ptr as *const core::ffi::c_void,
                std::ptr::null(),
                &mut prev_source,
            );
            if reg_res != 0 {
                WlanCloseHandle(handle, std::ptr::null_mut());
                let _ = Box::from_raw(tx_ptr);
                return Err(format!("WlanRegisterNotification failed with error code {}", reg_res));
            }

            Ok(Self { rx, tx_ptr, handle })
        }
    }

    pub fn wait_event(&self, timeout: Option<Duration>) -> Result<OsEvent, String> {
        let res = match timeout {
            Some(d) => match self.rx.recv_timeout(d) {
                Ok(ev) => Ok(OsEvent::NetworkChange(ev)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(OsEvent::Timeout),
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    Err("Wlan notification channel disconnected".to_string())
                }
            },
            None => match self.rx.recv() {
                Ok(ev) => Ok(OsEvent::NetworkChange(ev)),
                Err(e) => Err(e.to_string()),
            },
        };

        if let Ok(OsEvent::NetworkChange(_)) = res {
            // Debounce burst notifications
            sleep(Duration::from_millis(300));
            while self.rx.try_recv().is_ok() {}
        }

        res
    }
}

#[cfg(windows)]
impl Drop for OsNetworkListener {
    fn drop(&mut self) {
        use windows_sys::Win32::NetworkManagement::WiFi::*;
        unsafe {
            let mut prev_source = 0;
            let _ = WlanRegisterNotification(
                self.handle,
                WLAN_NOTIFICATION_SOURCE_NONE,
                1,
                None,
                std::ptr::null(),
                std::ptr::null(),
                &mut prev_source,
            );
            WlanCloseHandle(self.handle, std::ptr::null_mut());
            if !self.tx_ptr.is_null() {
                let _ = Box::from_raw(self.tx_ptr);
            }
        }
    }
}

pub fn run_daemon() -> ! {
    let _lock = acquire_daemon_lock();

    let (mut username, mut password) = match config::get_active_credentials() {
        (Some(u), Some(p)) => (u, p),
        _ => {
            log(&format!(
                "{}",
                color(RED, "Error: No credentials saved. Run 'pesu-wifi add' first.")
            ));
            std::process::exit(1);
        }
    };

    let interval = get_keep_alive_interval();

    #[cfg(unix)]
    let os_engine = "Linux Netlink (rtnetlink RTMGRP_LINK | RTMGRP_IPV4_IFADDR)";
    #[cfg(windows)]
    let os_engine = "Win32 WlanRegisterNotification (ACM/MSM events)";

    log(&format!(
        "Starting Always-On Event-Driven Keepalive Watchdog for '{}'...",
        username
    ));
    log(&format!("⚡ OS System Call Engine: {}", os_engine));
    log(&format!(
        "   Architecture: Reactive OS kernel notifications (0ms event wakeup) + {}s keepalive heartbeat (anti-disconnect).",
        interval
    ));

    let listener = OsNetworkListener::new().unwrap_or_else(|err| {
        log(&format!("⚠ Failed to initialize OS network listener: {}. Exiting.", err));
        std::process::exit(1);
    });

    let mut last_bssid: Option<String> = None;
    let mut last_standby_state: Option<String> = None;
    let mut inhibitor = SleepInhibitor::new();
    let mut is_first_run = true;
    let mut next_wait: Option<Duration> = None;
    let mut unreachable_streak = 0;
    let mut consecutive_session_drops = 0;
    let mut just_connected = true;

    loop {
        // Step 1: Wait for OS Network Event or Keepalive Watchdog Timeout
        if !is_first_run {
            match listener.wait_event(next_wait) {
                Ok(OsEvent::NetworkChange(ev)) => {
                    log(&format!("⚡ OS Network Event: {} [evaluating immediately]", ev));
                    just_connected = true;
                }
                Ok(OsEvent::Timeout) => {
                    // Keepalive watchdog interval elapsed (or standby poll timeout)
                }
                Err(e) => {
                    log(&format!("⚠ OS event listener error: {}. Retrying in 5s...", e));
                    sleep(Duration::from_secs(5));
                    continue;
                }
            }
        } else {
            is_first_run = false;
            log("⚡ Initial link state evaluation on daemon startup...");
        }

        // Step 2: Refresh credentials dynamically if user updated config
        if let (Some(u), Some(p)) = config::get_active_credentials() {
            username = u;
            password = p;
        }

        let current_ssid = wifi::get_current_wifi_ssid();
        let current_bssid = wifi::get_current_wifi_bssid();

        // Step 3: Evaluate OS Wi-Fi System Call Rules
        // Non-campus SSID or Disconnected Interface
        if let Some(ref ssid) = current_ssid {
            if !wifi::is_campus_ssid(ssid) {
                let state_str = format!("off-campus:{}", ssid);
                if last_standby_state.as_deref() != Some(&state_str) {
                    log(&format!(
                        "Connected to non-campus Wi-Fi '{}'. Daemon in standby.",
                        ssid
                    ));
                    last_standby_state = Some(state_str);
                }
                inhibitor.deactivate();
                last_bssid = None;
                unreachable_streak = 0;
                consecutive_session_drops = 0;
                just_connected = true;
                next_wait = Some(Duration::from_secs(10));
                continue;
            }
        } else {
            if last_standby_state.as_deref() != Some("disconnected") {
                log("Wi-Fi disconnected. Daemon sleeping in kernel wait for interface reconnect...");
                last_standby_state = Some("disconnected".to_string());
            }
            inhibitor.deactivate();
            last_bssid = None;
            unreachable_streak = 0;
            consecutive_session_drops = 0;
            just_connected = true;
            next_wait = Some(Duration::from_secs(10));
            continue;
        }

        // Campus Network Confirmed
        let ssid = current_ssid.unwrap();
        last_standby_state = None;

        // Rule 1: Power-Saving & Sleep Immunity System Calls
        if just_connected {
            wifi::disable_wifi_powersave(&ssid);
        }
        inhibitor.activate();

        // Rule 2: AP Roaming & BSSID Handover Verification
        if let Some(ref bssid) = current_bssid {
            if let Some(ref prev) = last_bssid {
                if prev != bssid {
                    log(&format!(
                        "⚡ AP Roaming detected: {} ➔ {}. Verifying gateway link...",
                        prev, bssid
                    ));
                    just_connected = true;
                }
            }
            last_bssid = Some(bssid.clone());
        }

        let interval = get_keep_alive_interval();

        // Rule 3: Session Keepalive Watchdog & Auto-Authentication
        // Primary check: Check /live heartbeat directly.
        // /live is a lightweight ~35-byte XML query that keeps the captive portal session alive
        // and verifies that our IP is still authorized by the gateway.
        if portal::check_live(Some(&username), false) {
            if unreachable_streak > 0 {
                log(&format!(
                    "✔ Connectivity restored after {} failed check(s).",
                    unreachable_streak
                ));
                unreachable_streak = 0;
            }
            consecutive_session_drops = 0;
            just_connected = false;
            let jittered = calculate_jittered_interval(interval);
            log(&format!(
                "✔ Keepalive heartbeat OK ({}). Session active. Next check in {}s.",
                username, jittered
            ));
            next_wait = Some(Duration::from_secs(jittered));
            continue;
        }

        // Session check failed: verify whether the portal gateway itself is reachable
        if !portal::is_portal_online() {
            unreachable_streak += 1;
            consecutive_session_drops = 0;
            log(&format!(
                "⚠ Portal gateway unreachable on '{}' (streak: {}).",
                ssid, unreachable_streak
            ));

            match unreachable_streak {
                3 => {
                    wifi::heal_network(1);
                    next_wait = Some(Duration::from_secs(10));
                }
                5 | 6 => {
                    wifi::heal_network(2);
                    next_wait = Some(Duration::from_secs(10));
                }
                s if s >= 9 => {
                    wifi::heal_network(3);
                    next_wait = Some(Duration::from_secs(15));
                    unreachable_streak = 4;
                }
                _ => {
                    next_wait = Some(Duration::from_secs(5));
                }
            }
            continue;
        }

        // Gateway is online, but session expired or dropped (or newly connected)
        if unreachable_streak > 0 {
            log(&format!(
                "✔ Connectivity restored after {} failed check(s).",
                unreachable_streak
            ));
            unreachable_streak = 0;
        }

        // Only wait for jitter confirmation if we had an established session and didn't just connect/roam
        if !just_connected {
            consecutive_session_drops += 1;
            if consecutive_session_drops < 2 {
                log("⚠ Keepalive missed 1 check (possible Wi-Fi jitter). Verifying in 3s before re-authenticating...");
                next_wait = Some(Duration::from_secs(3));
                continue;
            }
        }

        consecutive_session_drops = 0;
        just_connected = false;
        log(&format!(
            "⚡ Keepalive watchdog: session expired. Authenticating as '{}'...",
            username
        ));
        match portal::do_login(&username, &password) {
            Ok(_) => {
                let jittered = calculate_jittered_interval(interval);
                log(&format!(
                    "✔ Authenticated successfully as '{}'. Next keepalive check in {}s.",
                    username, jittered
                ));
                notify_desktop(
                    "PESU WiFi",
                    &format!("Logged in as {}", username),
                    "normal",
                );
                next_wait = Some(Duration::from_secs(jittered));
            }
            Err(err) => {
                log(&format!("✖ Authentication failed: {}", err));
                notify_desktop("PESU WiFi Login Failed", &err, "critical");
                next_wait = Some(Duration::from_secs(10));
            }
        }
    }
}

#[cfg(unix)]
pub fn cmd_start(foreground: bool) -> i32 {
    let (username, password) = config::get_active_credentials();
    if username.is_none() || password.is_none() {
        print_err("No credentials configured.");
        println!("{}", color(YELLOW, "  Run 'pesu-wifi add' first to save credentials."));
        return 1;
    }

    if foreground {
        run_daemon();
    }

    let (daemon_active, daemon_pid) = is_daemon_running();
    if daemon_active {
        let pid_str = daemon_pid
            .map(|p| format!(" (PID: {})", p))
            .unwrap_or_default();
        print_ok(&format!("PESU WiFi daemon is already running{}.", pid_str));
        return 0;
    }

    print_info("Starting PESU WiFi daemon (systemd user service)...");
    let res = Command::new("systemctl")
        .args(["--user", "start", "pesu-wifi.service"])
        .output();

    match res {
        Ok(out) if out.status.success() => {
            sleep(Duration::from_millis(600));
            let (active, pid) = is_daemon_running();
            if active {
                let pid_str = pid.map(|p| format!(" (PID: {})", p)).unwrap_or_default();
                print_ok(&format!("PESU WiFi daemon started successfully{}.", pid_str));
                0
            } else {
                print_warn("Daemon attempted to start but may have exited.");
                if let Ok(log_res) = Command::new("journalctl")
                    .args(["--user", "-u", "pesu-wifi.service", "-n", "3", "--no-pager"])
                    .output()
                {
                    let log_txt = String::from_utf8_lossy(&log_res.stdout);
                    if !log_txt.trim().is_empty() {
                        println!("{}", color(DIM, log_txt.trim()));
                    }
                }
                1
            }
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let msg = if !err.is_empty() {
                err
            } else {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            };
            print_err(&format!("Failed to start systemd service: {}", msg));
            1
        }
        Err(e) => {
            print_err(&format!("Error starting daemon: {}", e));
            1
        }
    }
}

#[cfg(windows)]
pub fn cmd_start(foreground: bool) -> i32 {
    let (username, password) = config::get_active_credentials();
    if username.is_none() || password.is_none() {
        print_err("No credentials configured.");
        println!("{}", color(YELLOW, "  Run 'pesu-wifi add' first to save credentials."));
        return 1;
    }

    if foreground {
        run_daemon();
    }

    let (daemon_active, daemon_pid) = is_daemon_running();
    if daemon_active {
        let pid_str = daemon_pid
            .map(|p| format!(" (PID: {})", p))
            .unwrap_or_default();
        print_ok(&format!("PESU WiFi continuous daemon is already running{}.", pid_str));
        return 0;
    }

    print_info("Starting PESU WiFi continuous daemon in background...");
    let exe = match std::env::current_exe() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => "pesu-wifi.exe".to_string(),
    };

    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new(&exe);
    cmd.arg("daemon");
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    match cmd.spawn() {
        Ok(_) => {
            sleep(Duration::from_millis(600));
            let (active, pid) = is_daemon_running();
            if active {
                let pid_str = pid.map(|p| format!(" (PID: {})", p)).unwrap_or_default();
                print_ok(&format!("PESU WiFi continuous daemon started successfully{}.", pid_str));
                0
            } else {
                print_warn("Daemon attempted to start but may have exited.");
                1
            }
        }
        Err(e) => {
            print_err(&format!("Error starting daemon: {}", e));
            1
        }
    }
}

#[cfg(unix)]
pub fn cmd_stop() -> i32 {
    print_info("Stopping PESU WiFi daemon...");

    let _ = Command::new("systemctl")
        .args(["--user", "stop", "pesu-wifi.service"])
        .output();

    let _ = Command::new("pkill")
        .args(["-f", "pesu[-_]wifi.*daemon"])
        .output();

    sleep(Duration::from_millis(500));
    let (still_active, _) = is_daemon_running();
    if !still_active {
        print_ok("PESU WiFi daemon stopped.");
    } else {
        print_warn("Daemon process may still be stopping.");
    }
    0
}

#[cfg(windows)]
pub fn cmd_stop() -> i32 {
    print_info("Stopping PESU WiFi daemon...");

    let current_pid = std::process::id();
    let script = format!(
        "Get-CimInstance Win32_Process -Filter \"Name like 'pesu%wifi%'\" | Where-Object {{ $_.ProcessId -ne {} -and $_.CommandLine -like '*daemon*' }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force }}",
        current_pid
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .output();

    sleep(Duration::from_millis(500));
    let (still_active, _) = is_daemon_running();
    if !still_active {
        print_ok("PESU WiFi daemon stopped.");
    } else {
        print_warn("Daemon process may still be stopping.");
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keep_alive_interval_default() {
        std::env::remove_var("PESU_KEEPALIVE_INTERVAL");
        assert_eq!(get_keep_alive_interval(), 180);
    }

    #[test]
    fn test_keep_alive_interval_env_override() {
        std::env::set_var("PESU_KEEPALIVE_INTERVAL", "120");
        assert_eq!(get_keep_alive_interval(), 120);
        std::env::remove_var("PESU_KEEPALIVE_INTERVAL");
    }

    #[test]
    fn test_calculate_jittered_interval() {
        let val = calculate_jittered_interval(180);
        assert!(val >= 175 && val <= 185);
    }

    #[test]
    fn test_lock_file_path_resolution() {
        let path = get_lock_file_path();
        assert!(path.to_string_lossy().contains("daemon.lock") || path.to_string_lossy().contains("pesu_wifi_daemon.lock"));
    }

    #[test]
    #[cfg(unix)]
    fn test_os_network_listener_creation() {
        let listener = OsNetworkListener::new();
        assert!(listener.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_os_network_listener_wait_event_timeout() {
        let listener = OsNetworkListener::new().expect("Failed to create listener");
        let start = std::time::Instant::now();
        let res = listener.wait_event(Some(Duration::from_millis(50)));
        assert_eq!(res, Ok(OsEvent::Timeout));
        assert!(start.elapsed() >= Duration::from_millis(45));
    }
}

