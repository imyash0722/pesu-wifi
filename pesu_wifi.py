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

def print_ok(msg):    print(_c(GREEN, f"✔  {msg}"))
def print_err(msg):   print(_c(RED, f"✖  {msg}"))
def print_warn(msg):  print(_c(YELLOW, f"⚠  {msg}"))
def print_info(msg):  print(_c(CYAN, f"➜  {msg}"))

def get_time_str():
    return datetime.now().strftime("[%H:%M:%S]")

def log(msg):
    print(f"{_c(DIM, get_time_str())} {msg}", flush=True)

# ── Configuration & Paths ─────────────────────────────────────────────────────
PORTAL_BASE = os.getenv("PESU_PORTAL_BASE", "http://192.168.254.1:8090")
LOGIN_URL = f"{PORTAL_BASE}/login.xml"
LOGOUT_URL = f"{PORTAL_BASE}/logout.xml"
LIVE_URL = f"{PORTAL_BASE}/live"
CONNECTIVITY_URL = "http://connectivitycheck.gstatic.com/generate_204"
DEFAULT_WIFI_CON = os.getenv("PESU_WIFI_CON", "PESU-EC-Campus")
LOCK_FILE = "/tmp/pesu_wifi_daemon.lock"
KEEP_ALIVE_INTERVAL = 60  # Polling rate in seconds

def get_config_dir() -> str:
    xdg_config = os.getenv("XDG_CONFIG_HOME")
    if xdg_config:
        return os.path.join(xdg_config, "pesu-wifi")
    return os.path.join(os.path.expanduser("~"), ".config", "pesu-wifi")

def get_config_candidates() -> list[str]:
    candidates = []
    # 1. Active user config
    candidates.append(os.path.join(get_config_dir(), "config.json"))
    # 2. Sudo invoking user config if running under sudo
    sudo_user = os.getenv("SUDO_USER")
    if sudo_user:
        candidates.append(f"/home/{sudo_user}/.config/pesu-wifi/config.json")
    # 3. System-wide config
    candidates.append("/etc/pesu-wifi/config.json")
    return candidates

def get_env_candidates() -> list[str]:
    candidates = []
    candidates.append(os.path.join(get_config_dir(), ".env"))
    sudo_user = os.getenv("SUDO_USER")
    if sudo_user:
        candidates.append(f"/home/{sudo_user}/.config/pesu-wifi/.env")
    candidates.append("/etc/pesu-wifi/.env")
    candidates.append(".env")
    return candidates

def load_config_data() -> dict:
    # Check JSON configs
    for path in get_config_candidates():
        if os.path.isfile(path):
            try:
                with open(path, "r", encoding="utf-8") as f:
                    return json.load(f)
            except Exception:
                pass

    # Fallback to .env configs
    for path in get_env_candidates():
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

    # Fallback to environment variables
    env_usr = os.getenv("PESU_USERNAME")
    env_pwd = os.getenv("PESU_PASSWORD")
    if env_usr and env_pwd:
        return {"active_user": env_usr, "accounts": {env_usr: env_pwd}}

    return {"active_user": None, "accounts": {}}

def save_config_data(data: dict):
    conf_dir = get_config_dir()
    os.makedirs(conf_dir, exist_ok=True)
    json_path = os.path.join(conf_dir, "config.json")
    env_path = os.path.join(conf_dir, ".env")

    # Write JSON
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
    os.chmod(json_path, 0o600)

    # Write synchronized .env
    active = data.get("active_user")
    pwd = data.get("accounts", {}).get(active, "") if active else ""
    with open(env_path, "w", encoding="utf-8") as f:
        f.write(f"# PESU WiFi Saved Credentials\nPESU_USERNAME={active or ''}\nPESU_PASSWORD={pwd or ''}\n")
    os.chmod(env_path, 0o600)

def get_active_credentials() -> tuple[str | None, str | None]:
    data = load_config_data()
    active = data.get("active_user")
    accounts = data.get("accounts", {})
    if active and active in accounts:
        return active, accounts[active]
    if accounts:
        first_user = next(iter(accounts))
        return first_user, accounts[first_user]
    return None, None

# ── Session & Network Engine ──────────────────────────────────────────────────
HEADERS = {
    "User-Agent": "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0",
    "Accept-Language": "en-US,en;q=0.5",
}

