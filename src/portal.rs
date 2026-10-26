use crate::config;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0";

pub const DEFAULT_PORTAL_BASE: &str = "http://192.168.254.1:8090";

static DISCOVERED_PORTAL_BASE: Mutex<Option<String>> = Mutex::new(None);
static DYNAMIC_KEEPALIVE_INTERVAL: AtomicU64 = AtomicU64::new(180);

#[derive(Debug)]
struct InsecureServerCertVerifier;

impl rustls::client::danger::ServerCertVerifier for InsecureServerCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls_pki_types::CertificateDer<'_>,
        _intermediates: &[rustls_pki_types::CertificateDer<'_>],
        _server_name: &rustls_pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

pub fn create_agent(timeout: Duration) -> ureq::Agent {
    let tls_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(InsecureServerCertVerifier))
        .with_no_client_auth();

    ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .tls_config(Arc::new(tls_config))
        .build()
}

pub fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(unix)]
pub fn get_default_gateway_ip() -> Option<String> {
    let file = std::fs::File::open("/proc/net/route").ok()?;
    let reader = std::io::BufReader::new(file);
    use std::io::BufRead;
    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "00000000" {
            if let Ok(gw_hex) = u32::from_str_radix(parts[2], 16) {
                if gw_hex != 0 {
                    // /proc/net/route format: little-endian 32-bit hex
                    let b0 = (gw_hex & 0xFF) as u8;
                    let b1 = ((gw_hex >> 8) & 0xFF) as u8;
                    let b2 = ((gw_hex >> 16) & 0xFF) as u8;
                    let b3 = ((gw_hex >> 24) & 0xFF) as u8;
                    return Some(format!("{}.{}.{}.{}", b0, b1, b2, b3));
                }
            }
        }
    }
    None
}

#[cfg(windows)]
pub fn get_default_gateway_ip() -> Option<String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue | Select-Object -First 1).NextHop",
        ])
        .output()
        .ok()?;
    let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !ip.is_empty() && ip.contains('.') {
        Some(ip)
    } else {
        None
    }
}

pub fn invalidate_portal_base() {
    if let Ok(mut lock) = DISCOVERED_PORTAL_BASE.lock() {
        *lock = None;
    }
}

pub fn set_portal_base(base: &str) {
    if let Ok(mut lock) = DISCOVERED_PORTAL_BASE.lock() {
        *lock = Some(base.trim().trim_end_matches('/').to_string());
    }
}

pub fn probe_url_alive(base_url: &str, timeout_ms: u64) -> bool {
    let agent = create_agent(Duration::from_millis(timeout_ms));
    let url = format!("{}/httpclient.html", base_url.trim_end_matches('/'));
    let req = agent
        .get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept-Language", "en-US,en;q=0.5");

    match req.call() {
        Ok(resp) => resp.status() >= 200 && resp.status() < 500,
        Err(ureq::Error::Status(code, _)) => code >= 200 && code < 500,
        Err(_) => false,
    }
}

