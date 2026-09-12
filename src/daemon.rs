use crate::config;
use crate::portal;
use crate::ui::{color, log, print_err, print_info, print_ok, print_warn, DIM, RED, YELLOW};
use crate::wifi;
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

pub const LOCK_FILE: &str = "/tmp/pesu_wifi_daemon.lock";
pub const KEEP_ALIVE_INTERVAL: u64 = 60;

pub fn acquire_daemon_lock() -> File {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(LOCK_FILE)
        .unwrap_or_else(|e| {
            eprintln!("Failed to open lock file: {}", e);
            std::process::exit(1);
        });

    if file.try_lock_exclusive().is_err() {
        print_warn("Another instance of pesu-wifi daemon is already running. Exiting.");
        std::process::exit(0);
    }

    file
}

pub fn is_daemon_running() -> (bool, Option<u32>) {
    if let Ok(file) = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(LOCK_FILE)
    {
        if file.try_lock_exclusive().is_ok() {
            let _ = file.unlock();
            return (false, None);
        }
    }

    // Lock was held or pgrep
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

    (true, None)
}

pub fn notify_desktop(title: &str, message: &str, urgency: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "PESU WiFi", "-u", urgency, title, message])
        .output();
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

    log(&format!(
        "Starting keepalive watchdog for '{}' (interval: {}s)...",
        username, KEEP_ALIVE_INTERVAL
    ));

    let mut unreachable_streak = 0;
    let mut consecutive_session_drops = 0;
    let mut last_standby_state: Option<String> = None;

    loop {
        if let (Some(u), Some(p)) = config::get_active_credentials() {
            username = u;
            password = p;
        }

        let current_ssid = wifi::get_current_wifi_ssid();

        // 0. Check if connected to a campus network
        if let Some(ref ssid) = current_ssid {
            if !wifi::is_campus_ssid(ssid) {
                let state_str = format!("off-campus:{}", ssid);
                if last_standby_state.as_deref() != Some(&state_str) {
                    log(&format!(
                        "Connected to non-campus Wi-Fi '{}'. Watchdog in standby (polling in {}s)...",
                        ssid, KEEP_ALIVE_INTERVAL
                    ));
                    last_standby_state = Some(state_str);
                }
                unreachable_streak = 0;
                consecutive_session_drops = 0;
                sleep(Duration::from_secs(KEEP_ALIVE_INTERVAL));
                continue;
            } else if last_standby_state.is_some() {
                log(&format!(
                    "Connected to campus Wi-Fi '{}'. Resuming active keepalive watchdog.",
                    ssid
                ));
                last_standby_state = None;
            }
        } else {
            if last_standby_state.as_deref() != Some("disconnected") {
                log("Wi-Fi disconnected. Waiting for connection...");
                last_standby_state = Some("disconnected".to_string());
            }
            unreachable_streak = 0;
            consecutive_session_drops = 0;
            sleep(Duration::from_secs(15));
            continue;
        }

        let current_ssid_str = current_ssid.unwrap_or_default();

        // 1. Check if portal gateway is online
        if !portal::is_portal_online() {
            unreachable_streak += 1;
            consecutive_session_drops = 0;
            log(&format!(
                "⚠ Portal gateway unreachable on '{}' (streak: {}).",
                current_ssid_str, unreachable_streak
            ));

            match unreachable_streak {
                3 => {
                    wifi::heal_network(1);
                    sleep(Duration::from_secs(10));
                }
                5 | 6 => {
                    wifi::heal_network(2);
                    sleep(Duration::from_secs(10));
                }
                s if s >= 9 => {
                    wifi::heal_network(3);
                    sleep(Duration::from_secs(15));
                    unreachable_streak = 4;
                }
                _ => {
                    sleep(Duration::from_secs(20));
                }
            }
            continue;
        }

        // 2. Portal is online
        if unreachable_streak > 0 {
            log(&format!(
                "✔ Connectivity restored after {} failed check(s).",
                unreachable_streak
            ));
            unreachable_streak = 0;
        }

        // 3. Check if session is live
        if portal::check_live(Some(&username), true) {
            consecutive_session_drops = 0;
            log(&format!(
                "Session active ({}). Next check in {}s.",
                username, KEEP_ALIVE_INTERVAL
            ));
            sleep(Duration::from_secs(KEEP_ALIVE_INTERVAL));
        } else {
            consecutive_session_drops += 1;
            if consecutive_session_drops < 2 {
                log("⚠ Keepalive missed 1 check (possible Wi-Fi jitter). Verifying in 5s before re-authenticating...");
                sleep(Duration::from_secs(5));
                continue;
            }

            consecutive_session_drops = 0;
            log(&format!("Session expired for '{}'. Logging in...", username));
            match portal::do_login(&username, &password) {
                Ok(_) => {
                    log(&format!(
                        "✔ Logged in as '{}'. Next check in {}s.",
                        username, KEEP_ALIVE_INTERVAL
                    ));
                    notify_desktop(
                        "PESU WiFi",
                        &format!("Session restored: Logged in as {}", username),
                        "normal",
                    );
                    sleep(Duration::from_secs(KEEP_ALIVE_INTERVAL));
                }
                Err(err) => {
                    log(&format!("✖ Login failed: {}", err));
                    notify_desktop("PESU WiFi Login Failed", &err, "critical");
                    sleep(Duration::from_secs(15));
                }
            }
        }
    }
}

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
