# PESU WiFi Login Manager & Watchdog Daemon

A lightweight, high-performance automated captive portal login manager and keepalive watchdog daemon for PES University campus Wi-Fi networks.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Release](https://img.shields.io/github/v/release/imyash0722/pesu-wifi?color=blue)](https://github.com/imyash0722/pesu-wifi/releases)
[![Arch Package](https://img.shields.io/badge/Arch%20Linux-.pkg.tar.zst-1793d1.svg)](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.3.0)
[![CI](https://github.com/imyash0722/pesu-wifi/actions/workflows/ci.yml/badge.svg)](https://github.com/imyash0722/pesu-wifi/actions/workflows/ci.yml)
[![Changelog](https://img.shields.io/badge/Changelog-Release%20Notes-green.svg)](CHANGELOG.md)

---

## Features

- **Automated Keepalive Watchdog:** Heartbeat polling keeps your captive portal session alive indefinitely, even when browsers are closed or devices idle.
- **Campus SSID Auto-Standby:** Automatically detects whether you are connected to a campus network. Gracefully pauses watchdog activities when at home or on non-campus Wi-Fi to preserve network stability.
- **Desktop Notifications:** Dispatches native system notifications (`notify-send`) on login, session renewal, and authentication errors.
- **Sub-Second Status Detection:** Probes the local gateway in 10–30ms to detect session states without redundant network traffic or false timeout warnings.
- **Interactive Wi-Fi Selector:** Built-in scanner scans available nearby access points, reports signal strength and security types, and connects directly via NetworkManager.
- **Multi-Account Switching:** Save multiple student accounts, switch the active user instantly, or explicitly log in under a specific account.
- **Secure Atomic Credential Storage:** Stores credentials locally in `~/.config/pesu-wifi/config.json` and `~/.config/pesu-wifi/.env` with atomic `0600` permissions.
- **Resilient Multi-Tier Self-Healing:**
  - **Tier 1:** NetworkManager connection renegotiation.
  - **Tier 2:** Wi-Fi radio power-cycling.
  - **Tier 3:** Non-root Wi-Fi interface recovery.
- **Shell Auto-Completions:** Native tab-completion support for Bash, Zsh, and Fish shells.
- **Proxy Bypass:** Bypasses campus HTTP proxies that interfere with local gateway negotiation.
- **Modern CLI:** Clean ANSI status cards, colored indicators, and comprehensive verification suite.

---

## Installation

### Method 1: Direct Pacman Install (Arch Linux / CachyOS)
Install the pre-built release package directly using `pacman`:
```bash
sudo pacman -U https://github.com/imyash0722/pesu-wifi/releases/download/v2.3.0/pesu-wifi-2.3.0-any.pkg.tar.zst
```
*Or download the `.pkg.tar.zst` asset directly from [**GitHub Releases**](https://github.com/imyash0722/pesu-wifi/releases).*

---

### Method 2: Python pip / pipx (Any Linux Distribution)
```bash
pipx install git+https://github.com/imyash0722/pesu-wifi.git
```

---

### Method 3: One-Click Installer (Clone & Script)
```bash
# Clone the repository
git clone https://github.com/imyash0722/pesu-wifi.git ~/pesu-wifi
cd ~/pesu-wifi

# Run installer
chmod +x install.sh
./install.sh
```

> **Note on AUR:** The PKGBUILD is ready in [`aur/`](aur/) and will be published to the Arch User Repository once AUR account registrations reopen. In the meantime, install directly via the pre-built `.pkg.tar.zst` release above.

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
# Start background keepalive daemon via systemd
pesu-wifi start

# Stop background keepalive daemon
pesu-wifi stop

# Restart background keepalive daemon
pesu-wifi restart

# Run daemon in the foreground
pesu-wifi daemon

# View live daemon logs
journalctl --user -u pesu-wifi.service -f

# Check version
pesu-wifi version
```

---

## Testing & Verification Suite

### Offline Unit Tests
Run the mock-based offline unit test suite (requires no network or campus Wi-Fi):
```bash
python3 -m unittest discover tests -p "test_unit.py"
```

### End-to-End Campus Verification Suite
When on campus, run the automated integration diagnostic script in [`tests/`](tests/) to test live link latency, multi-account rotation, and connection hold stability:
```bash
python3 tests/test_wifi_accounts.py
```

---

## Repository Structure

```text
pesu-wifi/
├── pesu_wifi.py             # Core CLI executable & keepalive watchdog daemon
├── install.sh               # Standalone system installer and service configurator
├── pyproject.toml           # Standard Python package configuration
├── pesu-wifi.service        # Systemd user service unit definition
├── completions/             # Shell completion definitions
│   ├── pesu-wifi.bash       # Bash auto-completions
│   ├── pesu-wifi.zsh        # Zsh auto-completions
│   └── pesu-wifi.fish       # Fish auto-completions
├── aur/
│   ├── PKGBUILD             # Arch User Repository package build definition
│   ├── .SRCINFO             # Arch User Repository package metadata
│   └── pesu-wifi.install    # Arch package post-install instructions
├── tests/
│   ├── test_unit.py         # Offline mock-based unit tests
│   └── test_wifi_accounts.py# Automated multi-account & network verification suite
├── .github/workflows/
│   └── ci.yml               # Automated multi-python GitHub Actions CI
├── LICENSE                  # MIT License
└── README.md                # Documentation and guide
```

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

## Uninstallation

To remove `pesu-wifi`:

**If installed via AUR or `.pkg.tar.zst`:**
```bash
sudo pacman -R pesu-wifi-git
```

**If installed via pipx:**
```bash
pipx uninstall pesu-wifi
```

**If installed via `install.sh`:**
```bash
systemctl --user disable --now pesu-wifi.service
rm -f ~/.local/bin/pesu-wifi ~/.config/systemd/user/pesu-wifi.service
systemctl --user daemon-reload
```

---

## Release History & Changelog

See [**CHANGELOG.md**](CHANGELOG.md) for full commit-by-commit technical breakdowns, architectural notes, and upgrade guides for every release.

- [**v2.3.0**](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.3.0) — Process controls (`start`/`stop`/`restart`), campus SSID auto-standby, desktop notifications, atomic umask hardening, anti-storm jitter protection, and multi-packaging CI.
- [**v2.2.0**](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.2.0) — Interactive Wi-Fi selector, sub-second gateway detection, multi-account credentials management, and verification suite.
- [**v2.0.0**](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.0.0) — Initial rewritten Python CLI release with automated Cyberoam login & keepalive.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
