use crate::config;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_portal_base() -> String {
    std::env::var("PESU_PORTAL_BASE").unwrap_or_else(|_| "http://192.168.254.1:8090".to_string())
}

pub fn get_login_url() -> String {
    format!("{}/login.xml", get_portal_base())
}

pub fn get_logout_url() -> String {
    format!("{}/logout.xml", get_portal_base())
}

pub fn get_live_url() -> String {
    format!("{}/live", get_portal_base())
}

pub fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub fn clean_message(raw: &str) -> String {
    raw.replace("<![CDATA[", "")
        .replace("]]>", "")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .trim()
        .to_string()
}

pub fn parse_xml(xml_content: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if xml_content.trim().is_empty() {
        return result;
    }

    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut current_tag = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
            }
            Ok(Event::Text(e)) => {
                if !current_tag.is_empty() {
                    let text = String::from_utf8_lossy(e.as_ref()).to_string();
                    result
                        .entry(current_tag.clone())
                        .or_insert_with(String::new)
                        .push_str(&text);
                }
            }
            Ok(Event::CData(e)) => {
                if !current_tag.is_empty() {
                    let text = String::from_utf8_lossy(e.as_ref()).to_string();
                    result
                        .entry(current_tag.clone())
                        .or_insert_with(String::new)
                        .push_str(&text);
                }
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                // Quick regex-like extraction fallback for broken XML
                let tags = ["status", "message", "ack", "logoutmessage", "state"];
                for tag in tags {
                    let start_tag = format!("<{}>", tag);
                    let end_tag = format!("</{}>", tag);
                    if let Some(start) = xml_content.find(&start_tag) {
                        let content_start = start + start_tag.len();
                        if let Some(end) = xml_content[content_start..].find(&end_tag) {
                            let val = &xml_content[content_start..content_start + end];
                            result.insert(tag.to_string(), val.to_string());
                        }
                    }
                }
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    result
        .into_iter()
        .map(|(k, v)| (k, clean_message(&v)))
        .collect()
}

pub fn create_agent(timeout: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build()
}

pub fn is_portal_online() -> bool {
    let agent = create_agent(Duration::from_millis(4000));
    let url = format!("{}/httpclient.html", get_portal_base());
    let req = agent
        .get(&url)
        .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0")
        .set("Accept-Language", "en-US,en;q=0.5")
        .set("Connection", "close");

    match req.call() {
        Ok(resp) => resp.status() >= 200 && resp.status() < 500,
        Err(ureq::Error::Status(code, _)) => code >= 200 && code < 500,
        Err(_) => false,
    }
}

pub fn check_live(username: Option<&str>, retry: bool) -> bool {
    let user_buf;
    let user = match username {
        Some(u) => u,
        None => {
            let (u, _) = config::get_active_credentials();
            user_buf = u.unwrap_or_else(|| "user".to_string());
            &user_buf
        }
    };

    let attempt = || -> bool {
        let agent = create_agent(Duration::from_millis(4000));
        let ts = get_timestamp();
        let url = format!(
            "{}?mode=192&username={}&a={}&producttype=0",
            get_live_url(),
            user,
            ts
        );
        let req = agent
            .get(&url)
            .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0")
            .set("Accept-Language", "en-US,en;q=0.5")
            .set("Connection", "close");

        match req.call() {
            Ok(resp) => {
                if let Ok(body) = resp.into_string() {
                    let parsed = parse_xml(&body);
                    let ack = parsed.get("ack").map(|s| s.to_lowercase()).unwrap_or_default();
                    let status = parsed.get("status").map(|s| s.to_lowercase()).unwrap_or_default();
                    ack == "ack" || status.contains("live") || status.contains("ok")
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    };

    if attempt() {
        return true;
    }
    if !retry {
        return false;
    }
    std::thread::sleep(Duration::from_millis(500));
    attempt()
}

pub fn do_login(username: &str, password: &str) -> Result<String, String> {
    let agent = create_agent(Duration::from_millis(8000));
    let ts = get_timestamp().to_string();

    let req = agent
        .post(&get_login_url())
        .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0")
        .set("Accept-Language", "en-US,en;q=0.5")
        .set("Connection", "close");

    let resp = req
        .send_form(&[
            ("mode", "191"),
            ("username", username),
            ("password", password),
            ("a", &ts),
            ("producttype", "0"),
        ])
        .map_err(|e| format!("Portal unreachable ({})", e))?;

    let body = resp.into_string().map_err(|e| format!("Invalid response body: {}", e))?;
    let parsed = parse_xml(&body);

    let status = parsed.get("status").map(|s| s.to_uppercase()).unwrap_or_default();
    let message = parsed.get("message").cloned().unwrap_or_default();
    let msg_lower = message.to_lowercase();

    let is_live = status == "LIVE"
        || msg_lower.contains("signed in")
        || msg_lower.contains("you are signed in");

    if is_live {
        let msg = if !message.is_empty() {
            message
        } else {
            format!("Signed in as {}", username)
        };
        Ok(msg)
    } else {
        let err_reason = if !message.is_empty() {
            message
        } else if status == "LOGIN" {
            "Login failed: Invalid credentials or session limit reached".to_string()
        } else if !status.is_empty() {
            format!("Login failed (status: {})", status)
        } else {
            "Unexpected response from portal".to_string()
        };
        Err(err_reason)
    }
}

pub fn do_logout(username: Option<&str>) -> Result<String, String> {
    let user_buf;
    let user = match username {
        Some(u) => u,
        None => {
            let (u, _) = config::get_active_credentials();
            user_buf = u.unwrap_or_else(|| "user".to_string());
            &user_buf
        }
    };

    let agent = create_agent(Duration::from_millis(6000));
    let ts = get_timestamp().to_string();

    let req = agent
        .post(&get_logout_url())
        .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0")
        .set("Accept-Language", "en-US,en;q=0.5")
        .set("Connection", "close");

    let resp = req
        .send_form(&[
            ("mode", "193"),
            ("username", user),
            ("a", &ts),
            ("producttype", "0"),
        ])
        .map_err(|e| format!("Portal unreachable ({})", e))?;

    let body = resp.into_string().map_err(|e| format!("Invalid response body: {}", e))?;
    let parsed = parse_xml(&body);

    let message = parsed
        .get("message")
        .or_else(|| parsed.get("logoutmessage"))
        .cloned()
        .unwrap_or_else(|| "Signed out successfully".to_string());

    Ok(message)
}
