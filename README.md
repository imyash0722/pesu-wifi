# PESU WiFi Login Manager & Watchdog Daemon

A lightweight, high-performance automated captive portal login manager and keepalive watchdog daemon for PES University campus Wi-Fi networks.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![AUR package](https://img.shields.io/badge/AUR-pesu--wifi--git-blue.svg)](https://aur.archlinux.org/packages/pesu-wifi-git)

---

## Features

- **Automated Keepalive Watchdog:** Heartbeat polling keeps your captive portal session alive indefinitely, even when browsers are closed or devices idle.
- **Smart Status Detection:** Probes the local gateway in sub-second time (10–30ms) to detect session states without redundant network traffic or false error alerts.
- **Interactive Wi-Fi Selector:** Built-in scanner scans available nearby access points, reports signal strength and security types, and connects directly via NetworkManager.
- **Multi-Account Switching:** Save multiple accounts, switch the active user instantly, or explicitly log in under a specific student account.
- **Secure Lightweight Credential Storage:** Stores credentials locally in `~/.config/pesu-wifi/config.json` and `~/.config/pesu-wifi/.env` with strict `0600` permissions.
- **Resilient Multi-Tier Self-Healing:**
  - **Tier 1:** NetworkManager connection renegotiation.
  - **Tier 2:** Wi-Fi radio power-cycling.
  - **Tier 3:** NetworkManager service recovery.
- **Proxy Bypass:** Bypasses local HTTP proxies that interfere with local gateway negotiation.
- **Modern CLI:** ANSI-colored status cards, intuitive commands, and detailed test diagnostics.

---

## Installation

### Option 1: Arch User Repository (Arch Linux / CachyOS)
```bash
paru -S pesu-wifi-git
# OR
yay -S pesu-wifi-git
```

### Option 2: One-Click Installer (Any Linux Distribution)
```bash
# Clone the repository
git clone https://github.com/imyash0722/pesu-wifi.git ~/pesu-wifi
cd ~/pesu-wifi

# Run the installer
chmod +x install.sh
./install.sh
```

---

## Setup & Credential Management

Add your login credentials securely:
```bash
pesu-wifi add
```
```text
Enter login credentials:
  username: <your_username>
  password: <your_password>
```

Credentials are saved in `~/.config/pesu-wifi/config.json` and `~/.config/pesu-wifi/.env` with strict `0600` (user-only read/write) permissions.

---

## CLI Command Reference

### Connection & Status
```bash
# View live connection card (gateway, SSID, session state, active user, daemon PID)
pesu-wifi status

# Interactive Wi-Fi network scanner and picker
pesu-wifi wifi

# Connect directly to a specific Wi-Fi SSID
pesu-wifi wifi PESU-EC-Campus
```

### Authentication & Sessions
```bash
# Smart login using the active account
pesu-wifi login

# Explicitly login using a specific saved account
pesu-wifi login <username>

# Cleanly log out from the captive portal
pesu-wifi logout
```

### Account Management
```bash
# Interactively select or switch the default active account
pesu-wifi select

# Directly set active account (alias: use)
pesu-wifi use <username>

# List saved accounts
pesu-wifi list

# List saved accounts with passwords visible
pesu-wifi list -p

# Add or update an account
pesu-wifi add

# Remove a saved account
pesu-wifi del <username>
```

### Background Watchdog Daemon
```bash
# Run daemon in the foreground
pesu-wifi daemon

# Manage the systemd user service
systemctl --user start pesu-wifi.service
systemctl --user stop pesu-wifi.service
systemctl --user restart pesu-wifi.service

# View live daemon logs
journalctl --user -u pesu-wifi.service -f
```

---

## Automated Verification Suite

An automated diagnostic script is included to test end-to-end network connectivity, multi-account logins, throughput latency, and connection stability:
```bash
python3 test_wifi_accounts.py
```

The test suite performs:
1. **Network link verification:** Connects to the preferred Wi-Fi SSID and tests gateway response times.
2. **Account rotation test:** Iterates through every saved credential, logs in, performs real-world HTTP checks, tests a 4-second connection stability hold, and logs out cleanly.
3. **Restoration:** Restores the original active account, verifies internet connectivity, resumes `pesu-wifi.service`, and outputs a telemetry report to `/tmp/pesu_wifi_test_report.json`.

---

## Environment Variable Overrides

You can override defaults without modifying configuration files:

| Variable | Description | Default |
|---|---|---|
| `PESU_USERNAME` | Override active login username | *(Config file)* |
| `PESU_PASSWORD` | Override active login password | *(Config file)* |
| `PESU_PORTAL_BASE` | Portal gateway URL | `http://192.168.254.1:8090` |
| `PESU_WIFI_CON` | Preferred Wi-Fi connection name | `PESU-EC-Campus` |
| `FORCE_COLOR` | Force ANSI color formatting (`1` / `0`) | Auto-detected |

---

## License

MIT License. See [LICENSE](LICENSE) for details.
