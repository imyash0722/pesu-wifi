use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub active_user: Option<String>,
    #[serde(default)]
    pub accounts: HashMap<String, String>,
    #[serde(default)]
    pub preferred_ssid: Option<String>,
}

pub fn get_config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("pesu-wifi");
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("pesu-wifi")
}

pub fn load_config() -> Config {
    let mut candidates = Vec::new();
    candidates.push(get_config_dir().join("config.json"));

    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if !sudo_user.is_empty() {
            candidates.push(PathBuf::from(format!("/home/{}/.config/pesu-wifi/config.json", sudo_user)));
        }
    }
    candidates.push(PathBuf::from("/etc/pesu-wifi/config.json"));

    for path in &candidates {
        if path.is_file() {
            if let Ok(file) = File::open(path) {
                if let Ok(cfg) = serde_json::from_reader::<_, Config>(file) {
                    return cfg;
                }
            }
        }
    }

    // Fallback: .env file
    let mut env_candidates = Vec::new();
    env_candidates.push(get_config_dir().join(".env"));
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if !sudo_user.is_empty() {
            env_candidates.push(PathBuf::from(format!("/home/{}/.config/pesu-wifi/.env", sudo_user)));
        }
    }
    env_candidates.push(PathBuf::from(".env"));

    for path in &env_candidates {
        if path.is_file() {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                let mut usr = None;
                let mut pwd = None;
                for line in reader.lines().flatten() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') && line.contains('=') {
                        let mut parts = line.splitn(2, '=');
                        let k = parts.next().unwrap_or("").trim();
                        let v = parts.next().unwrap_or("").trim().trim_matches('\'').trim_matches('"');
                        if k == "PESU_USERNAME" {
                            usr = Some(v.to_string());
                        } else if k == "PESU_PASSWORD" {
                            pwd = Some(v.to_string());
                        }
                    }
                }
                if let (Some(u), Some(p)) = (usr, pwd) {
                    let mut accounts = HashMap::new();
                    accounts.insert(u.clone(), p);
                    return Config {
                        active_user: Some(u),
                        accounts,
                        preferred_ssid: None,
                    };
                }
            }
        }
    }

    // Fallback: environment variables
    let env_usr = std::env::var("PESU_USERNAME").ok();
    let env_pwd = std::env::var("PESU_PASSWORD").ok();
    if let (Some(u), Some(p)) = (env_usr, env_pwd) {
        if !u.is_empty() && !p.is_empty() {
            let mut accounts = HashMap::new();
            accounts.insert(u.clone(), p);
            return Config {
                active_user: Some(u),
                accounts,
                preferred_ssid: None,
            };
        }
    }

    Config::default()
}

pub fn save_config(config: &Config) -> std::io::Result<()> {
    let conf_dir = get_config_dir();
    fs::create_dir_all(&conf_dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&conf_dir, fs::Permissions::from_mode(0o700));
    }

    let json_path = conf_dir.join("config.json");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&json_path)?;

    serde_json::to_writer_pretty(&mut file, config)?;
    file.flush()?;

    let env_path = conf_dir.join(".env");
    let mut env_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&env_path)?;

    let active_usr = config.active_user.as_deref().unwrap_or("");
    let active_pwd = active_usr
        .strip_prefix("")
        .and_then(|u| config.accounts.get(u))
        .map(|s| s.as_str())
        .unwrap_or("");

    writeln!(env_file, "# PESU WiFi Saved Credentials")?;
    writeln!(env_file, "PESU_USERNAME={}", active_usr)?;
    writeln!(env_file, "PESU_PASSWORD={}", active_pwd)?;
    env_file.flush()?;

    Ok(())
}

pub fn get_active_credentials() -> (Option<String>, Option<String>) {
    let cfg = load_config();
    if let Some(active) = &cfg.active_user {
        if let Some(pwd) = cfg.accounts.get(active) {
            return (Some(active.clone()), Some(pwd.clone()));
        }
    }
    if let Some((user, pwd)) = cfg.accounts.iter().next() {
        return (Some(user.clone()), Some(pwd.clone()));
    }
    (None, None)
}

pub fn get_credentials_for(username: &str) -> Option<String> {
    let cfg = load_config();
    cfg.accounts.get(username).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serde() {
        let mut accounts = HashMap::new();
        accounts.insert("testuser".to_string(), "testpass".to_string());
        let cfg = Config {
            active_user: Some("testuser".to_string()),
            accounts,
            preferred_ssid: Some("PESU-EC-Campus".to_string()),
        };
        let serialized = serde_json::to_string(&cfg).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.active_user, Some("testuser".to_string()));
        assert_eq!(deserialized.accounts.get("testuser").unwrap(), "testpass");
        assert_eq!(deserialized.preferred_ssid, Some("PESU-EC-Campus".to_string()));
    }
}
