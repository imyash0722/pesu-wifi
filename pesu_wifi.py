#!/usr/bin/env python3
"""
PESU WiFi Login Manager & Resilient Watchdog Daemon
Automated captive portal login manager and keepalive daemon for PES University.
"""
import sys
import os
import time
import html
import json
import fcntl
import getpass
import subprocess
import requests
import re
import xml.etree.ElementTree as ET
from datetime import datetime

# ── ANSI Color & Styling ──────────────────────────────────────────────────────
USE_COLOR = sys.stdout.isatty() or os.getenv("FORCE_COLOR") == "1"
ANSI_REGEX = re.compile(r"\x1b\[[0-9;]*m")

def _c(code: str, text: str) -> str:
    return f"{code}{text}\033[0m" if USE_COLOR else text

def visual_len(s: str) -> int:
    return len(ANSI_REGEX.sub("", s))

BOLD    = "\033[1m"
DIM     = "\033[2m"
RED     = "\033[91m"
GREEN   = "\033[92m"
YELLOW  = "\033[93m"
BLUE    = "\033[94m"
MAGENTA = "\033[95m"
CYAN    = "\033[96m"
WHITE   = "\033[97m"

def print_ok(msg):    print(_c(GREEN,  f"✔  {msg}"))
def print_err(msg):   print(_c(RED,    f"✖  {msg}"))
def print_warn(msg):  print(_c(YELLOW, f"⚠  {msg}"))
def print_info(msg):  print(_c(CYAN,   f"➜  {msg}"))

def get_time_str():
    return datetime.now().strftime("[%H:%M:%S]")

def log(msg):
    print(f"{_c(DIM, get_time_str())} {msg}", flush=True)

# ── Configuration & Paths ─────────────────────────────────────────────────────
PORTAL_BASE         = os.getenv("PESU_PORTAL_BASE", "http://192.168.254.1:8090")
LOGIN_URL           = f"{PORTAL_BASE}/login.xml"
LOGOUT_URL          = f"{PORTAL_BASE}/logout.xml"
LIVE_URL            = f"{PORTAL_BASE}/live"
DEFAULT_WIFI_CON    = os.getenv("PESU_WIFI_CON", "PESU-EC-Campus")
LOCK_FILE           = "/tmp/pesu_wifi_daemon.lock"
KEEP_ALIVE_INTERVAL = 60  # seconds between portal keepalive pings

# ── Config File Helpers ───────────────────────────────────────────────────────
def get_config_dir() -> str:
    xdg = os.getenv("XDG_CONFIG_HOME")
    return os.path.join(xdg, "pesu-wifi") if xdg else os.path.join(os.path.expanduser("~"), ".config", "pesu-wifi")

def load_config_data() -> dict:
    candidates = [os.path.join(get_config_dir(), "config.json")]
    sudo_user = os.getenv("SUDO_USER")
    if sudo_user:
        candidates.append(f"/home/{sudo_user}/.config/pesu-wifi/config.json")
    candidates.append("/etc/pesu-wifi/config.json")

    for path in candidates:
        if os.path.isfile(path):
            try:
                with open(path, "r", encoding="utf-8") as f:
                    return json.load(f)
            except Exception:
                pass

    # Fallback: .env file
    env_candidates = [os.path.join(get_config_dir(), ".env"), ".env"]
    if sudo_user:
        env_candidates.insert(1, f"/home/{sudo_user}/.config/pesu-wifi/.env")
    for path in env_candidates:
        if os.path.isfile(path):
            try:
                env_dict = {}
                with open(path, "r", encoding="utf-8") as f:
                    for line in f:
                        line = line.strip()
                        if line and not line.startswith("#") and "=" in line:
                            k, v = line.split("=", 1)
                            env_dict[k.strip()] = v.strip().strip("'\"")
                usr = env_dict.get("PESU_USERNAME")
                pwd = env_dict.get("PESU_PASSWORD")
                if usr and pwd:
                    return {"active_user": usr, "accounts": {usr: pwd}}
            except Exception:
                pass

    # Fallback: env vars
    env_usr = os.getenv("PESU_USERNAME")
    env_pwd = os.getenv("PESU_PASSWORD")
    if env_usr and env_pwd:
        return {"active_user": env_usr, "accounts": {env_usr: env_pwd}}

    return {"active_user": None, "accounts": {}}

