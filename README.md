# PESU WiFi — Unified Multi-Platform Captive Portal Engine

A blazing fast, native multi-platform captive portal login manager and continuous keepalive watchdog daemon for PES University campus Wi-Fi networks (RR Campus, EC Campus & Amaatra Hostel).

[![Latest Release](https://img.shields.io/github/v/release/imyash0722/pesu-wifi?color=blue)](https://github.com/imyash0722/pesu-wifi/releases/latest)
[![Website](https://img.shields.io/badge/Website-GitHub_Pages-6C6E36.svg)](https://imyash0722.github.io/pesu-wifi/)
[![Setup Guide](https://img.shields.io/badge/Setup_Guide-Visual_Instructions-E3BD90.svg)](https://imyash0722.github.io/pesu-wifi/instructions.html)
[![iOS Shortcut](https://img.shields.io/badge/iOS_Shortcut-1--Tap_Generator-92e4c5.svg)](https://imyash0722.github.io/pesu-wifi/ios.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> **Note on Monorepo Consolidation:**  
> The Android application codebase (formerly hosted separately in `pesu-wifi-android`) has been fully merged into this canonical monorepo under the [`android/`](android/) directory. This single repository now hosts and maintains the Android application, desktop Rust daemon (Windows & Linux), iOS shortcut companion, and modern React landing page.

---

## Supported Platforms

| Platform | Interface | Engine | Status | Download |
| :--- | :--- | :--- | :--- | :--- |
| **Android** (8–16) | Material 3 Jetpack Compose | Foreground Service (`specialUse`) + Exact RTC Watchdog | Stable v3.7.3 | [Download APK](https://github.com/imyash0722/pesu-wifi/releases/latest) |
| **Windows** (10/11) | Win32 Tray / CLI | Native Rust + Win32 WlanAPI + Inno Setup | Stable v3.7.3 | [Download .exe](https://github.com/imyash0722/pesu-wifi/releases/latest) |
| **Linux** (x86_64) | CLI + systemd | Native Rust daemon + `/32` host route bypass | Stable v3.7.3 | [Download tar.gz](https://github.com/imyash0722/pesu-wifi/releases/latest) |
| **iOS / iPadOS** | Native Shortcuts | 1-Tap `.shortcut` Generator + Personal Automation | Live Tool | [Generate Shortcut](https://imyash0722.github.io/pesu-wifi/ios.html) |
| **macOS** | CLI / LaunchAgent | Native Rust CLI | Compatible | Build from Source |

---

## Key Features

- **Full VPN Compatibility (Zero User Fiddling):**
  - **Android:** Custom `ResilientSocketFactory` and `bindProcessToNetwork` catch kernel `EPERM` errors during socket `connect()` and transparently routes portal traffic via direct physical interface binding without requiring manual split-tunneling in Tailscale, WireGuard, Cloudflare WARP, OpenVPN, or AdGuard.
  - **Windows & Linux Desktop:** Automatically provisions `/32` host routes for campus gateway endpoints (`192.168.254.1/32`, `172.16.1.1/32`, `192.168.1.1/32`, `10.14.142.1/32`) bound strictly to the physical Wi-Fi interface. Prevents VPN default routes (`0.0.0.0/0`) from hijacking or dropping portal login and keepalive packets.
  - **iOS:** Complete companion guide explaining VPN connection order and Shortcuts automated sign-in.
- **In-App Self-Updater (Android):**
  - Automatic GitHub Releases API checks on launch and on-demand via the Guides screen.
  - Live version diff (`v3.7.2 → v3.7.3`), release notes rendering, and streaming APK download progress bar.
  - 1-tap package installation via secure `FileProvider` and Android `PackageInstaller` (`REQUEST_INSTALL_PACKAGES`).
- **In-App Knowledge Base & Guides (Android):**
  - Built-in Material 3 Guides screen accessible directly from the top app bar book icon.
  - Covers Quick Start permissions, AP roaming topologies (RR Campus, EC Campus, Amaatra), signal strength auto-connect, VPN compatibility, and multi-account failover.
- **1-Tap Direct iOS Shortcut Generator:**
  - Web generator at [`ios.html`](https://imyash0722.github.io/pesu-wifi/ios.html) outputs an Apple Shortcuts `.shortcut` file preloaded with your SRN credentials, opening directly into Safari without manual action assembly.
- **Smart Multi-Account Auto-Failover:**
  - Cascades seamlessly to backup credentials when an account hits concurrent device limits (error 529), daily quota limits, or session expiry.
- **Continuous Jittered Keepalive Watchdog:**
  - XML keepalive heartbeats sent every 120s–180s with anti-storm jitter (`mode=192`), satisfying Cyberoam's Dead Client Detection (DCD) without causing server-side load spikes.
- **Deep Sleep & Standby Immunity:**
  - **Android 15 & 16:** Exempt from 6-hour cumulative background execution limits via `FOREGROUND_SERVICE_SPECIAL_USE`.
  - **Windows & Linux:** Win32 Away Mode execution and systemd idle inhibitors keep the background watchdog alive while connected to campus SSIDs.
- **Android Quick Settings Tile:**
  - 1-tap connect / disconnect tile for your notification shade with live service status updates.

---

## Quick Installation

### 1. Android Phone (8.0 to 16)
- Download the standalone **[Stable APK](https://github.com/imyash0722/pesu-wifi/releases/latest)** (`pesu-wifi-v3.7.3-android-stable.apk`) or **[Tester APK](https://github.com/imyash0722/pesu-wifi/releases/latest)**.
- Follow the interactive **[Visual Setup Guide](https://imyash0722.github.io/pesu-wifi/instructions.html)**.
- Grant notification and battery optimization exemption permissions, add your credentials, and turn the service on.
- Future updates can be installed with a single tap directly within the app!

### 2. Windows PC (10 / 11)
Run in PowerShell (per-user install, no admin prompt needed):
```powershell
irm https://raw.githubusercontent.com/imyash0722/pesu-wifi/master/install.ps1 | iex
```
Or download the standalone Inno Setup **[`pesu-wifi-v3.7.3-windows-x86_64-setup.exe`](https://github.com/imyash0722/pesu-wifi/releases/latest)** from GitHub Releases.

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
Use our 1-tap **[iOS Shortcut Generator](https://imyash0722.github.io/pesu-wifi/ios.html)** to generate a `.shortcut` file preloaded with your SRN, then configure a single Personal Automation triggered whenever your device connects to `PESU-RR` or `PESU-EC`.

---

## CLI & Credential Management

The command-line interface and the Android app share the same credential structures and intuitive ergonomics:

```bash
# Add a new account (interactive password prompt without terminal echo)
pesu-wifi add
# Aliases: init, set

# Add account inline
pesu-wifi add <username> <password>

# List all configured accounts (-p to reveal saved passwords)
pesu-wifi list
# Alias: ls

# Switch the default active account
pesu-wifi select <username>
# Aliases: switch, use

# Delete a saved account
pesu-wifi del <username>
# Aliases: rm, remove, delete

# Run one-shot captive portal sign-in
pesu-wifi login

# Start persistent continuous background keepalive daemon
pesu-wifi daemon
```

---

## Repository Structure

```text
pesu-wifi/
├── android/                 # Native Android app (Kotlin, Material 3 Jetpack Compose)
│   ├── app/                 # Android application module & unit tests
│   │   └── src/main/java/com/imyash/pesuwifi/
│   │       ├── ui/          # Compose UI screens (HomeScreen, GuidesScreen, AccountsScreen, LogsScreen)
│   │       ├── service/     # Persistent Foreground Service & Quick Settings Tile
│   │       ├── util/        # ResilientSocketFactory, UpdateManager, NetworkProbe
│   │       └── viewmodel/   # PortalViewModel state manager
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
│   └── main.rs              # CLI command parser, subcommands & aliases
├── website/                 # React + Vite modern landing page
│   ├── src/                 # React components (CommandBlock, SpotlightCard, BentoGrid, Simulator)
│   └── public/              # Static companion tools (instructions.html, ios.html)
├── .github/workflows/
│   ├── ci.yml               # Multi-platform CI & release workflow (Linux, Windows, Android)
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
