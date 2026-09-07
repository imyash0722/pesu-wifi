# PESU WiFi Login Manager & Watchdog Daemon

A lightweight, automated captive portal login manager and keepalive watchdog daemon for PES University WiFi networks.

## Features

- **Automated Keepalive Heartbeat:** Pings `/live` every 60 seconds to keep your session alive even when the browser is closed.
- **Smart Login & Logout:** Checks live portal state before sending requests to avoid redundant traffic or false errors.
- **Secure Lightweight Credential Storage:** Store login credentials locally in `~/.config/pesu-wifi/config.json` and `.env` (`chmod 600`).
- **Multi-Account Management:** Add or delete multiple accounts with `pesu-wifi add` and `pesu-wifi del`.
- **Resilient Self-Healing Watchdog:** Multi-tier recovery (L1 reconnection, L2 Wi-Fi radio cycle, L3 NetworkManager service restart).
- **Proxy Bypass:** Bypasses local proxy environment variables that disrupt captive portal connections.
- **Modern CLI:** Clean status cards, colored indicators, and intuitive commands.

---

## Installation

### Method 1: AUR (Arch Linux / CachyOS)
```bash
paru -S pesu-wifi-git
# OR
yay -S pesu-wifi-git
```

### Method 2: One-Click Installer
```bash
# Clone the repository
git clone https://github.com/imyash0722/pesu-wifi.git ~/pesu-wifi
cd ~/pesu-wifi

# Run installer
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
- username: <your_username>
- password: <your_password>
```

Credentials are saved in `~/.config/pesu-wifi/config.json` and `~/.config/pesu-wifi/.env` with strict `0600` permissions.

---

## Commands

```bash
# View network & live session status card
pesu-wifi status

# Interactive Wi-Fi network scanner and connector
pesu-wifi wifi
# Or connect directly to an SSID
pesu-wifi wifi PESU-EC-Campus

# Smart login (uses active account)
pesu-wifi login
# Login using a specific saved account
pesu-wifi login deltatime-1

# Smart logout (verifies status first)
pesu-wifi logout

# Select / switch active account (alias: pesu-wifi use)
pesu-wifi select
pesu-wifi use deltatime-1

# List saved accounts (-p to reveal passwords)
pesu-wifi list
pesu-wifi list -p

# Add or update credentials
pesu-wifi add

# Remove a saved account
pesu-wifi del deltatime-1

# View daemon logs
journalctl --user -u pesu-wifi -f
```

---

## Configuration & Overrides

You can also override settings via environment variables:

- `PESU_USERNAME`: Override active username
- `PESU_PASSWORD`: Override active password
- `PESU_PORTAL_BASE`: Override portal gateway (default: `http://192.168.254.1:8090`)
- `PESU_WIFI_CON`: Override target Wi-Fi connection name (default: `PESU-EC-Campus`)