def save_config_data(data: dict):
    conf_dir = get_config_dir()
    os.makedirs(conf_dir, exist_ok=True)
    json_path = os.path.join(conf_dir, "config.json")
    env_path  = os.path.join(conf_dir, ".env")

    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
    os.chmod(json_path, 0o600)

    active = data.get("active_user")
    pwd = data.get("accounts", {}).get(active, "") if active else ""
    with open(env_path, "w", encoding="utf-8") as f:
        f.write(f"# PESU WiFi Saved Credentials\nPESU_USERNAME={active or ''}\nPESU_PASSWORD={pwd or ''}\n")
    os.chmod(env_path, 0o600)

def get_active_credentials() -> tuple[str | None, str | None]:
    data = load_config_data()
    active   = data.get("active_user")
    accounts = data.get("accounts", {})
    if active and active in accounts:
        return active, accounts[active]
    if accounts:
        first = next(iter(accounts))
        return first, accounts[first]
    return None, None

def get_credentials_for(username: str) -> str | None:
    data = load_config_data()
    return data.get("accounts", {}).get(username)

# ── HTTP Engine (fresh session per request — avoids TCP keepalive hangs) ──────
_BASE_HEADERS = {
    "User-Agent":      "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0",
    "Accept-Language": "en-US,en;q=0.5",
    "Connection":      "close",
}

def _request(method: str, url: str, **kwargs):
    """Always create a fresh Session so we never reuse a stale TCP socket."""
    s = requests.Session()
    s.trust_env = False
    s.headers.update(_BASE_HEADERS)
    try:
        fn = s.get if method.upper() == "GET" else s.post
        return fn(url, **kwargs)
    finally:
        s.close()

# ── Daemon Lock ───────────────────────────────────────────────────────────────
_lock_fd = None