pub fn discover_portal_base() -> String {
    // 1. Check environment variable override
    if let Ok(env_base) = std::env::var("PESU_PORTAL_BASE") {
        let trimmed = env_base.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    // 2. Check user configuration override
    let cfg = config::load_config();
    if let Some(portal_url) = cfg.portal_url {
        let trimmed = portal_url.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    // 3. Fast probe standard PESU gateway endpoints
    let standard_candidates = [
        "http://192.168.254.1:8090",
        "https://192.168.254.1:8090",
    ];
    for &cand in &standard_candidates {
        if probe_url_alive(cand, 1200) {
            set_portal_base(cand);
            return cand.to_string();
        }
    }

    // 4. Captive Portal Probing: Query standard captive portal check URL (redirect detection)
    let probe_agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(1500))
        .timeout_read(Duration::from_millis(1500))
        .redirects(0)
        .build();

    if let Ok(resp) = probe_agent
        .get("http://connectivitycheck.gstatic.com/generate_204")
        .set("User-Agent", USER_AGENT)
        .call()
    {
        if let Some(loc) = resp.header("Location") {
            if let Ok(parsed_url) = url::Url::parse(loc) {
                if let (Some(host), Some(port)) = (parsed_url.host_str(), parsed_url.port()) {
                    let scheme = parsed_url.scheme();
                    let origin = format!("{}://{}:{}", scheme, host, port);
                    if probe_url_alive(&origin, 1500) {
                        set_portal_base(&origin);
                        return origin;
                    }
                }
            }
        }
    } else if let Err(ureq::Error::Status(code, resp)) = probe_agent
        .get("http://connectivitycheck.gstatic.com/generate_204")
        .set("User-Agent", USER_AGENT)
        .call()
    {
        if (code >= 300 && code < 400) || code == 200 {
            if let Some(loc) = resp.header("Location") {
                if let Ok(parsed_url) = url::Url::parse(loc) {
                    if let (Some(host), Some(port)) = (parsed_url.host_str(), parsed_url.port()) {
                        let scheme = parsed_url.scheme();
                        let origin = format!("{}://{}:{}", scheme, host, port);
                        if probe_url_alive(&origin, 1500) {
                            set_portal_base(&origin);
                            return origin;
                        }
                    }
                }
            }
        }
    }

    // 5. Default route gateway IP check
    if let Some(gw_ip) = get_default_gateway_ip() {
        let gw_http = format!("http://{}:8090", gw_ip);
        if probe_url_alive(&gw_http, 1200) {
            set_portal_base(&gw_http);
            return gw_http;
        }
        let gw_https = format!("https://{}:8090", gw_ip);
        if probe_url_alive(&gw_https, 1200) {
            set_portal_base(&gw_https);
            return gw_https;
        }
    }

    // 6. Secondary campus subnets fallback
    let secondary_candidates = [
        "http://10.14.142.1:8090",
        "https://10.14.142.1:8090",
        "http://172.16.1.1:8090",
        "https://172.16.1.1:8090",
    ];
    for &cand in &secondary_candidates {
        if probe_url_alive(cand, 1000) {
            set_portal_base(cand);
            return cand.to_string();
        }
    }

    // Fallback to default
    DEFAULT_PORTAL_BASE.to_string()
}

pub fn get_portal_base() -> String {
    if let Ok(env_base) = std::env::var("PESU_PORTAL_BASE") {
        let trimmed = env_base.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    let cfg = config::load_config();
    if let Some(portal_url) = cfg.portal_url {
        let trimmed = portal_url.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    if let Ok(lock) = DISCOVERED_PORTAL_BASE.lock() {
        if let Some(ref base) = *lock {
            return base.clone();
        }
    }

    discover_portal_base()
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

pub fn detect_portal_keepalive_interval() -> Option<u64> {
    let agent = create_agent(Duration::from_millis(3000));
    let url = format!("{}/httpclient.html", get_portal_base());
    let resp = agent.get(&url).set("User-Agent", USER_AGENT).call().ok()?;
    let body = resp.into_string().ok()?;

    for line in body.lines() {
        if let Some(pos) = line.find("liveReqTimeInJS") {
            let after = &line[pos + "liveReqTimeInJS".len()..];
            let digits: String = after
                .chars()
                .skip_while(|c| *c == ' ' || *c == '=' || *c == ':' || *c == '"' || *c == '\'')
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(val) = digits.parse::<u64>() {
                if val >= 30 && val <= 3600 {
                    DYNAMIC_KEEPALIVE_INTERVAL.store(val, Ordering::Relaxed);
                    return Some(val);
                }
            }
        } else if let Some(pos) = line.find("liverequesttime") {
            let after = &line[pos + "liverequesttime".len()..];
            let digits: String = after
                .chars()
                .skip_while(|c| *c == ' ' || *c == '=' || *c == ':' || *c == '"' || *c == '\'')
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(val) = digits.parse::<u64>() {
                if val >= 30 && val <= 3600 {
                    DYNAMIC_KEEPALIVE_INTERVAL.store(val, Ordering::Relaxed);
                    return Some(val);
                }
            }
        }
    }
    None
}

pub fn get_detected_keepalive_interval() -> u64 {
    DYNAMIC_KEEPALIVE_INTERVAL.load(Ordering::Relaxed)
}

pub fn clean_message(raw: &str, username: Option<&str>) -> String {
    let mut cleaned = raw
        .replace("<![CDATA[", "")
        .replace("]]>", "")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .trim()
        .to_string();

    if let Some(user) = username {
        cleaned = cleaned.replace("{username}", user);
    }
    cleaned
}

pub fn parse_xml(xml_content: &str, username: Option<&str>) -> HashMap<String, String> {
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
        .map(|(k, v)| (k, clean_message(&v, username)))
        .collect()
}

pub fn is_portal_online() -> bool {
    let base = get_portal_base();
    if probe_url_alive(&base, 3500) {
        return true;
    }

    // Portal may have moved or network changed: invalidate and rediscover
    invalidate_portal_base();
    let new_base = get_portal_base();
    if new_base != base {
        probe_url_alive(&new_base, 3500)
    } else {
        false
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
        let agent = create_agent(Duration::from_millis(3500));
        let ts = get_timestamp();
        let url = format!(
            "{}?mode=192&username={}&a={}&producttype=0",
            get_live_url(),
            user,
            ts
        );
        let req = agent
            .get(&url)
            .set("User-Agent", USER_AGENT)
            .set("Accept-Language", "en-US,en;q=0.5");

        match req.call() {
            Ok(resp) => {
                if let Ok(body) = resp.into_string() {
                    let parsed = parse_xml(&body, Some(user));
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
    std::thread::sleep(Duration::from_millis(300));
    attempt()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginResult {
    Success(String),
    ConcurrencyLimit(String),
    DataLimitReached(String),
    InvalidCredentials(String),
    AlreadyLoggedIn(String),
    Failed(String),
}

pub fn do_login(username: &str, password: &str) -> LoginResult {
    let agent = create_agent(Duration::from_millis(8000));
    let ts = get_timestamp().to_string();

    let resp_res = agent
        .post(&get_login_url())
        .set("User-Agent", USER_AGENT)
        .set("Accept-Language", "en-US,en;q=0.5")
        .send_form(&[
            ("mode", "191"),
            ("username", username),
            ("password", password),
            ("a", &ts),
            ("producttype", "0"),
        ]);

    let resp = match resp_res {
        Ok(r) => r,
        Err(e) => return LoginResult::Failed(format!("Portal unreachable ({})", e)),
    };

    let body = match resp.into_string() {
        Ok(b) => b,
        Err(e) => return LoginResult::Failed(format!("Invalid response body ({})", e)),
    };

    let parsed = parse_xml(&body, Some(username));
    let status = parsed.get("status").map(|s| s.to_uppercase()).unwrap_or_default();
    let message = parsed.get("message").cloned().unwrap_or_default();
    let msg_lower = message.to_lowercase();

    // 1. Success check
    let is_live = status == "LIVE"
        || status == "OK"
        || msg_lower.contains("signed in as")
        || msg_lower.contains("successfully logged into");

    if is_live {
        let msg = if !message.is_empty() {
            message
        } else {
            format!("Signed in as {}", username)
        };
        return LoginResult::Success(msg);
    }

    // 2. Concurrent login limit reached
    if msg_lower.contains("maximum login limit")
        || msg_lower.contains("multiple login not allowed")
        || msg_lower.contains("login limit reached")
        || msg_lower.contains("simultaneous login")
        || msg_lower.contains("limit reached")
        || msg_lower.contains("code: 529")
    {
        return LoginResult::ConcurrencyLimit(message);
    }

    // 3. Data transfer limit exceeded
    if msg_lower.contains("data transfer limit")
        || msg_lower.contains("limit has been exceeded")
        || msg_lower.contains("quota exceeded")
        || msg_lower.contains("data limit")
    {
        return LoginResult::DataLimitReached(message);
    }

    // 4. User is already logged in
    if msg_lower.contains("already logged in") {
        if check_live(Some(username), false) {
            return LoginResult::Success(format!("Session already active for {}", username));
        } else {
            return LoginResult::AlreadyLoggedIn(message);
        }
    }

    // 5. Invalid credentials
    if msg_lower.contains("could not log you on")
        || msg_lower.contains("password is correct")
        || msg_lower.contains("invalid username or password")
        || msg_lower.contains("incorrect password")
    {
        return LoginResult::InvalidCredentials(message);
    }

    // 6. Generic failure
    let err_reason = if !message.is_empty() {
        message
    } else if status == "LOGIN" {
        "Login failed: Invalid credentials or session rejected".to_string()
    } else if !status.is_empty() {
        format!("Login failed (status: {})", status)
    } else {
        "Unexpected response from captive portal".to_string()
    };
    LoginResult::Failed(err_reason)
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

    let resp = agent
        .post(&get_logout_url())
        .set("User-Agent", USER_AGENT)
        .set("Accept-Language", "en-US,en;q=0.5")
        .send_form(&[
            ("mode", "193"),
            ("username", user),
            ("a", &ts),
            ("producttype", "0"),
        ])
        .map_err(|e| format!("Portal unreachable ({})", e))?;

    let body = resp.into_string().map_err(|e| format!("Invalid response body: {}", e))?;
    let parsed = parse_xml(&body, Some(user));

    let message = parsed
        .get("message")
        .or_else(|| parsed.get("logoutmessage"))
        .cloned()
        .unwrap_or_else(|| "Signed out successfully".to_string());

    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_message_with_username() {
        let dirty = "<![CDATA[You are signed in as {username}]]>";
        assert_eq!(
            clean_message(dirty, Some("PES1UG25CS001")),
            "You are signed in as PES1UG25CS001"
        );
    }

    #[test]
    fn test_parse_xml_live_ack() {
        let xml = "<response><ack>ack</ack></response>";
        let parsed = parse_xml(xml, None);
        assert_eq!(parsed.get("ack").map(|s| s.as_str()), Some("ack"));
    }

    #[test]
    fn test_parse_xml_login_cdata() {
        let xml = "<?xml version='1.0' ?><requestresponse><status><![CDATA[LIVE]]></status><message><![CDATA[You are signed in as {username}]]></message><logoutmessage><![CDATA[You have successfully logged off]]></logoutmessage></requestresponse>";
        let parsed = parse_xml(xml, Some("testuser"));
        assert_eq!(parsed.get("status").map(|s| s.as_str()), Some("LIVE"));
        assert_eq!(
            parsed.get("message").map(|s| s.as_str()),
            Some("You are signed in as testuser")
        );
    }

    #[test]
    fn test_parse_xml_concurrency_error() {
        let xml = "<requestresponse><status><![CDATA[LOGIN]]></status><message><![CDATA[Maximum login limit reached]]></message></requestresponse>";
        let parsed = parse_xml(xml, None);
        assert_eq!(parsed.get("status").map(|s| s.as_str()), Some("LOGIN"));
        assert_eq!(
            parsed.get("message").map(|s| s.as_str()),
            Some("Maximum login limit reached")
        );
    }

    #[test]
    fn test_portal_base_env_override() {
        std::env::set_var("PESU_PORTAL_BASE", "https://10.14.142.1:8090");
        assert_eq!(get_portal_base(), "https://10.14.142.1:8090");
        std::env::remove_var("PESU_PORTAL_BASE");
    }

    #[test]
    fn test_parse_xml_data_limit() {
        let xml = "<requestresponse><status><![CDATA[LOGIN]]></status><message><![CDATA[Your data transfer limit has been exceeded.]]></message></requestresponse>";
        let parsed = parse_xml(xml, None);
        assert_eq!(parsed.get("status").map(|s| s.as_str()), Some("LOGIN"));
        assert!(parsed.get("message").unwrap().contains("data transfer limit"));
    }

    #[test]
    fn test_parse_xml_invalid_password() {
        let xml = "<requestresponse><status><![CDATA[LOGIN]]></status><message><![CDATA[The system could not log you on. Make sure your password is correct]]></message></requestresponse>";
        let parsed = parse_xml(xml, None);
        assert_eq!(parsed.get("status").map(|s| s.as_str()), Some("LOGIN"));
        assert!(parsed.get("message").unwrap().contains("password is correct"));
    }

    #[test]
    fn test_dynamic_keepalive_interval_parsing() {
        let html = "<html><script>var liveReqTimeInJS = 180; var ackTimer = 0;</script></html>";
        let mut found = None;
        for line in html.lines() {
            if let Some(pos) = line.find("liveReqTimeInJS") {
                let after = &line[pos + "liveReqTimeInJS".len()..];
                let digits: String = after
                    .chars()
                    .skip_while(|c| *c == ' ' || *c == '=' || *c == ':' || *c == '"' || *c == '\'')
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(val) = digits.parse::<u64>() {
                    found = Some(val);
                }
            }
        }
        assert_eq!(found, Some(180));
    }

    #[test]
    fn test_agent_creation_with_insecure_tls() {
        let _agent = create_agent(Duration::from_millis(1000));
    }

    #[test]
    #[cfg(unix)]
    fn test_get_default_gateway_ip() {
        let gw = get_default_gateway_ip();
        // If connected to a network, default gateway should be a valid IPv4 string
        if let Some(ip) = gw {
            assert!(ip.contains('.'));
            let parts: Vec<&str> = ip.split('.').collect();
            assert_eq!(parts.len(), 4);
        }
    }
}
