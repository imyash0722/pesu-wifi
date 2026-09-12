mod config;
mod daemon;
mod portal;
mod ui;
mod wifi;

use std::cmp::max;
use std::io::{self, Write};
use ui::{color, print_err, print_info, print_ok, visual_len, BOLD, CYAN, DIM, GREEN, RED, YELLOW};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn cmd_status() {
    let (username, _) = config::get_active_credentials();

    let portal_alive = portal::is_portal_online();
    let session_status = if portal_alive {
        portal::check_live(username.as_deref(), false)
    } else {
        false
    };
    let (daemon_active, daemon_pid) = daemon::is_daemon_running();
    let wifi_ssid = wifi::get_active_wifi_ssid();

    let gw_val = format!(
        "{} {}",
        portal::get_portal_base(),
        if portal_alive {
            color(GREEN, "[Online]")
        } else {
            color(RED, "[Unreachable]")
        }
    );

    let s_val = if !portal_alive {
        color(&format!("{}{}", BOLD, RED), "UNREACHABLE")
    } else if session_status {
        color(&format!("{}{}", BOLD, GREEN), "LOGGED IN")
    } else {
        color(&format!("{}{}", BOLD, YELLOW), "SIGNED OUT")
    };

    let acc_val = if let Some(ref u) = username {
        color(GREEN, u)
    } else {
        color(YELLOW, "(None — run 'pesu-wifi add')")
    };

    let d_val = if daemon_active {
        let pid_str = daemon_pid
            .map(|p| format!(" (PID: {})", p))
            .unwrap_or_default();
        format!(
            "{}{}{}",
            color(GREEN, "Active"),
            color(DIM, &pid_str),
            color(DIM, &format!(" [polling: {}s]", daemon::KEEP_ALIVE_INTERVAL))
        )
    } else {
        color(
            DIM,
            &format!("Inactive [polling: {}s]", daemon::KEEP_ALIVE_INTERVAL),
        )
    };

    let rows = [
        ("Portal Gateway", gw_val),
        ("Wi-Fi Network", wifi_ssid),
        ("Session State", s_val),
        ("Active Account", acc_val),
        ("Daemon Watcher", d_val),
    ];

    let label_width = 15;
    let max_val_len = rows.iter().map(|(_, v)| visual_len(v)).max().unwrap_or(0);
    let box_width = max(58, 24 + max_val_len);

    let title_prefix = "╭── PESU WiFi Status ";
    let top_dashes = "─".repeat(box_width.saturating_sub(title_prefix.chars().count() + 1));
    let top_border = color(&format!("{}{}", BOLD, CYAN), &format!("{}{}\u{256e}", title_prefix, top_dashes));
    let bot_border = color(
        &format!("{}{}", BOLD, CYAN),
        &format!("\u{2570}{}\u{256f}", "─".repeat(box_width.saturating_sub(2))),
    );

    println!();
    println!("{}", top_border);
    for (label, val) in rows {
        let v_len = visual_len(&val);
        let pad_len = box_width.saturating_sub(24 + v_len);
        let pad = " ".repeat(pad_len);
        let bl = color(&format!("{}{}", BOLD, CYAN), "│");
        let br = color(&format!("{}{}", BOLD, CYAN), "│");
        let padded_label = format!("{:<width$}", label, width = label_width);
        println!(
            "{}  {} : {}{}  {}",
            bl,
            color(BOLD, &padded_label),
            val,
            pad,
            br
        );
    }
    println!("{}", bot_border);
    println!();
}

