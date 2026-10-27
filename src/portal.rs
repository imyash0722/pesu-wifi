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
pub fn get_wifi_interface_name() -> Option<String> {
    // 1. Scan sysfs for wireless devices (fast, 0 overhead, no fork/exec)
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("wireless").exists() || path.join("phy80211").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    return Some(name.to_string());
                }
            }
        }
    }

    // 2. Fallback: nmcli dev
    if let Ok(out) = std::process::Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE", "dev"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if line.contains(":wifi") {
                if let Some(dev) = line.split(':').next() {
                    return Some(dev.trim().to_string());
                }
            }
        }
    }

    None
}

#[cfg(windows)]
pub fn get_wifi_interface_name() -> Option<String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-NetAdapter -Physical -ErrorAction SilentlyContinue | Where-Object { $_.MediaType -eq 'Native 802.11' -or $_.InterfaceDescription -like '*Wi-Fi*' } | Select-Object -First 1).Name",
        ])
        .output()
        .ok()?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !name.is_empty() {
        Some(name)
    } else {
        None
    }
}

#[cfg(unix)]
pub fn get_default_gateway_ip() -> Option<String> {
    let file = std::fs::File::open("/proc/net/route").ok()?;
    let reader = std::io::BufReader::new(file);
    use std::io::BufRead;

    let wifi_iface = get_wifi_interface_name();
    let mut fallback_gw = None;

    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "00000000" {
            let iface = parts[0];
            // Skip known VPN / tunnel / bridge / container interfaces
            let is_vpn = iface.starts_with("tun")
                || iface.starts_with("tap")
                || iface.starts_with("wg")
                || iface.starts_with("tailscale")
                || iface.starts_with("docker")
                || iface.starts_with("br-")
                || iface.starts_with("veth");

            if let Ok(gw_hex) = u32::from_str_radix(parts[2], 16) {
                if gw_hex != 0 {
                    let b0 = (gw_hex & 0xFF) as u8;
                    let b1 = ((gw_hex >> 8) & 0xFF) as u8;
                    let b2 = ((gw_hex >> 16) & 0xFF) as u8;
                    let b3 = ((gw_hex >> 24) & 0xFF) as u8;
                    let ip = format!("{}.{}.{}.{}", b0, b1, b2, b3);

                    // If this matches our physical Wi-Fi interface, prioritize it immediately
                    if let Some(ref w_if) = wifi_iface {
                        if iface == w_if {
                            return Some(ip);
                        }
                    }

                    if !is_vpn && fallback_gw.is_none() {
                        fallback_gw = Some(ip);
                    }
                }
            }
        }
    }
    fallback_gw
}

#[cfg(windows)]
pub fn get_default_gateway_ip() -> Option<String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
              Where-Object { $_.InterfaceAlias -like '*Wi-Fi*' -or $_.InterfaceAlias -like '*WLAN*' -or $_.InterfaceDescription -like '*Wireless*' -or $_.InterfaceDescription -like '*802.11*' } |
              Select-Object -First 1).NextHop",
        ])
        .output()
        .ok()?;
    let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !ip.is_empty() && ip.contains('.') {
        Some(ip)
    } else {
        let fallback = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
                  Where-Object { $_.InterfaceAlias -notlike '*VPN*' -and $_.InterfaceDescription -notlike '*TAP*' -and $_.InterfaceDescription -notlike '*Wintun*' -and $_.InterfaceDescription -notlike '*WireGuard*' } |
                  Select-Object -First 1).NextHop",
            ])
            .output()
            .ok()?;
        let fb_ip = String::from_utf8_lossy(&fallback.stdout).trim().to_string();
        if !fb_ip.is_empty() && fb_ip.contains('.') {
            Some(fb_ip)
        } else {
            None
        }
    }
}

