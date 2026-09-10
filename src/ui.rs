use chrono::Local;
use std::io::IsTerminal;

pub fn use_color() -> bool {
    std::io::stdout().is_terminal() || std::env::var("FORCE_COLOR").unwrap_or_default() == "1"
}

pub fn color(code: &str, text: &str) -> String {
    if use_color() {
        format!("{}{}\x1b[0m", code, text)
    } else {
        text.to_string()
    }
}

pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RED: &str = "\x1b[91m";
pub const GREEN: &str = "\x1b[92m";
pub const YELLOW: &str = "\x1b[93m";
pub const CYAN: &str = "\x1b[96m";

pub fn visual_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

pub fn print_ok(msg: &str) {
    println!("{}", color(GREEN, &format!("✔  {}", msg)));
}

pub fn print_err(msg: &str) {
    eprintln!("{}", color(RED, &format!("✖  {}", msg)));
}

pub fn print_warn(msg: &str) {
    println!("{}", color(YELLOW, &format!("⚠  {}", msg)));
}

pub fn print_info(msg: &str) {
    println!("{}", color(CYAN, &format!("➜  {}", msg)));
}

pub fn get_time_str() -> String {
    Local::now().format("[%H:%M:%S]").to_string()
}

pub fn log(msg: &str) {
    println!("{} {}", color(DIM, &get_time_str()), msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_len() {
        let plain = "PESU WiFi Status";
        let colored = format!("{}{}{}\x1b[0m", BOLD, GREEN, plain);
        assert_eq!(visual_len(plain), plain.len());
        assert_eq!(visual_len(&colored), plain.len());
        assert_eq!(visual_len(""), 0);
    }
}