fn cmd_login(target_user: Option<&str>) -> i32 {
    let (username, password) = if let Some(target) = target_user {
        match config::get_credentials_for(target) {
            Some(p) => {
                let mut cfg = config::load_config();
                cfg.active_user = Some(target.to_string());
                let _ = config::save_config(&cfg);
                print_info(&format!("Using account '{}'.", target));
                (target.to_string(), p)
            }
            None => {
                print_err(&format!("No saved credentials for '{}'.", target));
                println!("{}", color(YELLOW, "  Run 'pesu-wifi add' to add this account."));
                return 1;
            }
        }
    } else {
        match config::get_active_credentials() {
            (Some(u), Some(p)) => (u, p),
            _ => {
                print_err("No credentials configured.");
                println!("{}", color(YELLOW, "  Run 'pesu-wifi add' to save login credentials."));
                return 1;
            }
        }
    };

    if !portal::is_portal_online() {
        print_err(&format!(
            "Portal gateway ({}) is unreachable. Check your Wi-Fi connection.",
            portal::get_portal_base()
        ));
        return 1;
    }

    if portal::check_live(Some(&username), true) {
        print_ok(&format!("Already logged in as '{}'.", username));
        return 0;
    }

    print_info(&format!("Logging in as '{}'...", username));
    match portal::do_login(&username, &password) {
        Ok(_) => {
            print_ok(&format!("Logged in as '{}'.", username));
            daemon::notify_desktop("PESU WiFi", &format!("Logged in as {}", username), "normal");
            0
        }
        Err(msg) => {
            print_err(&format!("Login failed: {}", msg));
            daemon::notify_desktop("PESU WiFi Login Failed", &msg, "critical");
            1
        }
    }
}

fn cmd_logout() -> i32 {
    let (username, _) = config::get_active_credentials();

    if !portal::is_portal_online() {
        print_err(&format!(
            "Portal gateway ({}) is unreachable. Check your Wi-Fi connection.",
            portal::get_portal_base()
        ));
        return 1;
    }

    if !portal::check_live(username.as_deref(), false) {
        print_ok("Already logged out. No active session found.");
        return 0;
    }

    let user_display = username.as_deref().unwrap_or("user");
    print_info(&format!("Active session found. Logging out '{}'...", user_display));
    match portal::do_logout(username.as_deref()) {
        Ok(_) => {
            print_ok("Logged out successfully.");
            daemon::notify_desktop("PESU WiFi", "Logged out successfully", "normal");
            0
        }
        Err(msg) => {
            print_err(&format!("Logout failed: {}", msg));
            1
        }
    }
}

fn cmd_add(args: &[String]) -> i32 {
    let (username, password) = if args.len() >= 2 {
        (args[0].trim().to_string(), args[1].trim().to_string())
    } else {
        println!("{}", color(BOLD, "Enter login credentials:"));
        print!("  username: ");
        let _ = io::stdout().flush();
        let mut u = String::new();
        if io::stdin().read_line(&mut u).is_err() {
            println!("\nCancelled.");
            return 1;
        }
        let u = u.trim().to_string();

        let p = match rpassword::prompt_password("  password: ") {
            Ok(p) => p.trim().to_string(),
            Err(_) => {
                println!("\nCancelled.");
                return 1;
            }
        };
        (u, p)
    };

    if username.is_empty() {
        print_err("Username cannot be empty.");
        return 1;
    }
    if password.is_empty() {
        print_err("Password cannot be empty.");
        return 1;
    }

    let mut cfg = config::load_config();
    let existed = cfg.accounts.contains_key(&username);
    cfg.accounts.insert(username.clone(), password);
    cfg.active_user = Some(username.clone());

    if let Err(e) = config::save_config(&cfg) {
        print_err(&format!("Failed to save config: {}", e));
        return 1;
    }

    let action = if existed { "Updated" } else { "Saved" };
    print_ok(&format!(
        "{} credentials for '{}'. Set as active account.",
        action, username
    ));
    0
}

fn cmd_del(args: &[String]) -> i32 {
    let username = if !args.is_empty() {
        args[0].trim().to_string()
    } else {
        print!("Enter username to delete: ");
        let _ = io::stdout().flush();
        let mut u = String::new();
        if io::stdin().read_line(&mut u).is_err() {
            println!("\nCancelled.");
            return 1;
        }
        u.trim().to_string()
    };

    if username.is_empty() {
        print_err("Username cannot be empty.");
        return 1;
    }

    let mut cfg = config::load_config();
    if !cfg.accounts.contains_key(&username) {
        print_err(&format!(
            "Account '{}' not found in saved accounts.",
            username
        ));
        return 1;
    }

    cfg.accounts.remove(&username);
    if cfg.active_user.as_deref() == Some(&username) {
        cfg.active_user = cfg.accounts.keys().next().cloned();
        if let Some(ref new_act) = cfg.active_user {
            print_info(&format!("Active account switched to '{}'.", new_act));
        }
    }

    if let Err(e) = config::save_config(&cfg) {
        print_err(&format!("Failed to save config: {}", e));
        return 1;
    }

    print_ok(&format!("Removed credentials for '{}'.", username));
    0
}