#[cfg(unix)]
pub fn connect_wifi_socket(addr: std::net::SocketAddr, timeout: Duration) -> std::io::Result<std::net::TcpStream> {
    use std::os::unix::io::FromRawFd;

    let socket = if addr.is_ipv4() {
        unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) }
    } else {
        unsafe { libc::socket(libc::AF_INET6, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) }
    };
    if socket < 0 {
        return Err(std::io::Error::last_os_error());
    }

    // Direct interface binding: force all traffic on this socket out of physical Wi-Fi adapter
    // SO_BINDTOIFINDEX (62) is unprivileged on modern Linux kernels (5.7+)
    if let Some(ifname) = get_wifi_interface_name() {
        if let Ok(c_name) = std::ffi::CString::new(ifname) {
            let ifindex = unsafe { libc::if_nametoindex(c_name.as_ptr()) };
            if ifindex > 0 {
                let _ = unsafe {
                    libc::setsockopt(
                        socket,
                        libc::SOL_SOCKET,
                        libc::SO_BINDTOIFINDEX,
                        &ifindex as *const _ as *const libc::c_void,
                        std::mem::size_of::<libc::c_int>() as libc::socklen_t,
                    )
                };
            }
        }
    }

    // Non-blocking mode for timeout handling
    unsafe {
        let flags = libc::fcntl(socket, libc::F_GETFL, 0);
        libc::fcntl(socket, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }

    let (sockaddr, socklen) = match addr {
        std::net::SocketAddr::V4(v4) => {
            let mut sa: libc::sockaddr_in = unsafe { std::mem::zeroed() };
            sa.sin_family = libc::AF_INET as libc::sa_family_t;
            sa.sin_port = v4.port().to_be();
            sa.sin_addr.s_addr = u32::from_ne_bytes(v4.ip().octets());
            (
                &sa as *const libc::sockaddr_in as *const libc::sockaddr,
                std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
            )
        }
        std::net::SocketAddr::V6(v6) => {
            let mut sa: libc::sockaddr_in6 = unsafe { std::mem::zeroed() };
            sa.sin6_family = libc::AF_INET6 as libc::sa_family_t;
            sa.sin6_port = v6.port().to_be();
            sa.sin6_addr.s6_addr = v6.ip().octets();
            (
                &sa as *const libc::sockaddr_in6 as *const libc::sockaddr,
                std::mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t,
            )
        }
    };

    let ret = unsafe { libc::connect(socket, sockaddr, socklen) };
    if ret < 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() != Some(libc::EINPROGRESS) {
            unsafe { libc::close(socket); }
            return Err(err);
        }

        let mut pfd = libc::pollfd {
            fd: socket,
            events: libc::POLLOUT,
            revents: 0,
        };
        let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as libc::c_int;
        let poll_ret = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
        if poll_ret <= 0 {
            unsafe { libc::close(socket); }
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Connection timed out",
            ));
        }

        let mut sock_err: libc::c_int = 0;
        let mut err_len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
        let getsockopt_ret = unsafe {
            libc::getsockopt(
                socket,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                &mut sock_err as *mut _ as *mut libc::c_void,
                &mut err_len,
            )
        };
        if getsockopt_ret < 0 || sock_err != 0 {
            unsafe { libc::close(socket); }
            return Err(std::io::Error::from_raw_os_error(if sock_err != 0 { sock_err } else { 1 }));
        }
    }

    // Restore blocking
    unsafe {
        let flags = libc::fcntl(socket, libc::F_GETFL, 0);
        libc::fcntl(socket, libc::F_SETFL, flags & !libc::O_NONBLOCK);
        Ok(std::net::TcpStream::from_raw_fd(socket))
    }
}

