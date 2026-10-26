# PESU WiFi — Unified Multi-Platform Captive Portal Engine

A blazing fast, native multi-platform captive portal login manager and continuous keepalive watchdog daemon for PES University campus Wi-Fi networks (RR Campus, EC Campus & Amaatra Hostel).

[![Latest Release](https://img.shields.io/github/v/release/imyash0722/pesu-wifi?color=blue)](https://github.com/imyash0722/pesu-wifi/releases/latest)
[![Website](https://img.shields.io/badge/Website-GitHub_Pages-6C6E36.svg)](https://imyash0722.github.io/pesu-wifi/)
[![Setup Guide](https://img.shields.io/badge/Setup_Guide-Visual_Instructions-E3BD90.svg)](https://imyash0722.github.io/pesu-wifi/instructions.html)
[![iOS Guide](https://img.shields.io/badge/iOS_Guide-Apple_Shortcuts-92e4c5.svg)](https://imyash0722.github.io/pesu-wifi/ios.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Supported Platforms

| Platform | Interface | Engine | Status | Download |
| :--- | :--- | :--- | :--- | :--- |
| **Android** (8–16) | Material 3 Jetpack Compose | Foreground Service (`specialUse`) + Exact RTC Watchdog | Stable v3.7.0 | [Download APK](https://github.com/imyash0722/pesu-wifi/releases/latest) |
| **Windows** (10/11) | Win32 Tray / CLI | Native Rust + Win32 WlanAPI + Inno Setup | Stable v3.7.0 | [Download .exe](https://github.com/imyash0722/pesu-wifi/releases/latest) |
| **Linux** (x86_64) | CLI + systemd | Native Rust daemon + `/32` host route bypass | Stable v3.7.0 | [Download tar.gz](https://github.com/imyash0722/pesu-wifi/releases/latest) |
| **iOS / iPadOS** | Native Shortcuts | Apple Shortcuts Personal Automation | Live Guide | [View Guide](https://imyash0722.github.io/pesu-wifi/ios.html) |
| **macOS** | CLI / LaunchAgent | Native Rust CLI | Compatible | Build from Source |

---

## Key Features

- **Full VPN Compatibility (WireGuard, Tailscale, WARP, OpenVPN, AdGuard):**
  - **Linux & Windows Desktop:** Automatically provisions host `/32` routes for gateway endpoints (`192.168.254.1/32`, `172.16.1.1/32`, `10.14.142.1/32`) bound to the physical Wi-Fi interface. Prevents VPN default routes (`0.0.0.0/0`) from dropping local captive portal packets.
  - **Android:** Custom `ResilientSocketFactory` intercepts kernel `EPERM (Operation not permitted)` during socket `connect()` and gracefully routes traffic via RFC-1918 gateway bypass. Fallback to `LinkProperties.dhcpServerAddress` on Android 11+ (API 30+).
- **Smart Multi-Account Auto-Failover:** Automatically cascades to backup accounts when the active user hits concurrent device limits (code 529), daily data transfer quotas, or account expiration.
- **Continuous Jittered Keepalive Watchdog:**
  - Sends XML keepalive heartbeats every 120s–180s with randomized anti-storm jitter (`mode=192`) satisfying Cyberoam's strict Dead Client Detection (DCD) while preventing server-side DDoS conditions.
- **Deep Sleep & Standby Immunity:**
  - **Android 15 & 16:** Exempt from 6-hour cumulative background limits via `FOREGROUND_SERVICE_SPECIAL_USE` (`specialUse`). Synchronized coroutine WakeLocks prevent Doze CPU sleep drops mid-probe.
  - **Windows & Linux Desktop:** Inhibits system sleep and disables Wi-Fi 802.11 power saving while associated with campus networks.
- **Sub-Second Gateway Discovery:** Probes candidate gateways (`192.168.254.1:8090` and `172.16.1.1:8090`) with self-signed SSL certificate handling for Sophos firewalls.
- **Instant Roaming Re-Auth (< 150ms):** Automatically detects physical Access Point transitions across campus blocks and re-authenticates with zero dropped packets.

---

## Quick Installation

### 1. Android Phone
Download the **[Latest Stable APK](https://github.com/imyash0722/pesu-wifi/releases/latest)** or check our **[Visual Setup Guide](https://imyash0722.github.io/pesu-wifi/instructions.html)**.

### 2. Windows PC (10 / 11)
Run in PowerShell (per-user install, no admin prompt needed):
```powershell
irm https://raw.githubusercontent.com/imyash0722/pesu-wifi/master/install.ps1 | iex
```
Or download the standalone Inno Setup **[`pesu-wifi-v3.7.0-windows-x86_64-setup.exe`](https://github.com/imyash0722/pesu-wifi/releases/latest)** from GitHub Releases.

### 3. Linux (Arch / Ubuntu / Fedora / Debian)
Run in your terminal:
```bash
curl -sSL https://raw.githubusercontent.com/imyash0722/pesu-wifi/master/install.sh | bash
```
Or on Arch Linux / CachyOS via AUR:
```bash
yay -S pesu-wifi-bin
```

### 4. Apple iPhone / iPad (iOS)
Follow our step-by-step native **[iOS Automation Guide](https://imyash0722.github.io/pesu-wifi/ios.html)** to configure zero-click sign-in via Apple Shortcuts Personal Automations.

---

## Repository Structure

```text
pesu-wifi/
├── android/                 # Native Android app (Kotlin, Material 3 Jetpack Compose)
│   ├── app/                 # Android application module & unit tests
│   ├── gradle/              # Gradle wrapper & version catalogs
│   └── build.gradle.kts     # Android root Gradle configuration
├── aur/                     # Arch User Repository (AUR) PKGBUILD package
├── completions/             # Shell completions (Bash, Zsh, Fish)
├── contrib/                 # Scoop package manifests & extra configs
├── src/                     # Native Rust CLI & background daemon source code
│   ├── daemon.rs            # Autonomous background keepalive watchdog
│   ├── portal.rs            # Cyberoam HTTP / XML authentication client
│   ├── wifi.rs              # OS Wi-Fi subsystems & /32 VPN host route bypass
│   ├── config.rs            # Encrypted credential storage (~/.config/pesu-wifi)
│   └── main.rs              # CLI command parser & entry point
├── website/                 # React + React Bits modern landing page
│   ├── src/                 # React components (SpotlightCard, BentoGrid, Simulator)
│   └── public/              # Static guides (instructions.html, ios.html)
├── .github/workflows/
│   ├── ci.yml               # Unified multi-platform CI & release workflow (Linux, Windows, Android)
│   └── pages.yml            # GitHub Pages deployment workflow
├── Cargo.toml               # Rust package definition
├── installer.iss            # Inno Setup 6 Windows setup wizard
├── install.sh               # One-line Linux installer
├── install.ps1              # One-line Windows installer
└── pesu-wifi.service        # systemd user service definition
```

---

## Building from Source

### Building Desktop CLI (Rust)
Requires Rust 1.75+:
```bash
cargo test
cargo build --release
```
The compiled binary will be placed at `target/release/pesu-wifi` (or `pesu-wifi.exe` on Windows).

### Building Android App (Gradle)
Requires JDK 17:
```bash
cd android
./gradlew test
./gradlew assembleRelease
```
The compiled APKs will be located in `android/app/build/outputs/apk/`.

### Building React Website
Requires Node.js 20+:
```bash
cd website
npm install
npm run build
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.