fn cmd_select(args: &[String]) -> i32 {
    let mut cfg = config::load_config();
    if cfg.accounts.is_empty() {
        print_err("No saved accounts. Run 'pesu-wifi add' first.");
        return 1;
    }

    let username = if !args.is_empty() {
        args[0].trim().to_string()
    } else {
        println!("{}", color(BOLD, "Select default account:"));
        let users: Vec<String> = cfg.accounts.keys().cloned().collect();
        for (i, u) in users.iter().enumerate() {
            let marker = if Some(u) == cfg.active_user.as_ref() {
                color(CYAN, " [active]")
            } else {
                String::new()
            };
            println!("  {} {}{}", color(DIM, &format!("{}.", i + 1)), u, marker);
        }

        print!("  Enter number or username: ");
        let _ = io::stdout().flush();
        let mut choice = String::new();
        if io::stdin().read_line(&mut choice).is_err() {
            println!("\nCancelled.");
            return 1;
        }
        let choice = choice.trim();
        if let Ok(idx) = choice.parse::<usize>() {
            if idx >= 1 && idx <= users.len() {
                users[idx - 1].clone()
            } else {
                choice.to_string()
            }
        } else {
            choice.to_string()
        }
    };

    if !cfg.accounts.contains_key(&username) {
        print_err(&format!(
            "Account '{}' not found. Run 'pesu-wifi list' to see saved accounts.",
            username
        ));
        return 1;
    }

    cfg.active_user = Some(username.clone());
    let _ = config::save_config(&cfg);
    print_ok(&format!("Default account set to '{}'.", username));
    0
}

fn cmd_list(show_passwords: bool) -> i32 {
    let cfg = config::load_config();
    println!("{}", color(&format!("{}{}", BOLD, CYAN), "\nSaved Accounts:"));

    if cfg.accounts.is_empty() {
        println!("{}", color(DIM, "  No accounts saved. Run 'pesu-wifi add' to add one.\n"));
        return 0;
    }

    for (user, pwd) in &cfg.accounts {
        let is_active = Some(user) == cfg.active_user.as_ref();
        let marker = if is_active {
            color(CYAN, " [active]")
        } else {
            String::new()
        };
        let bullet = if is_active {
            color(GREEN, "•")
        } else {
            color(DIM, "•")
        };
        let user_str = if is_active {
            color(BOLD, user)
        } else {
            user.clone()
        };
        let pwd_str = if show_passwords {
            format!("  {}", color(DIM, pwd))
        } else {
            String::new()
        };
        println!("  {} {}{}{}", bullet, user_str, marker, pwd_str);
    }
    println!();
    0
}