#[cfg(unix)]
pub fn probe_url_direct_wifi(base_url: &str, timeout_ms: u64) -> bool {
    use std::io::{Read, Write};
    use std::net::ToSocketAddrs;

    let parsed = match url::Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return false,
    };
    let host = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };
    let port = parsed.port().unwrap_or(8090);

    let addrs = match format!("{}:{}", host, port).to_socket_addrs() {
        Ok(a) => a,
        Err(_) => return false,
    };

    let timeout = Duration::from_millis(timeout_ms);
    for addr in addrs {
        if let Ok(mut stream) = connect_wifi_socket(addr, timeout) {
            let req = format!(
                "HEAD /httpclient.html HTTP/1.1\r\nHost: {}:{}\r\nUser-Agent: {}\r\nConnection: close\r\n\r\n",
                host, port, USER_AGENT
            );
            if stream.write_all(req.as_bytes()).is_ok() {
                let mut buf = [0u8; 256];
                if let Ok(n) = stream.read(&mut buf) {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    if text.starts_with("HTTP/1.") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(unix)]
pub fn get_direct_wifi(path_and_query: &str, timeout: Duration) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::net::ToSocketAddrs;

    let base = get_portal_base();
    let parsed = url::Url::parse(&base).map_err(|e| format!("Invalid portal URL: {}", e))?;
    let host = parsed.host_str().ok_or_else(|| "No host in portal URL".to_string())?;
    let port = parsed.port().unwrap_or(8090);

    let addrs = format!("{}:{}", host, port)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution error: {}", e))?;

    let req = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nUser-Agent: {}\r\nAccept-Language: en-US,en;q=0.5\r\nConnection: close\r\n\r\n",
        path_and_query, host, port, USER_AGENT
    );

    for addr in addrs {
        if let Ok(mut stream) = connect_wifi_socket(addr, timeout) {
            let _ = stream.set_read_timeout(Some(timeout));
            let _ = stream.set_write_timeout(Some(timeout));
            if stream.write_all(req.as_bytes()).is_ok() {
                let mut resp_bytes = Vec::new();
                if stream.read_to_end(&mut resp_bytes).is_ok() {
                    let text = String::from_utf8_lossy(&resp_bytes);
                    if let Some(pos) = text.find("\r\n\r\n") {
                        return Ok(text[pos + 4..].to_string());
                    } else {
                        return Ok(text.to_string());
                    }
                }
            }
        }
    }
    Err("Direct Wi-Fi socket GET failed".to_string())
}

#[cfg(unix)]
pub fn post_direct_wifi(path: &str, params: &[(&str, &str)], timeout: Duration) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::net::ToSocketAddrs;

    let base = get_portal_base();
    let parsed = url::Url::parse(&base).map_err(|e| format!("Invalid portal URL: {}", e))?;
    let host = parsed.host_str().ok_or_else(|| "No host in portal URL".to_string())?;
    let port = parsed.port().unwrap_or(8090);

    let addrs = format!("{}:{}", host, port)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution error: {}", e))?;

    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in params {
        serializer.append_pair(k, v);
    }
    let body = serializer.finish();

    let req = format!(
        "POST {} HTTP/1.1\r\nHost: {}:{}\r\nUser-Agent: {}\r\nAccept-Language: en-US,en;q=0.5\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, host, port, USER_AGENT, body.len(), body
    );

    for addr in addrs {
        if let Ok(mut stream) = connect_wifi_socket(addr, timeout) {
            let _ = stream.set_read_timeout(Some(timeout));
            let _ = stream.set_write_timeout(Some(timeout));
            if stream.write_all(req.as_bytes()).is_ok() {
                let mut resp_bytes = Vec::new();
                if stream.read_to_end(&mut resp_bytes).is_ok() {
                    let text = String::from_utf8_lossy(&resp_bytes);
                    if let Some(pos) = text.find("\r\n\r\n") {
                        return Ok(text[pos + 4..].to_string());
                    } else {
                        return Ok(text.to_string());
                    }
                }
            }
        }
    }
    Err("Direct Wi-Fi socket POST failed".to_string())
}

