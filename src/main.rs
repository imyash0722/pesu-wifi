mod config;
mod daemon;
mod portal;
mod ui;
mod wifi;

use clap::{Parser, Subcommand};
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

fn cmd_add(user_opt: Option<&str>, pass_opt: Option<&str>) -> i32 {
    let (username, password) = match (user_opt, pass_opt) {
        (Some(u), Some(p)) => (u.trim().to_string(), p.trim().to_string()),
        (Some(u), None) => {
            let p = match rpassword::prompt_password("  password: ") {
                Ok(p) => p.trim().to_string(),
                Err(_) => {
                    println!("\nCancelled.");
                    return 1;
                }
            };
            (u.trim().to_string(), p)
        }
        _ => {
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
        }
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

fn cmd_del(user_opt: Option<&str>) -> i32 {
    let username = match user_opt {
        Some(u) => u.trim().to_string(),
        None => {
            print!("Enter username to delete: ");
            let _ = io::stdout().flush();
            let mut u = String::new();
            if io::stdin().read_line(&mut u).is_err() {
                println!("\nCancelled.");
                return 1;
            }
            u.trim().to_string()
        }
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

fn cmd_select(user_opt: Option<&str>) -> i32 {
    let mut cfg = config::load_config();
    if cfg.accounts.is_empty() {
        print_err("No saved accounts. Run 'pesu-wifi add' first.");
        return 1;
    }

    let username = match user_opt {
        Some(u) => u.trim().to_string(),
        None => {
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

#[derive(Parser, Debug)]
#[command(
    name = "pesu-wifi",
    version = VERSION,
    about = "Automated captive portal login & keepalive watchdog for PES University.",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    /// Print version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,

    /// List saved accounts showing passwords
    #[arg(short = 'p', long = "passwords")]
    passwords: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show live connection status card
    Status,

    /// Start keepalive watchdog daemon (-f to run in foreground)
    Start {
        /// Run watchdog loop in foreground
        #[arg(short = 'f', long = "foreground")]
        foreground: bool,
    },

    /// Stop background keepalive watchdog daemon
    Stop,

    /// Smart login; optionally with a specific account
    Login {
        /// Account username to log in with
        username: Option<String>,
    },

    /// Sign out cleanly from captive portal
    Logout,

    /// Set default active account (alias: use)
    #[command(alias = "use")]
    Select {
        /// Account username to select
        username: Option<String>,
    },

    /// Save or update login credentials
    Add {
        /// Optional username
        username: Option<String>,
        /// Optional password
        password: Option<String>,
    },

    /// Remove a saved account
    Del {
        /// Username to delete
        username: Option<String>,
    },

    /// List saved accounts (-p to show passwords)
    List {
        /// Show passwords
        #[arg(short = 'p', long = "passwords")]
        passwords: bool,
    },

    /// Fast-path helper to list saved accounts for shell completions
    #[command(hide = true, name = "__list-accounts")]
    ListAccounts,

    /// Hidden alias for systemd service backwards-compatibility
    #[command(hide = true)]
    Daemon,
}

fn main() {
    let cli = Cli::parse();

    if cli.passwords && cli.command.is_none() {
        std::process::exit(cmd_list(true));
    }

    match cli.command {
        Some(Commands::Status) => {
            cmd_status();
            std::process::exit(0);
        }
        Some(Commands::Start { foreground }) => {
            std::process::exit(daemon::cmd_start(foreground));
        }
        Some(Commands::Stop) => {
            std::process::exit(daemon::cmd_stop());
        }
        Some(Commands::Login { username }) => {
            std::process::exit(cmd_login(username.as_deref()));
        }
        Some(Commands::Logout) => {
            std::process::exit(cmd_logout());
        }
        Some(Commands::Select { username }) => {
            std::process::exit(cmd_select(username.as_deref()));
        }
        Some(Commands::Add { username, password }) => {
            std::process::exit(cmd_add(username.as_deref(), password.as_deref()));
        }
        Some(Commands::Del { username }) => {
            std::process::exit(cmd_del(username.as_deref()));
        }
        Some(Commands::List { passwords }) => {
            std::process::exit(cmd_list(passwords));
        }
        Some(Commands::ListAccounts) => {
            let cfg = config::load_config();
            for user in cfg.accounts.keys() {
                println!("{}", user);
            }
            std::process::exit(0);
        }
        Some(Commands::Daemon) => {
            daemon::run_daemon();
        }
        None => {
            eprintln!("pesu-wifi: no command provided");
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