fn print_help() {
    let banner = format!(
        r#"{name} {ver}
{desc}

{bold_usage}
  pesu-wifi [OPTIONS] <COMMAND>

{bold_commands}
  {c_status} Show live connection status card
  {c_start} Start keepalive watchdog daemon (-f to run in foreground)
  {c_stop} Stop background keepalive watchdog daemon
  {c_login} Smart login; optionally with a specific account
  {c_logout} Sign out cleanly from captive portal
  {c_select} Set default active account (alias: use)
  {c_add} Save or update login credentials
  {c_del} Remove a saved account
  {c_list} List saved accounts (-p to show passwords)

{bold_options}
  {c_help} Print help information
  {c_version} Print version information
"#,
        name = color(&format!("{}{}", BOLD, CYAN), "PESU WiFi Manager"),
        ver = color(DIM, &format!("v{}", VERSION)),
        desc = color(
            DIM,
            "Automated captive portal login & keepalive watchdog for PES University."
        ),
        bold_usage = color(BOLD, "Usage:"),
        bold_commands = color(BOLD, "Commands:"),
        bold_options = color(BOLD, "Options:"),
        c_status = color(GREEN, &format!("{:<26}", "status")),
        c_start = color(GREEN, &format!("{:<26}", "start [-f, --foreground]")),
        c_stop = color(GREEN, &format!("{:<26}", "stop")),
        c_login = color(GREEN, &format!("{:<26}", "login [username]")),
        c_logout = color(GREEN, &format!("{:<26}", "logout")),
        c_select = color(GREEN, &format!("{:<26}", "select [username]")),
        c_add = color(GREEN, &format!("{:<26}", "add")),
        c_del = color(GREEN, &format!("{:<26}", "del [username]")),
        c_list = color(GREEN, &format!("{:<26}", "list [-p, --passwords]")),
        c_help = color(GREEN, &format!("{:<26}", "-h, --help")),
        c_version = color(GREEN, &format!("{:<26}", "-v, --version")),
    );
    print!("{}", banner);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("pesu-wifi: no command provided");
        eprintln!("Try 'pesu-wifi --help' for more information.");
        std::process::exit(2);
    }

    let cmd = args[1].to_lowercase();
    let cmd_args = &args[2..];

    if cmd_args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        std::process::exit(0);
    }

    match cmd.as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            std::process::exit(0);
        }
        "-v" | "--version" | "version" => {
            println!("pesu-wifi v{}", VERSION);
            std::process::exit(0);
        }
        "status" => {
            cmd_status();
            std::process::exit(0);
        }
        "start" => {
            let fg = cmd_args.iter().any(|a| a == "-f" || a == "--foreground");
            std::process::exit(daemon::cmd_start(fg));
        }
        "stop" => {
            std::process::exit(daemon::cmd_stop());
        }
        "login" => {
            let target = cmd_args.first().map(|s| s.as_str());
            std::process::exit(cmd_login(target));
        }
        "logout" => {
            std::process::exit(cmd_logout());
        }
        "select" | "use" => {
            std::process::exit(cmd_select(cmd_args));
        }
        "add" => {
            std::process::exit(cmd_add(cmd_args));
        }
        "del" => {
            std::process::exit(cmd_del(cmd_args));
        }
        "list" => {
            let show_pw = cmd_args.iter().any(|a| a == "-p" || a == "--passwords");
            std::process::exit(cmd_list(show_pw));
        }
        "__list-accounts" => {
            let cfg = config::load_config();
            for user in cfg.accounts.keys() {
                println!("{}", user);
            }
            std::process::exit(0);
        }
        "daemon" => {
            daemon::run_daemon();
        }
        _ => {
            eprintln!("pesu-wifi: unrecognized command '{}'", cmd);
            eprintln!("Try 'pesu-wifi --help' for more information.");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::portal::{clean_message, parse_xml};

    #[test]
    fn test_parse_xml_live_ack() {
        let xml = "<response><ack>ack</ack></response>";
        let parsed = parse_xml(xml);
        assert_eq!(parsed.get("ack").map(|s| s.as_str()), Some("ack"));
    }

    #[test]
    fn test_parse_xml_login_cdata() {
        let xml = "<?xml version='1.0' ?><requestresponse><status><![CDATA[LIVE]]></status><message><![CDATA[You are signed in as {username}]]></message><logoutmessage><![CDATA[You have successfully logged off]]></logoutmessage></requestresponse>";
        let parsed = parse_xml(xml);
        assert_eq!(parsed.get("status").map(|s| s.as_str()), Some("LIVE"));
        assert_eq!(
            parsed.get("message").map(|s| s.as_str()),
            Some("You are signed in as {username}")
        );
        assert_eq!(
            parsed.get("logoutmessage").map(|s| s.as_str()),
            Some("You have successfully logged off")
        );
    }

    #[test]
    fn test_parse_xml_html_entities() {
        let xml = "<requestresponse><status><![CDATA[LOGIN]]></status><message><![CDATA[You&#39;ve signed out]]></message></requestresponse>";
        let parsed = parse_xml(xml);
        assert_eq!(parsed.get("status").map(|s| s.as_str()), Some("LOGIN"));
        assert_eq!(
            parsed.get("message").map(|s| s.as_str()),
            Some("You've signed out")
        );
    }

    #[test]
    fn test_clean_message() {
        let dirty = "<![CDATA[The system could not log you on. Make sure your password is correct]]>";
        assert_eq!(
            clean_message(dirty),
            "The system could not log you on. Make sure your password is correct"
        );
    }

    #[test]
    fn test_parse_xml_failed_status() {
        let xml = "<requestresponse><status><![CDATA[FAILED]]></status><message><![CDATA[The system could not log you on. Make sure your password is correct]]></message></requestresponse>";
        let parsed = parse_xml(xml);
        assert_eq!(parsed.get("status").map(|s| s.as_str()), Some("FAILED"));
        assert_eq!(
            parsed.get("message").map(|s| s.as_str()),
            Some("The system could not log you on. Make sure your password is correct")
        );
    }
}