#[cfg(unix)]
pub fn query_live_direct_wifi(username: &str, timeout_ms: u64) -> bool {
    let ts = get_timestamp();
    let path = format!("/live?mode=192&username={}&a={}&producttype=0", username, ts);
    if let Ok(body) = get_direct_wifi(&path, Duration::from_millis(timeout_ms)) {
        let parsed = parse_xml(&body, Some(username));
        let ack = parsed.get("ack").map(|s| s.to_lowercase()).unwrap_or_default();
        let status = parsed.get("status").map(|s| s.to_lowercase()).unwrap_or_default();
        return ack == "ack" || status.contains("live") || status.contains("ok");
    }
    false
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
        Err(_) => {
            #[cfg(unix)]
            {
                probe_url_direct_wifi(base_url, timeout_ms)
            }
            #[cfg(not(unix))]
            {
                false
            }
        }
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

    // 3. Fast probe standard PESU gateway endpoints (EC/RR Campus, Amaatra Hostel)
    let standard_candidates = [
        "http://192.168.254.1:8090",
        "https://192.168.254.1:8090",
        "http://172.16.1.1:8090",
        "https://172.16.1.1:8090",
        "http://192.168.1.1:8090",
        "https://192.168.1.1:8090",
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
    let body = match agent
        .get(&url)
        .set("User-Agent", USER_AGENT)
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
    {
        Some(b) => b,
        None => {
            #[cfg(unix)]
            {
                get_direct_wifi("/httpclient.html", Duration::from_millis(3000)).ok()?
            }
            #[cfg(not(unix))]
            {
                return None;
            }
        }
    };

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
    if probe_url_alive(&base, 1500) {
        return true;
    }

    // Portal may have moved or network changed: invalidate and rediscover across all campus gateways
    invalidate_portal_base();
    let discovered = discover_portal_base();
    if probe_url_alive(&discovered, 1500) {
        set_portal_base(&discovered);
        return true;
    }

    false
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
            Err(_) => {
                #[cfg(unix)]
                {
                    query_live_direct_wifi(user, 3500)
                }
                #[cfg(not(unix))]
                {
                    false
                }
            }
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

    let body = match resp_res {
        Ok(r) => match r.into_string() {
            Ok(b) => b,
            Err(e) => return LoginResult::Failed(format!("Invalid response body ({})", e)),
        },
        Err(e) => {
            #[cfg(unix)]
            {
                match post_direct_wifi(
                    "/login.xml",
                    &[
                        ("mode", "191"),
                        ("username", username),
                        ("password", password),
                        ("a", &ts),
                        ("producttype", "0"),
                    ],
                    Duration::from_millis(6000),
                ) {
                    Ok(b) => b,
                    Err(_) => return LoginResult::Failed(format!("Portal unreachable ({})", e)),
                }
            }
            #[cfg(not(unix))]
            {
                return LoginResult::Failed(format!("Portal unreachable ({})", e));
            }
        }
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

    let resp_res = agent
        .post(&get_logout_url())
        .set("User-Agent", USER_AGENT)
        .set("Accept-Language", "en-US,en;q=0.5")
        .send_form(&[
            ("mode", "193"),
            ("username", user),
            ("a", &ts),
            ("producttype", "0"),
        ]);

    let body = match resp_res {
        Ok(r) => match r.into_string() {
            Ok(b) => b,
            Err(e) => return Err(format!("Invalid response body: {}", e)),
        },
        Err(e) => {
            #[cfg(unix)]
            {
                match post_direct_wifi(
                    "/logout.xml",
                    &[
                        ("mode", "193"),
                        ("username", user),
                        ("a", &ts),
                        ("producttype", "0"),
                    ],
                    Duration::from_millis(5000),
                ) {
                    Ok(b) => b,
                    Err(_) => return Err(format!("Portal unreachable ({})", e)),
                }
            }
            #[cfg(not(unix))]
            {
                return Err(format!("Portal unreachable ({})", e));
            }
        }
    };
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

    #[test]
    #[cfg(unix)]
    fn test_get_wifi_interface_name() {
        let iface = get_wifi_interface_name();
        // If Wi-Fi is available on host, should be non-empty name like "wlan0"
        if let Some(name) = iface {
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_login_result_variants() {
        let s = LoginResult::Success("Signed in as user".to_string());
        assert_eq!(s, LoginResult::Success("Signed in as user".to_string()));
        let c = LoginResult::ConcurrencyLimit("529".to_string());
        assert_ne!(s, c);
    }
}