def acquire_daemon_lock():
    global _lock_fd
    try:
        _lock_fd = open(LOCK_FILE, "w")
        fcntl.flock(_lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except (IOError, OSError):
        print_warn("Another instance of pesu-wifi daemon is already running. Exiting.")
        sys.exit(0)

def is_daemon_running() -> tuple[bool, int | None]:
    try:
        fd = open(LOCK_FILE, "a+")
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        fcntl.flock(fd, fcntl.LOCK_UN)
        fd.close()
        return False, None
    except (IOError, OSError):
        try:
            r = subprocess.run(["pgrep", "-f", "pesu_wifi.py daemon"],
                               capture_output=True, text=True)
            pids = [int(p) for p in r.stdout.strip().split()
                    if p and int(p) != os.getpid()]
            return True, pids[0] if pids else None
        except Exception:
            return True, None

# ── Network & Portal Helpers ──────────────────────────────────────────────────
def get_timestamp() -> int:
    return int(time.time() * 1000)

def clean_message(raw: str) -> str:
    return html.unescape(raw or "").strip()

def get_active_wifi_ssid() -> str:
    try:
        res = subprocess.run(
            ["nmcli", "-t", "-f", "name,type", "connection", "show", "--active"],
            capture_output=True, text=True, timeout=2)
        for line in res.stdout.strip().splitlines():
            if ":802-11-wireless" in line:
                return line.split(":")[0]
    except Exception:
        pass
    data = load_config_data()
    return data.get("preferred_ssid") or DEFAULT_WIFI_CON

def is_portal_online() -> bool:
    """Fast check (20-40ms) whether the portal gateway responds."""
    try:
        r = _request("GET", PORTAL_BASE, timeout=4.0)
        return r.status_code == 200
    except Exception:
        return False

def check_live(username: str | None = None, retry: bool = True) -> bool:
    """
    Returns True if user has an active session, False otherwise.
    The Cyberoam portal responds in ~15ms with <ack>ack</ack> when logged in,
    or drops/hangs the connection when logged out.
    Includes a 500ms jitter retry to avoid false session drops on congested campus Wi-Fi.
    """
    if not username:
        username, _ = get_active_credentials()
        username = username or "user"
    def _attempt() -> bool:
        try:
            r = _request("GET", LIVE_URL, params={
                "mode": 192, "username": username,
                "a": get_timestamp(), "producttype": 0,
            }, timeout=4.0)
            root = ET.fromstring(r.text)
            ack    = (root.findtext("ack")    or "").strip().lower()
            status = (root.findtext("status") or "").strip().lower()
            return ack == "ack" or "live" in status or "ok" in status
        except Exception:
            return False

    if _attempt():
        return True
    if not retry:
        return False
    time.sleep(0.5)
    return _attempt()

def do_login(username: str, password: str) -> tuple[bool, str]:
    """Returns (success, message)."""
    try:
        r = _request("POST", LOGIN_URL, data={
            "mode": 191, "username": username, "password": password,
            "a": get_timestamp(), "producttype": 0,
        }, timeout=8)
        root    = ET.fromstring(r.text)
        message = clean_message(root.findtext("message") or "")
        status  = (root.findtext("status") or "").strip().upper()

        if status == "LIVE":
            return True, f"Signed in as {username}"
        if "signed in" in message.lower() or "you are signed in" in message.lower():
            return True, f"Signed in as {username}"
        if "failed" in message.lower() or "invalid" in message.lower():
            return False, message
        return True, message or f"Signed in as {username}"
    except Exception as e:
        return False, f"Portal unreachable ({e})"

def do_logout(username: str | None = None) -> tuple[bool, str]:
    """Returns (success, message)."""
    if not username:
        username, _ = get_active_credentials()
        username = username or "user"
    try:
        r = _request("POST", LOGOUT_URL, data={
            "mode": 193, "username": username,
            "a": get_timestamp(), "producttype": 0,
        }, timeout=6)
        root    = ET.fromstring(r.text)
        message = clean_message(root.findtext("message") or "")
        return True, message or "Signed out successfully"
    except Exception as e:
        return False, f"Portal unreachable ({e})"

def heal_network(tier: int):
    """Multi-tier self-healing: reconnect SSID → cycle radio → restart NetworkManager."""
    wifi_con = get_active_wifi_ssid()
    try:
        if tier == 1:
            log(f"[Self-Healing L1] Reconnecting to '{wifi_con}'...")
            subprocess.run(["nmcli", "connection", "up", wifi_con], timeout=20)
        elif tier == 2:
            log("[Self-Healing L2] Cycling Wi-Fi radio...")
            subprocess.run(["nmcli", "radio", "wifi", "off"], timeout=10)
            time.sleep(2)
            subprocess.run(["nmcli", "radio", "wifi", "on"], timeout=10)
            time.sleep(5)
            subprocess.run(["nmcli", "connection", "up", wifi_con], timeout=20)
        elif tier == 3:
            log("[Self-Healing L3] Restarting NetworkManager...")
            subprocess.run(["systemctl", "restart", "NetworkManager"], timeout=20)
            time.sleep(6)
            subprocess.run(["nmcli", "connection", "up", wifi_con], timeout=20)
    except Exception as err:
        log(f"Self-healing Tier {tier} error: {err}")

# ── User Commands ─────────────────────────────────────────────────────────────

def cmd_status():
    username, _ = get_active_credentials()

    portal_alive = is_portal_online()
    session_status = check_live(username) if portal_alive else False
    daemon_active, daemon_pid = is_daemon_running()
    wifi_ssid = get_active_wifi_ssid()

    gw_val = f"{PORTAL_BASE} " + (_c(GREEN, "[Online]") if portal_alive else _c(RED, "[Unreachable]"))

    if not portal_alive:
        s_val = _c(BOLD + RED, "UNREACHABLE")
    elif session_status:
        s_val = _c(BOLD + GREEN, "LOGGED IN")
    else:
        s_val = _c(BOLD + YELLOW, "SIGNED OUT")

    acc_val = _c(GREEN, username) if username else _c(YELLOW, "(None — run 'pesu-wifi add')")

    if daemon_active:
        pid_str = f" (PID: {daemon_pid})" if daemon_pid else ""
        d_val = _c(GREEN, "Active") + _c(DIM, pid_str) + " " + _c(DIM, f"[polling: {KEEP_ALIVE_INTERVAL}s]")
    else:
        d_val = _c(DIM, f"Inactive [polling: {KEEP_ALIVE_INTERVAL}s]")

    rows = [
        ("Portal Gateway", gw_val),
        ("Wi-Fi Network",  wifi_ssid),
        ("Session State",  s_val),
        ("Active Account", acc_val),
        ("Daemon Watcher", d_val),
    ]

    label_width = 15
    max_val_len = max(visual_len(v) for _, v in rows)
    box_width   = max(58, 24 + max_val_len)

    title_prefix = "╭── PESU WiFi Status "
    top_border   = _c(BOLD + CYAN, title_prefix + ("─" * (box_width - len(title_prefix) - 1)) + "╮")
    bot_border   = _c(BOLD + CYAN, "╰" + ("─" * (box_width - 2)) + "╯")

    print("")
    print(top_border)
    for label, val in rows:
        pad  = " " * max(0, box_width - 24 - visual_len(val))
        bl   = _c(BOLD + CYAN, "│")
        br   = _c(BOLD + CYAN, "│")
        print(f"{bl}  {_c(BOLD, label.ljust(label_width))} : {val}{pad}  {br}")
    print(bot_border)
    print("")

def cmd_login(target_user: str | None = None):
    """Login. If target_user is given, use that account (and set as active)."""
    if target_user:
        password = get_credentials_for(target_user)
        if not password:
            print_err(f"No saved credentials for '{target_user}'.")
            print(_c(YELLOW, f"  Run 'pesu-wifi add' to add this account."))
            return 1
        username = target_user
        data = load_config_data()
        data["active_user"] = username
        save_config_data(data)
        print_info(f"Using account '{username}'.")
    else:
        username, password = get_active_credentials()
        if not username or not password:
            print_err("No credentials configured.")
            print(_c(YELLOW, "  Run 'pesu-wifi add' to save login credentials."))
            return 1

    if not is_portal_online():
        print_err(f"Portal gateway ({PORTAL_BASE}) is unreachable. Check your Wi-Fi connection.")
        return 1

    if check_live(username):
        print_ok(f"Already logged in as '{username}'.")
        return 0

    print_info(f"Logging in as '{username}'...")
    success, msg = do_login(username, password)
    if success:
        print_ok(f"Logged in as '{username}'.")
        return 0
    else:
        print_err(f"Login failed: {msg}")
        return 1

def cmd_logout():
    username, _ = get_active_credentials()

    if not is_portal_online():
        print_err(f"Portal gateway ({PORTAL_BASE}) is unreachable. Check your Wi-Fi connection.")
        return 1

    if not check_live(username):
        print_ok("Already logged out. No active session found.")
        return 0

    print_info(f"Active session found. Logging out '{username or 'user'}'...")
    success, msg = do_logout(username)
    if success:
        print_ok("Logged out successfully.")
        return 0
    else:
        print_err(f"Logout failed: {msg}")
        return 1

def cmd_add(args: list[str]):
    if len(args) >= 2:
        username = args[0].strip()
        password = args[1].strip()
    else:
        print(_c(BOLD, "Enter login credentials:"))
        try:
            username = input("  username: ").strip()
            password = getpass.getpass("  password: ").strip()
        except (KeyboardInterrupt, EOFError):
            print("\nCancelled.")
            return 1

    if not username:
        print_err("Username cannot be empty.")
        return 1
    if not password:
        print_err("Password cannot be empty.")
        return 1

    data = load_config_data()
    if "accounts" not in data:
        data["accounts"] = {}

    existed = username in data["accounts"]
    data["accounts"][username] = password
    data["active_user"] = username
    save_config_data(data)

    action = "Updated" if existed else "Saved"
    print_ok(f"{action} credentials for '{username}'. Set as active account.")
    return 0

def cmd_del(args: list[str]):
    if len(args) >= 1:
        username = args[0].strip()
    else:
        try:
            username = input("Enter username to delete: ").strip()
        except (KeyboardInterrupt, EOFError):
            print("\nCancelled.")
            return 1

    if not username:
        print_err("Username cannot be empty.")
        return 1

    data     = load_config_data()
    accounts = data.get("accounts", {})

    if username not in accounts:
        print_err(f"Account '{username}' not found in saved accounts.")
        return 1

    del accounts[username]
    if data.get("active_user") == username:
        data["active_user"] = next(iter(accounts)) if accounts else None
        if data["active_user"]:
            print_info(f"Active account switched to '{data['active_user']}'.")

    data["accounts"] = accounts
    save_config_data(data)
    print_ok(f"Removed credentials for '{username}'.")
    return 0

def cmd_select(args: list[str]):
    """Set the active (default) account."""
    if len(args) >= 1:
        username = args[0].strip()
    else:
        data     = load_config_data()
        accounts = data.get("accounts", {})
        if not accounts:
            print_err("No saved accounts. Run 'pesu-wifi add' first.")
            return 1
        print(_c(BOLD, "Select default account:"))
        users = list(accounts.keys())
        for i, u in enumerate(users, 1):
            marker = _c(CYAN, " [active]") if u == data.get("active_user") else ""
            print(f"  {_c(DIM, str(i) + '.')} {u}{marker}")
        try:
            choice = input("  Enter number or username: ").strip()
        except (KeyboardInterrupt, EOFError):
            print("\nCancelled.")
            return 1
        if choice.isdigit() and 1 <= int(choice) <= len(users):
            username = users[int(choice) - 1]
        else:
            username = choice

    data = load_config_data()
    if username not in data.get("accounts", {}):
        print_err(f"Account '{username}' not found. Run 'pesu-wifi list' to see saved accounts.")
        return 1

    data["active_user"] = username
    save_config_data(data)
    print_ok(f"Default account set to '{username}'.")
    return 0

def cmd_list(show_passwords: bool = False):
    data     = load_config_data()
    accounts = data.get("accounts", {})
    active   = data.get("active_user")

    print(_c(BOLD + CYAN, "\nSaved Accounts:"))
    if not accounts:
        print(_c(DIM, "  No accounts saved. Run 'pesu-wifi add' to add one.\n"))
        return 0

    for user, pwd in accounts.items():
        is_active = (user == active)
        marker = _c(CYAN, " [active]") if is_active else ""
        bullet = _c(GREEN, "•") if is_active else _c(DIM, "•")
        pwd_str = f"  {_c(DIM, pwd)}" if show_passwords else ""
        print(f"  {bullet} {_c(BOLD, user) if is_active else user}{marker}{pwd_str}")
    print("")
    return 0

def cmd_wifi(args: list[str]):
    """Scan, select, and connect to a Wi-Fi network."""
    if len(args) >= 1:
        target_ssid = " ".join(args).strip()
    else:
        print_info("Scanning available Wi-Fi networks...")
        lines = []
        try:
            res = subprocess.run(
                ["nmcli", "-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY,BARS", "dev", "wifi", "list", "--rescan", "no"],
                capture_output=True, text=True, timeout=3)
            lines = res.stdout.strip().splitlines()
            if not lines:
                res = subprocess.run(
                    ["nmcli", "-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY,BARS", "dev", "wifi", "list", "--rescan", "yes"],
                    capture_output=True, text=True, timeout=6)
                lines = res.stdout.strip().splitlines()
        except Exception:
            lines = []

        saved_conns = []
        try:
            res2 = subprocess.run(
                ["nmcli", "-t", "-f", "NAME,TYPE", "connection", "show"],
                capture_output=True, text=True, timeout=2)
            for line in res2.stdout.strip().splitlines():
                parts = line.split(":")
                if len(parts) >= 2 and parts[1] == "802-11-wireless":
                    saved_conns.append(parts[0])
        except Exception:
            pass

        seen = set()
        wifi_list = []
        for line in lines:
            parts = line.split(":")
            if len(parts) >= 3:
                in_use = (parts[0].strip() == "*")
                ssid = parts[1].strip()
                signal = parts[2].strip()
                security = parts[3].strip() if len(parts) > 3 else ""
                bars = parts[4].strip() if len(parts) > 4 else ""
                if ssid and ssid not in seen:
                    seen.add(ssid)
                    wifi_list.append({
                        "ssid": ssid,
                        "in_use": in_use,
                        "signal": signal,
                        "security": security or "Open",
                        "bars": bars,
                        "saved": ssid in saved_conns
                    })

        for s in saved_conns:
            if s not in seen:
                seen.add(s)
                wifi_list.append({
                    "ssid": s,
                    "in_use": False,
                    "signal": "??",
                    "security": "",
                    "bars": "",
                    "saved": True
                })

        if not wifi_list:
            print_err("No Wi-Fi networks found.")
            return 1

        print(_c(BOLD + CYAN, "\nAvailable Wi-Fi Networks:"))
        for i, w in enumerate(wifi_list, 1):
            status_flags = []
            if w["in_use"]:
                status_flags.append(_c(GREEN, "[connected]"))
            if w["saved"]:
                status_flags.append(_c(DIM, "[saved]"))
            flag_str = " " + " ".join(status_flags) if status_flags else ""
            sig_str = f"{w['signal']}% {w['bars']}" if w['bars'] else f"{w['signal']}%"
            sec_str = f"[{w['security']}]" if w['security'] else ""
            print(f"  {_c(DIM, str(i) + '.')} {_c(BOLD, w['ssid'])}{flag_str}  {_c(DIM, sig_str)} {sec_str}")
        print("")

        try:
            choice = input("  Enter number or SSID to connect: ").strip()
        except (KeyboardInterrupt, EOFError):
            print("\nCancelled.")
            return 0

        if not choice:
            print("Cancelled.")
            return 0

        if choice.isdigit() and 1 <= int(choice) <= len(wifi_list):
            target_ssid = wifi_list[int(choice) - 1]["ssid"]
        else:
            target_ssid = choice

    print_info(f"Connecting to Wi-Fi '{target_ssid}'...")
    res = subprocess.run(["nmcli", "connection", "up", target_ssid], capture_output=True, text=True)
    if res.returncode != 0:
        res = subprocess.run(["nmcli", "dev", "wifi", "connect", target_ssid], capture_output=True, text=True)

    if res.returncode == 0:
        print_ok(f"Connected to '{target_ssid}'.")
        data = load_config_data()
        data["preferred_ssid"] = target_ssid
        save_config_data(data)
        return 0
    else:
        err_msg = res.stderr.strip() or res.stdout.strip()
        print_err(f"Failed to connect to '{target_ssid}': {err_msg}")
        return 1

def cmd_daemon():
    acquire_daemon_lock()
    username, password = get_active_credentials()
    if not username or not password:
        log(_c(RED, "Error: No credentials saved. Run 'pesu-wifi add' first."))
        sys.exit(1)

    log(f"Starting keepalive watchdog for '{username}' (interval: {KEEP_ALIVE_INTERVAL}s)...")
    unreachable_streak = 0
    consecutive_session_drops = 0

    while True:
        curr_user, curr_pwd = get_active_credentials()
        if curr_user and curr_pwd:
            username, password = curr_user, curr_pwd

        # 1. Check if portal gateway is online
        if not is_portal_online():
            unreachable_streak += 1
            consecutive_session_drops = 0
            log(f"⚠ Portal gateway unreachable (streak: {unreachable_streak}).")
            if unreachable_streak == 3:
                heal_network(1)
                time.sleep(10)
            elif unreachable_streak in (5, 6):
                heal_network(2)
                time.sleep(10)
            elif unreachable_streak >= 9:
                heal_network(3)
                time.sleep(15)
                unreachable_streak = 4
            else:
                time.sleep(20)
            continue

        # 2. Portal is online
        if unreachable_streak > 0:
            log(f"✔ Connectivity restored after {unreachable_streak} failed check(s).")
            unreachable_streak = 0

        # 3. Check if session is live
        if check_live(username):
            consecutive_session_drops = 0
            log(f"Session active ({username}). Next check in {KEEP_ALIVE_INTERVAL}s.")
            time.sleep(KEEP_ALIVE_INTERVAL)
        else:
            consecutive_session_drops += 1
            if consecutive_session_drops < 2:
                log(f"⚠ Keepalive missed 1 check (possible Wi-Fi jitter). Verifying in 5s before re-authenticating...")
                time.sleep(5)
                continue

            consecutive_session_drops = 0
            log(f"Session expired for '{username}'. Logging in...")
            ok, msg = do_login(username, password)
            if ok:
                log(f"✔ Logged in as '{username}'. Next check in {KEEP_ALIVE_INTERVAL}s.")
                time.sleep(KEEP_ALIVE_INTERVAL)
            else:
                log(f"✖ Login failed: {msg}")
                time.sleep(15)

def print_help():
    banner = f"""{_c(BOLD + CYAN, 'PESU WiFi Manager')} {_c(DIM, 'v2.2')}
{_c(DIM, 'Automated captive portal login & keepalive watchdog for PES University.')}

{_c(BOLD, 'USAGE:')}
  pesu-wifi <command> [arguments]

{_c(BOLD, 'COMMANDS:')}
  {_c(GREEN, 'status')}              Show live connection status card
  {_c(GREEN, 'login')} [username]    Smart login; optionally with a specific account
  {_c(GREEN, 'logout')}              Sign out cleanly
  {_c(GREEN, 'select')} [username]   Set default account (alias: {_c(GREEN, 'use')})
  {_c(GREEN, 'wifi')} [ssid]         Select & connect to a Wi-Fi network
  {_c(GREEN, 'add')}                 Save or update login credentials
  {_c(GREEN, 'del')} [username]      Remove a saved account
  {_c(GREEN, 'list')} [-p]           List saved accounts; -p to show passwords
  {_c(GREEN, 'daemon')}              Run keepalive watchdog in foreground ({KEEP_ALIVE_INTERVAL}s polling)
  {_c(GREEN, 'help')}                Show this message

{_c(BOLD, 'EXAMPLES:')}
  pesu-wifi status                  # View live connection overview
  pesu-wifi wifi                    # Interactive Wi-Fi network picker
  pesu-wifi wifi PESU-EC-Campus     # Connect directly to SSID
  pesu-wifi login                   # Login with active account
  pesu-wifi login student1          # Login with a specific account
  pesu-wifi select student1         # Switch active account
  pesu-wifi list -p                 # Show all accounts & passwords
"""
    print(banner)

def main():
    if len(sys.argv) < 2:
        print_help()
        sys.exit(0)

    cmd  = sys.argv[1].lower()
    args = sys.argv[2:]

    if cmd in ("-h", "--help", "help"):
        print_help()
    elif cmd == "status":
        cmd_status()
    elif cmd == "login":
        target = args[0] if args else None
        sys.exit(cmd_login(target))
    elif cmd == "logout":
        sys.exit(cmd_logout())
    elif cmd in ("select", "use"):
        sys.exit(cmd_select(args))
    elif cmd in ("wifi", "select-wifi", "connect"):
        sys.exit(cmd_wifi(args))
    elif cmd == "add":
        sys.exit(cmd_add(args))
    elif cmd == "del":
        sys.exit(cmd_del(args))
    elif cmd == "list":
        show_pw = "-p" in args or "--passwords" in args
        sys.exit(cmd_list(show_pw))
    elif cmd == "daemon":
        cmd_daemon()
    else:
        print_err(f"Unknown command '{cmd}'.")
        print_help()
        sys.exit(1)

if __name__ == "__main__":
    main()