session = requests.Session()
session.headers.update(HEADERS)
session.trust_env = False

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
        test_fd = open(LOCK_FILE, "a+")
        fcntl.flock(test_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        fcntl.flock(test_fd, fcntl.LOCK_UN)
        test_fd.close()
        return False, None
    except (IOError, OSError):
        # Read PID from lock file if possible
        try:
            r = subprocess.run(["pgrep", "-f", "pesu_wifi.py daemon"], capture_output=True, text=True)
            pids = [int(p) for p in r.stdout.strip().split() if p and int(p) != os.getpid()]
            return True, pids[0] if pids else None
        except Exception:
            return True, None

def get_timestamp() -> int:
    return int(time.time() * 1000)

def clean_message(raw_msg: str) -> str:
    if not raw_msg:
        return ""
    return html.unescape(raw_msg).strip()

def get_active_wifi_ssid() -> str:
    try:
        res = subprocess.run(
            ["nmcli", "-t", "-f", "name,type", "connection", "show", "--active"],
            capture_output=True, text=True, timeout=3
        )
        for line in res.stdout.strip().splitlines():
            if ":802-11-wireless" in line:
                return line.split(":")[0]
    except Exception:
        pass
    return DEFAULT_WIFI_CON

def check_internet() -> bool:
    try:
        r = session.get(CONNECTIVITY_URL, timeout=4)
        return r.status_code == 204
    except Exception:
        return False

def check_live(username: str | None = None) -> bool | None:
    """
    Check if session is currently active on the portal.
    Returns:
        True  -> Logged in
        False -> Signed out / session expired
        None  -> Portal unreachable
    """
    if not username:
        username, _ = get_active_credentials()
        username = username or "user"

    params = {
        "mode": 192,
        "username": username,
        "a": get_timestamp(),
        "producttype": 0,
    }
    try:
        r = session.get(LIVE_URL, params=params, timeout=5)
        if r.status_code == 200:
            root = ET.fromstring(r.text)
            ack = (root.findtext("ack") or "").strip().lower()
            status = (root.findtext("status") or "").strip().lower()
            if ack == "ack" or "live" in status or "ok" in status:
                return True
            return False
        return None
    except Exception:
        if check_internet():
            return True
        return None

def do_login(username: str, password: str) -> tuple[bool, str]:
    data = {
        "mode": 191,
        "username": username,
        "password": password,
        "a": get_timestamp(),
        "producttype": 0,
    }
    try:
        r = session.post(LOGIN_URL, data=data, timeout=8)
        root = ET.fromstring(r.text)
        message = clean_message(root.findtext("message") or "")
        ack = (root.findtext("ack") or "").strip().lower()

        if "signed in" in message.lower() or "you are signed in as" in message.lower() or ack == "ack":
            return True, message or f"Signed in as {username}"
        elif message:
            return ("signed in" in message.lower()), message
        return True, "Login OK"
    except Exception as e:
        return False, f"Portal unreachable ({e})"

def do_logout(username: str | None = None) -> tuple[bool, str]:
    if not username:
        username, _ = get_active_credentials()
        username = username or "user"

    data = {
        "mode": 193,
        "username": username,
        "a": get_timestamp(),
        "producttype": 0,
    }
    try:
        r = session.post(LOGOUT_URL, data=data, timeout=8)
        root = ET.fromstring(r.text)
        message = clean_message(root.findtext("message") or "")
        return True, message if message else "Signed out successfully"
    except Exception as e:
        return False, f"Portal unreachable ({e})"

def heal_network(tier: int):
    wifi_con = get_active_wifi_ssid()
    try:
        if tier == 1:
            log(f"[Self-Healing L1] Reconnecting to '{wifi_con}'...")
            subprocess.run(["nmcli", "connection", "up", wifi_con], timeout=20)
        elif tier == 2:
            log(f"[Self-Healing L2] Cycling Wi-Fi radio...")
            subprocess.run(["nmcli", "radio", "wifi", "off"], timeout=10)
            time.sleep(2)
            subprocess.run(["nmcli", "radio", "wifi", "on"], timeout=10)
            time.sleep(5)
            subprocess.run(["nmcli", "connection", "up", wifi_con], timeout=20)
        elif tier == 3:
            log(f"[Self-Healing L3] Restarting NetworkManager...")
            subprocess.run(["systemctl", "restart", "NetworkManager"], timeout=20)
            time.sleep(6)
            subprocess.run(["nmcli", "connection", "up", wifi_con], timeout=20)
    except Exception as err:
        log(f"Self-healing Tier {tier} execution error: {err}")

# ── User Commands ─────────────────────────────────────────────────────────────

def cmd_status():
    username, _ = get_active_credentials()
    portal_alive = None
    try:
        r = session.get(PORTAL_BASE, timeout=3)
        portal_alive = (r.status_code == 200)
    except Exception:
        portal_alive = False

    session_status = check_live(username)
    daemon_active, daemon_pid = is_daemon_running()
    wifi_ssid = get_active_wifi_ssid()

    gw_status = _c(GREEN, "[Online]") if portal_alive else _c(RED, "[Unreachable]")
    gw_val = f"{PORTAL_BASE} {gw_status}"

    if session_status is True:
        s_val = _c(BOLD + GREEN, "LOGGED IN")
    elif session_status is False:
        s_val = _c(BOLD + YELLOW, "SIGNED OUT")
    else:
        s_val = _c(BOLD + RED, "UNREACHABLE")

    acc_val = _c(GREEN, username) if username else _c(YELLOW, "(None - run 'pesu-wifi add')")

    if daemon_active:
        pid_str = f" (PID: {daemon_pid})" if daemon_pid else ""
        d_val = f"{_c(GREEN, 'Active')}{_c(DIM, pid_str)} {_c(DIM, f'[polling: {KEEP_ALIVE_INTERVAL}s]')}"
    else:
        d_val = _c(DIM, f"Inactive [polling: {KEEP_ALIVE_INTERVAL}s]")

    rows = [
        ("Portal Gateway", gw_val),
        ("Wi-Fi Network", wifi_ssid),
        ("Session State", s_val),
        ("Active Account", acc_val),
        ("Daemon Watcher", d_val),
    ]

    label_width = 16
    max_val_len = max(visual_len(v) for _, v in rows)
    inner_width = max(54, label_width + 4 + max_val_len)

    top_border = _c(BOLD + CYAN, "╭── PESU WiFi Status " + ("─" * (inner_width - 17)) + "╮")
    bot_border = _c(BOLD + CYAN, "╰" + ("─" * (inner_width + 4)) + "╯")

    print("")
    print(top_border)
    for label, val in rows:
        vlen = label_width + 3 + visual_len(val)
        pad = " " * max(0, inner_width - vlen)
        border_l = _c(BOLD + CYAN, "│")
        border_r = _c(BOLD + CYAN, "│")
        print(f"{border_l}  {_c(BOLD, label.ljust(label_width))} : {val}{pad}  {border_r}")
    print(bot_border)
    print("")

def cmd_login():
    username, password = get_active_credentials()
    if not username or not password:
        print_err("No credentials configured.")
        print(_c(YELLOW, "  Please run 'pesu-wifi add' to save your login credentials."))
        return 1

    # Requirement 4: Call status check first
    status = check_live(username)
    if status is True:
        print_ok(f"Already logged in. Active session detected for '{username}'.")
        return 0
    elif status is False:
        print_info(f"Session inactive. Logging in as '{username}'...")
    else:
        print_warn(f"Portal appears unreachable at {PORTAL_BASE}.")
        print_info(f"Attempting login request for '{username}'...")

    success, msg = do_login(username, password)

    # Re-verify status
    verify_status = check_live(username)
    if verify_status is True or success:
        print_ok(f"Successfully logged in as '{username}'.")
        return 0
    else:
        print_err(f"Login failed: {msg}")
        return 1

def cmd_logout():
    username, _ = get_active_credentials()

    # Requirement 5: Call status check first
    status = check_live(username)
    if status is False:
        print_ok("Already logged out. No active session found.")
        return 0
    elif status is None:
        print_warn(f"Portal unreachable at {PORTAL_BASE}. Cannot verify session status.")
    else:
        print_info(f"Active session found. Logging out '{username or 'user'}'...")

    success, msg = do_logout(username)

    # Re-verify status
    verify_status = check_live(username)
    if verify_status is False or success:
        print_ok("Successfully logged out.")
        return 0
    else:
        print_err(f"Logout failed: {msg}")
        return 1

def cmd_add(args: list[str]):
    # Requirement 2: Interactive or argument-based credential adding
    if len(args) >= 2:
        username = args[0].strip()
        password = args[1].strip()
    else:
        print(_c(BOLD, "Enter login credentials:"))
        try:
            username = input("- username: ").strip()
            password = getpass.getpass("- password: ").strip()
        except (KeyboardInterrupt, EOFError):
            print("\nCancelled.")
            return 1

    if not username:
        print_err("Username cannot be empty.")
        return 1
    if not password:
        print_err("Password cannot be empty.")
        return 1

    config = load_config_data()
    if "accounts" not in config:
        config["accounts"] = {}

    config["accounts"][username] = password
    config["active_user"] = username
    save_config_data(config)

    print_ok(f"Saved credentials for '{username}'. Set as active account.")
    return 0

def cmd_del(args: list[str]):
    # Requirement 3: Interactive or argument-based credential deletion
    if len(args) >= 1:
        username = args[0].strip()
    else:
        try:
            username = input("Enter username: ").strip()
        except (KeyboardInterrupt, EOFError):
            print("\nCancelled.")
            return 1

    if not username:
        print_err("Username cannot be empty.")
        return 1

    config = load_config_data()
    accounts = config.get("accounts", {})

    if username not in accounts:
        print_err(f"User '{username}' not found in saved accounts.")
        return 1

    del accounts[username]
    if config.get("active_user") == username:
        config["active_user"] = next(iter(accounts)) if accounts else None

    config["accounts"] = accounts
    save_config_data(config)

    print_ok(f"Removed credentials for '{username}'.")
    return 0

def cmd_list():
    config = load_config_data()
    accounts = config.get("accounts", {})
    active = config.get("active_user")

    print(_c(BOLD + CYAN, "\nSaved Accounts:"))
    if not accounts:
        print(_c(DIM, "  No accounts saved. Run 'pesu-wifi add' to add one.\n"))
        return 0

    for user in accounts:
        if user == active:
            print(f"  • {_c(BOLD + GREEN, user)} {_c(CYAN, '[active]')}")
        else:
            print(f"  • {user}")
    print("")
    return 0

def cmd_daemon():
    acquire_daemon_lock()
    username, password = get_active_credentials()
    if not username or not password:
        log(f"{_c(RED, 'Error:')} No credentials saved. Run 'pesu-wifi add' first.")
        sys.exit(1)

    log(f"Starting resilient keepalive watchdog loop for '{username}' (interval: {KEEP_ALIVE_INTERVAL}s)...")
    unreachable_count = 0

    while True:
        # Reload credentials in case user updated them via 'pesu-wifi add'
        curr_user, curr_pwd = get_active_credentials()
        if curr_user and curr_pwd:
            username, password = curr_user, curr_pwd

        status = check_live(username)

        if status is True:
            if unreachable_count > 0:
                log(f"✔ Connectivity fully restored after {unreachable_count} failed attempt(s).")
                unreachable_count = 0
            log(f"Session active ({username}). Next check in {KEEP_ALIVE_INTERVAL}s.")
            time.sleep(KEEP_ALIVE_INTERVAL)
        elif status is False:
            unreachable_count = 0
            log("Session expired. Attempting login...")
            ok, msg = do_login(username, password)
            if ok:
                log(f"✔ Logged in as '{username}'.")
            else:
                log(f"Login failed: {msg}")
            time.sleep(15)
        else:
            unreachable_count += 1
            log(f"⚠ Portal unreachable / network dropped (streak: {unreachable_count}).")

            if unreachable_count <= 2:
                log("Attempting fallback login request...")
                do_login(username, password)
                time.sleep(15)
            elif unreachable_count == 3:
                heal_network(1)
                time.sleep(8)
                do_login(username, password)
                time.sleep(10)
            elif unreachable_count in (5, 6):
                heal_network(2)
                time.sleep(8)
                do_login(username, password)
                time.sleep(10)
            elif unreachable_count >= 8:
                heal_network(3)
                time.sleep(8)
                do_login(username, password)
                time.sleep(15)
                unreachable_count = 4
            else:
                time.sleep(15)

def print_help():
    banner = f"""{_c(BOLD + CYAN, 'PESU WiFi Manager')} {_c(DIM, 'v2.0')}
{_c(DIM, 'Automated captive portal login manager and keepalive watchdog.')}

{_c(BOLD, 'USAGE:')}
  pesu-wifi <command> [arguments]

{_c(BOLD, 'COMMANDS:')}
  {_c(GREEN, 'status')}       Show live network, portal, and session status card
  {_c(GREEN, 'login')}        Check status and log in if session is inactive
  {_c(GREEN, 'logout')}       Check status and sign out cleanly
  {_c(GREEN, 'add')}          Save or update login credentials interactively
  {_c(GREEN, 'del')}          Remove a saved account
  {_c(GREEN, 'list')}         Display saved accounts
  {_c(GREEN, 'daemon')}       Run persistent keepalive watchdog loop ({KEEP_ALIVE_INTERVAL}s polling)
  {_c(GREEN, 'help')}         Display this help message

{_c(BOLD, 'EXAMPLES:')}
  pesu-wifi add               # Prompt to enter username and password
  pesu-wifi login             # Smart login with pre-status check
  pesu-wifi status            # View current connection overview
"""
    print(banner)

def main():
    if len(sys.argv) < 2:
        print_help()
        sys.exit(0)

    cmd = sys.argv[1].lower()
    args = sys.argv[2:]

    if cmd in ("-h", "--help", "help"):
        print_help()
    elif cmd == "status":
        cmd_status()
    elif cmd == "login":
        sys.exit(cmd_login())
    elif cmd == "logout":
        sys.exit(cmd_logout())
    elif cmd == "add":
        sys.exit(cmd_add(args))
    elif cmd == "del":
        sys.exit(cmd_del(args))
    elif cmd == "list":
        sys.exit(cmd_list())
    elif cmd == "daemon":
        cmd_daemon()
    else:
        print_err(f"Unknown command '{cmd}'.")
        print_help()
        sys.exit(1)

if __name__ == "__main__":
    main()
