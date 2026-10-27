# PESU WiFi — Changelog & Release Notes

A comprehensive, commit-by-commit record of all architecture changes, feature additions, bug fixes, and maintenance releases for the **PESU WiFi Login Manager & Watchdog Daemon**.

---

## Table of Contents
- [v3.7.4 — Roaming Portal Resolution, VPN Interface Isolation & Staged Auto-Reauth](#v374--roaming-portal-resolution-vpn-interface-isolation--staged-auto-reauth)
  - [Overview & Major Highlights](#v374-overview--major-highlights)
- [v3.7.3 — Resilient Fallback DNS for Updater, 3-Tier VPN Socket Escalation & Permanent Keystore](#v373--resilient-fallback-dns-for-updater-3-tier-vpn-socket-escalation--permanent-keystore)
  - [Overview & Major Highlights](#v373-overview--major-highlights)
- [v3.7.2 — Dual-Gateway Discovery, Kernel VPN Bypass Binding & Flapping Fix](#v372--dual-gateway-discovery-kernel-vpn-bypass-binding--flapping-fix)
  - [Overview & Major Highlights](#v372-overview--major-highlights)
- [v3.7.1 — In-App Guides, In-App Self-Updater, Syntax-Colored Command Blocks & CLI Aliases](#v371--in-app-guides-in-app-self-updater-syntax-colored-command-blocks--cli-aliases)
  - [Overview & Major Highlights](#v371-overview--major-highlights)
- [v3.7.0 — Unified Multi-Platform Monorepo (Android, Windows, Linux, iOS & React Landing Page)](#v370--unified-multi-platform-monorepo-android-windows-linux-ios--react-landing-page)
  - [Overview & Major Highlights](#v370-overview--major-highlights)
- [v3.6.0 — Dual-Layer VPN Compatibility, Multi-Account Auto-Failover & Self-Signed Portal SSL](#v360--dual-layer-vpn-compatibility-multi-account-auto-failover--self-signed-portal-ssl)
  - [Overview & Major Highlights](#v360-overview--major-highlights)
- [v3.5.1 — Fix Keepalive Heartbeat Watchdog in Event-Driven Engine](#v351--fix-keepalive-heartbeat-watchdog-in-event-driven-engine)
  - [Overview & Major Highlights](#v351-overview--major-highlights)
- [v3.5.0 — Always-On Pure Event-Driven OS Rules (Zero Keepalive Timers & Kernel Sockets)](#v350--always-on-pure-event-driven-os-rules-zero-keepalive-timers--kernel-sockets)
  - [Overview & Major Highlights](#v350-overview--major-highlights)
- [v3.4.0 — Continuous 24/7 Daemon, Sleep & Idle Immunity, Wi-Fi Sleep Prevention & Win32 Away Mode](#v340--continuous-247-daemon-sleep--idle-immunity-wi-fi-sleep-prevention--win32-away-mode)
  - [Overview & Major Highlights](#v340-overview--major-highlights)
- [v3.3.0 — Windows 10/11 Support, Win32 Native Wifi, Toast Notifications & Scoop](#v330--windows-1011-support-win32-native-wifi-toast-notifications--scoop)
  - [Overview & Major Highlights](#v330-overview--major-highlights)
- [v3.2.0 — Server-Load Minimization, Keepalive Tuning & Resilient Architecture](#v320--server-load-minimization-keepalive-tuning--resilient-architecture)
  - [Overview & Major Highlights](#v320-overview--major-highlights)
- [v3.1.0 — Clap Derive Migration & Production CLI Architecture](#v310--clap-derive-migration--production-cli-architecture)
  - [Overview & Major Highlights](#v310-overview--major-highlights)

---

## v3.7.4 — Roaming Portal Resolution, VPN Interface Isolation & Staged Auto-Reauth

**Release Date:** October 28, 2026  
**Git Tag:** [`v3.7.4`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.7.4)  
**Release Type:** Critical Roaming & VPN Network Isolation Fix (Android)

### v3.7.4 Overview & Major Highlights

#### 1. Strict Physical Wi-Fi Interface & IP Isolation (Fixes VPN Interface Capture)
- **Root Cause:** When Tailscale or ProtonVPN was active over Wi-Fi, Android marked the VPN network with both `TRANSPORT_VPN` and `TRANSPORT_WIFI`. `getWifiNetwork()` selected the VPN network, extracting Tailscale's CGNAT address (`100.x.x.x`) and calling `bindProcessToNetwork` on `tun0`. All subsequent portal requests were trapped inside the VPN tunnel and failed with `ENETUNREACH`.
- **Solution:** Hardened `getWifiNetwork()` to strictly require `!hasTransport(TRANSPORT_VPN)` and reject non-`wlan` interfaces. Hardened `getWifiLocalAddress()` to reject `100.64.0.0/10` CGNAT VPN addresses and `tun`/`wg` interfaces, ensuring Tier 2 fallback sockets only bind to genuine Wi-Fi IP addresses (`10.x.x.x`, `172.16.x.x`).

#### 2. Campus vs. Amaatra Portal Resolution Priority (Fixes Foodcourt 4s Timeout)
- **Root Cause:** In the Foodcourt AP subnet (`172.16.88.x`, gateway `172.16.88.1`), `resolvePortalBase()` matched `gw.startsWith("172.16.")` and incorrectly targeted `http://172.16.88.1:8090/login.xml`. Because `172.16.88.1` is a Cisco router gateway and not a Cyberoam server, all login requests hung for 4000ms until timeout.
- **Solution:** Reordered `resolvePortalBase()` to prioritize SSID matching (`Foodcourt`, `PESU-EC-Campus`, `pes south cafe`) and campus DNS (`192.168.3.2`). Only route to Amaatra portal bases when SSID is `AMAATRA-HOSTEL` or gateway is explicitly `172.16.1.1` / `172.16.0.1`. Foodcourt subnets now correctly target the central campus Cyberoam portal (`http://192.168.254.1:8090`).

#### 3. Eliminated False `loggedIn` Flag During Active VPN Sessions
- **Root Cause:** When Cyberoam returned `ack='nack'` (device unauthenticated), `refreshStatus()` checked `NetworkCapabilities.NET_CAPABILITY_VALIDATED`. If a VPN was active, Android reported the network as validated, causing `refreshStatus()` to falsely set `loggedIn=true` and suppress auto-reauthentication.
- **Solution:** If `vpnActive` is true, OS validation flags are strictly ignored when Cyberoam reports `nack`. The session remains `loggedIn=false`, immediately triggering auto-authentication.

#### 4. Disconnected Wi-Fi State Protection & Silent Coroutine Cancellations
- **Root Cause:** Network change callbacks received when Wi-Fi was disconnected triggered `login()` over cellular or VPN, causing unnecessary 4-second socket timeouts. Also, normal coroutine cancellations during AP roaming dumped `JobCancellationException` error logs.
- **Solution:** `login()` and `logout()` immediately return early if `wifiNet == null`. Handled `CancellationException` in keepalive check loop to suppress false error noise.

---

## v3.7.3 — Resilient Fallback DNS for Updater, 3-Tier VPN Socket Escalation & Permanent Keystore

**Release Date:** October 27, 2026  
**Git Tag:** [`v3.7.3`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.7.3)  
**Release Type:** Critical Stability & Network Resilience Release (Android & Monorepo)

### v3.7.3 Overview & Major Highlights

#### 1. Resilient Fallback Anycast DNS for In-App Updater (Fixes "No Address Associated With Hostname")
- **Root Cause:** Campus Wi-Fi DNS resolvers (`192.168.3.2`) frequently drop or throttle UDP port 53 packets directed to public domains, causing OkHttp to throw `UnknownHostException: Unable to resolve host "api.github.com": No address associated with hostname`.
- **Solution:** Integrated an intelligent `resilientDns` resolver within `UpdateManager.kt`. If the system DNS query times out or fails, requests to GitHub domains (`api.github.com`, `github.com`, `objects.githubusercontent.com`, `raw.githubusercontent.com`) automatically resolve to verified Fastly/GitHub anycast IP pools (`20.207.73.85`, `140.82.112.6`, `185.199.108.133`). Full TLS SNI and certificate verification are preserved.

#### 2. Dynamic Process Unbinding During Update Checks & Downloads
- **Root Cause:** `ConnectivityManager.bindProcessToNetwork` bound the entire application process to `wlan0`. If Wi-Fi was in a captive or degraded state, cellular data failover was completely blocked.
- **Solution:** Encapsulated update checks and APK downloads in `withProcessUnbound`, temporarily freeing the process from explicit physical binding so Android can leverage mobile data or uninhibited default routing.

#### 3. 3-Tier VPN Socket Escalation (Fixes "EPERM / Operation Not Permitted on Reboot")
- **Root Cause:** On device reboot, VPN applications (Tailscale, WireGuard, Cloudflare WARP) initialize in blocking/lockdown mode. In previous builds, if a bound socket threw `EPERM`, the fallback attempt also bound locally and threw `EPERM` unhandled.
- **Solution:** Implemented a non-blocking 3-tier escalation engine in `ResilientSocketFactory`:
  - **Tier 1:** Primary Wi-Fi network socket (`Network.getSocketFactory()`).
  - **Tier 2:** Local IP-bound socket (`localBindAddress`).
  - **Tier 3:** Completely unbound standard system socket (`fallbackFactory.createSocket()`), which completely bypasses the Linux kernel's local interface bind restrictions.
- Added automatic socket option transference (`tcpNoDelay`, `soTimeout`, `keepAlive`, `reuseAddress`) across fallback tiers.

#### 4. Permanent Release Signing Keystore (Fixes "Package Conflicts with Existing Package")
- **Root Cause:** Previous debug and CI builds generated transient signing keys, triggering `INSTALL_FAILED_UPDATE_INCOMPATIBLE` ("App not installed as package conflicts with an existing package") when upgrading.
- **Solution:** Committed a permanent release keystore (`android/app/release.keystore`) and configured both `release` and `debug` build types in Gradle to use it. All subsequent updates install smoothly with one tap.

---

## v3.7.2 — Dual-Gateway Discovery, Kernel VPN Bypass Binding & Flapping Fix

**Release Date:** October 27, 2026  
**Git Tag:** [`v3.7.2`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.7.2)  
**Release Type:** Critical Bugfix & Network Resilience Release (Android & Linux/Windows CLI)  

### v3.7.2 Overview & Major Highlights

- **⚡ Fast Concurrent Gateway Probing (192.x.x.x vs 172.x.x.x):**
  - Android client now concurrently probes all campus gateway candidates (`http://192.168.254.1:8090`, `http://172.16.1.1:8090`, `http://192.168.1.1:8090`, and `http://$gw:8090`) in parallel using Kotlin coroutines (`async`).
  - Automatically identifies whether the device is connected to Main/EC Campus (`192.168.254.1`) or Amaatra Hostel (`172.16.1.1`) within 1.5 seconds without hanging or timeouts.
- **🛡️ Native Kernel Process Binding (`bindProcessToNetwork`):**
  - Android app now calls `ConnectivityManager.bindProcessToNetwork(wifiNet)`, forcing all DNS lookups, OkHttp sockets, and HTTP requests across the entire application process directly through `wlan0`.
  - Fixes the issue where active VPN tunnels (Tailscale, WireGuard, WARP) or Cellular fallback routed requests from foreign IPs (e.g., `100.x.x.x`), preventing timeouts against private RFC-1918 gateway endpoints.
- **🔄 Elimination of Wi-Fi Connection Flapping & BSSID Pinning:**
  - Removed BSSID locking from `WifiSuggestionManager` and removed aggressive AP re-scanning in `PortalViewModel.connect()`.
  - If already associated with a campus Wi-Fi network, the app immediately authenticates without tearing down the active Wi-Fi link.
  - Allows the phone's native 802.11k/v/r roaming controller to transition smoothly across access points.
- **🛑 Fixed Background Job Loops & Coroutine Cancellations:**
  - `WifiWakeupJobService` now yields immediately if `WifiKeepaliveService` is running in the foreground, eliminating duplicate concurrent logins and coroutine cancellations (`JobCancellationException`).
  - Added a roaming debounce delay before keepalive teardown to survive momentary AP handoffs.
- **🖥️ Desktop CLI Portal Discovery & Non-Destructive Self-Healing:**
  - Fixed `is_portal_online()` in Rust CLI to dynamically rediscover alternative gateway bases (`192.168.254.1`, `172.16.1.1`, `192.168.1.1`, gateway IP) when the cached base fails.
  - Replaced destructive Wi-Fi radio cycling (`nmcli radio wifi off/on`) with safe re-association (`nmcli connection up`).
  - Added `/32` host routes for `172.16.1.1/32` and `192.168.1.1/32` on Windows and Linux to prevent VPN default routes from capturing hostel portal traffic.

---

## v3.7.1 — In-App Guides, In-App Self-Updater, Syntax-Colored Command Blocks & CLI Aliases

**Release Date:** October 26, 2026  
**Git Tag:** [`v3.7.1`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.7.1)  
**Release Type:** Feature & Ergonomics Release (All Operating Systems)  

### v3.7.1 Overview & Major Highlights

- **📚 In-App Knowledge Base & Guides**: Added a comprehensive Material 3 `GuidesScreen` accessible directly from the top app bar in Android. Covers:
  - Quick Start & Background Battery Whitelist (`specialUse` foreground service, QS Tile).
  - Campus Gateways & Roaming Subnets (`192.168.254.1:8090` vs `172.16.1.1:8090`).
  - Zero-Fiddling Dual-Layer VPN Bypass (`ResilientSocketFactory`).
  - Smart Multi-Account Auto-Failover.
  - iOS Companion Shortcut integration.
- **🔄 Seamless In-App Self-Updater**: Added `UpdateManager` and Material 3 `UpdateDialog` to the Android app:
  - Periodically checks the GitHub Releases API and compares semantic versions.
  - Displays formatted release notes and version diff (`v3.7.0` → `v3.7.1`).
  - Streams APK downloads with live byte progress tracking.
  - Hands off directly to Android's native `PackageInstaller` via `FileProvider` with `REQUEST_INSTALL_PACKAGES` permission handling.
- **💻 Syntax-Colored Terminal Blocks & Copy Button**: Created an interactive `CommandBlock` React component for the landing page with macOS window controls, shell badges, token color coding, and 1-tap clipboard copy.
- **📲 1-Tap iOS Shortcut Generator**: Enhanced `ios.html` with a client-side XML plist `.shortcut` file generator, allowing iOS users to download and immediately open their configured automation without manual action setup.
- **⚡ CLI Command Aliases**:
  - `pesu-wifi init`, `pesu-wifi set` as aliases for `pesu-wifi add`.
  - `pesu-wifi rm`, `pesu-wifi remove`, `pesu-wifi delete` as aliases for `pesu-wifi del`.
  - `pesu-wifi ls` as alias for `pesu-wifi list`.
  - `pesu-wifi switch` as alias for `pesu-wifi select`.
- **🗑️ Standalone Repository Deletion**: Safely archived (`.bundle`) and deleted the legacy `imyash0722/pesu-wifi-android` repository on GitHub following full monorepo consolidation.

---

## v3.7.0 — Unified Multi-Platform Monorepo (Android, Windows, Linux, iOS & React Landing Page)

**Release Date:** October 26, 2026  
**Git Tag:** [`v3.7.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.7.0)  
**Release Type:** Unified Multi-Platform Release (All Operating Systems)  

### v3.7.0 Overview & Major Highlights

- **📦 Monorepo Unification**: Fully consolidated `pesu-wifi-android` and `pesu-wifi` into a single, canonical repository under `pesu-wifi`.
- **📱 Android App Integration**:
  - Full Material 3 native Jetpack Compose app integrated into `android/`.
  - Android 14–16 foreground service hardened under `specialUse` (`FOREGROUND_SERVICE_TYPE_SPECIAL_USE`), exempting the continuous keepalive monitor from cumulative 6-hour kill switches.
  - Linux kernel `EPERM` interception in `ResilientSocketFactory` with RFC-1918 gateway bypass for active VPN tunnels (Tailscale, WireGuard, WARP, OpenVPN).
  - Multi-account auto-failover, 120s ±30s jittered keepalive daemon, and Quick Settings tile.
- **🖥️ Desktop & Daemon (Windows & Linux)**:
  - Linux x86_64 standalone binary and systemd user service with `/32` host route bypass for VPNs.
  - Windows 10/11 Win32 WlanAPI native service and Inno Setup 6 `.exe` setup wizard.
- **🌐 React & React Bits Website**:
  - Modern React 19 landing page deployed to `https://imyash0722.github.io/pesu-wifi/`.
  - Features React Bits Spotlight cards, Aurora background, Bento grid, and interactive real-time keepalive & roaming simulator.
- **🍎 iOS Personal Automation Guide**: Complete zero-click Apple Shortcuts guide hosted directly on the site.

---

## v3.6.0 — Dual-Layer VPN Compatibility, Multi-Account Auto-Failover & Self-Signed Portal SSL

**Release Date:** October 26, 2026  
**Git Tag:** [`v3.6.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.6.0)

### v3.6.0 Overview & Major Highlights
- **Dual-Layer VPN Compatibility (WireGuard, OpenVPN, Cloudflare WARP, Tailscale, Cisco AnyConnect):**
  - **Layer 1: Host `/32` Route Bypass:** Automatically configures `/32` host routes for `192.168.254.1/32` and `10.14.142.1/32` via the physical Wi-Fi gateway (`nmcli connection modify <conn> +ipv4.routes` on Linux, `New-NetRoute` / `route add` on Windows). Guarantees that captive portal requests and student browser tabs do not get sucked into VPN tunnels (`0.0.0.0/0` or `0.0.0.0/1`).
  - **Layer 2: Socket-Level Interface Pinning (`SO_BINDTOIFINDEX`):** Raw sockets bind directly to the physical Wi-Fi network interface index (`libc::SO_BINDTOIFINDEX = 62`). Runs completely unprivileged on modern Linux kernels (5.7+), routing probe, keepalive `/live`, login, and logout requests directly to the campus gateway regardless of VPN routing tables, default routes, or split-tunnel policies.
- **Multi-Account Auto-Failover:**
  - Automatically cascades to secondary accounts when the active account reaches concurrent device session limits (code 529: "Maximum login limit reached") or quota transfer limits ("data transfer limit has been exceeded").
- **Dynamic Keepalive Detection:**
  - Automatically parses `liveReqTimeInJS` from `/httpclient.html` on campus gateways, dynamically adjusting keepalive heartbeat intervals to match server settings.
- **Insecure / Self-Signed SSL Support:**
  - Handles self-signed certificates gracefully using a custom `InsecureServerCertVerifier` in Rustls without compromising security outside the captive portal.
- **Extended Campus SSID Registry:**
  - Added support for `PESU-CIE`, `AMAATRA_HOSTEL`, `Foodcourt`, `pes south cafe`, and related campus subnet variants.

---

## v3.5.1 — Fix Keepalive Heartbeat Watchdog in Event-Driven Engine
 
**Release Date:** September 19, 2026  
**Git Tag:** [`v3.5.1`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.5.1)  

### v3.5.1 Overview & Major Highlights
- **Fixed Disconnections Caused by Missing Keepalive Heartbeats:** Solved the regression where the captive portal gateway dropped active sessions due to portal server inactivity timeout.
- **Hybrid Event-Driven Keepalive Architecture:** Netlink route sockets (`libc::poll`) and Windows Wlan callbacks (`rx.recv_timeout`) now wait with a timeout equal to the configured keepalive interval (default 180s with anti-storm jitter).
  - **Instant 0ms Wakeup:** If any OS network event occurs (link bounce, carrier switch, IP change, roaming), the kernel system call returns immediately (0ms).
  - **Periodic Keepalive Guarantee:** If no network events occur, the timeout fires at 180s, sending the `/live` heartbeat to reset the gateway's inactivity timer and re-authenticating if expired.
- **Unit Test Coverage:** Added unit tests verifying `OsNetworkListener::wait_event` timeout behavior.

---

## v3.5.0 — Always-On Pure Event-Driven OS Rules (Zero Keepalive Timers & Kernel Sockets)

**Release Date:** September 19, 2026  
**Git Tag:** [`v3.5.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.5.0)  

### v3.5.0 Overview & Major Highlights
- **100% Pure Event-Driven OS Rules Engine:** Completely eliminated the legacy 120s/180s keepalive timer loop and periodic 5-second standby polling. The daemon blocks in OS kernel system calls, waking with **0ms latency** and consuming **0.00% CPU** until an actual network state transition occurs.
- **Linux Kernel Netlink System Calls (`rtnetlink`):** Direct socket binding to kernel multicast groups `RTMGRP_LINK | RTMGRP_IPV4_IFADDR | RTMGRP_NOTIFY` via `libc::socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE)`, receiving link UP/DOWN, carrier transitions, and DHCP lease assignments with zero polling.
- **Windows Native Wifi Event Callbacks (`WlanRegisterNotification`):** Direct integration with `WlanAPI.dll` registering for all ACM and MSM notifications (`wlan_notification_acm_connection_complete`, `wlan_notification_msm_roaming_end`, `wlan_notification_acm_disconnected`), piped over a thread-safe non-blocking `SyncSender` channel.
- **AP Roaming & BSSID Handover Verification:** Continuously queries and compares hardware BSSID (`nmcli` active BSSID on Linux, `WlanQueryInterface` / `dot11Bssid` on Windows). On physical AP handover across campus blocks/classrooms, proactively verifies gateway reachability and re-authenticates if needed.
- **Enhanced Standalone Release Installers:** `install.sh` and `install.ps1` automatically detect local precompiled binaries inside downloaded release archives, installing in <100ms without requiring Rust, Cargo, or network downloads.

---

## v3.4.0 — Continuous 24/7 Daemon, Sleep & Idle Immunity, Wi-Fi Sleep Prevention & Win32 Away Mode

**Release Date:** September 19, 2026  
**Git Tag:** [`v3.4.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.4.0)  

### v3.4.0 Overview & Major Highlights
- **Continuous 24/7 Background Daemon:** Operates completely independently of any external Wi-Fi manager (purged `contrib/99-pesu-wifi.sh` and eliminated all dispatcher scripts, hooks, and root requirements). Runs persistently across your session via user systemd (`pesu-wifi.service`) on Linux and Task Scheduler / pure Win32 `CREATE_NO_WINDOW` on Windows.
- **Power-Saving & Sleep Immunity:**
  - Automatically disables 802.11 Wi-Fi power saving on campus connections (`nmcli` powersave 2 on Linux, `powercfg` Wireless Maximum Performance on Windows) to prevent the adapter from entering low-power sleep or dropping packets.
  - Acquires active OS sleep/idle inhibitor locks (`systemd-inhibit` on Linux, Win32 `SetThreadExecutionState` with Away Mode `ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED` on Windows) while on campus Wi-Fi. Keeps CPU, network, and background tasks active even when the display sleeps.
  - Releases power inhibitor locks gracefully upon disconnecting or leaving campus.
- **Instant Auto-Login on Campus Connection:** Wakes up within 5s of connecting to campus Wi-Fi (`PESU-EC-Campus`) and authenticates immediately, while resting in ultra-low overhead standby (0% CPU, zero network requests) when disconnected or on non-campus networks.
- **Systemd Service Hardening:** Configured `Nice=-5`, `MemoryLow=32M`, and `Slice=session.slice` in `pesu-wifi.service` to prevent memory eviction under system pressure.

---

## v3.3.0 — Windows 10/11 Support, Win32 Native Wifi, Toast Notifications & Scoop

**Release Date:** September 18, 2026  
**Git Tag:** [`v3.3.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.3.0)  

### v3.3.0 Overview & Major Highlights
- **100% Feature Parity on Windows:** Windows users have the exact same capabilities as Linux: auto-login, continuous watchdog daemon, multi-account switching, Wi-Fi self-healing, and real-time status reporting.
- **Native Win32 Wi-Fi Subsystem (`WlanAPI.dll`):** Direct integration with Windows Native Wifi API via `windows-sys` (`WlanOpenHandle`, `WlanEnumInterfaces`, `WlanQueryInterface`) for zero-overhead SSID detection, with robust fallback to `netsh`.
- **Native Windows 10/11 Desktop Toast Notifications:** Integrated interactive Windows Action Center toast notifications using WinRT XML templates for successful logins, keepalive session renewals, and auth alerts.
- **Cross-Platform Config & Secrets:** Transparent storage resolution adhering to Windows standards (`%APPDATA%\pesu-wifi\config.json`) and Unix standards (`~/.config/pesu-wifi/config.json`).
- **Windows UTF-8 Console Rendering:** Automatic code page initialization (`SetConsoleOutputCP(65001)`) ensuring ANSI box-drawing frames and status indicators render cleanly in Windows Command Prompt and PowerShell.
- **Windows Packaging & Installers:**
  - One-click PowerShell installation script: `install.ps1`.
  - Official Scoop bucket package manifest: `contrib/pesu-wifi.json`.
  - Standalone release zip bundle: `pesu-wifi-v3.3.0-windows-x86_64.zip`.

---

## v3.2.0 — Server-Load Minimization, Keepalive Tuning & Resilient Architecture

**Release Date:** September 18, 2026  
**Git Tag:** [`v3.2.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.2.0)  

### v3.2.0 Overview & Major Highlights
- **180s Keepalive Interval:** Increased default keepalive watchdog polling interval from 60s to 180s, matching captive portal session lifetimes and slashing background requests by 66.7%.
- **Elimination of Redundant Gateway Page Downloads:** Inverted the daemon loop to check session heartbeat (`/live`) directly. Skips downloading the heavy full HTML page (`/httpclient.html`) on healthy cycles, reducing network traffic by >98%.
- **HTTP Keep-Alive Connection Reuse:** Removed forced `Connection: close` across all portal endpoints, allowing TCP socket reuse and eliminating connection tracking (`ip_conntrack`) table exhaustion on campus gateways.
- **Anti-Storm Desynchronization Jitter:** Added dynamic ±5s jitter to keepalive intervals to eliminate simultaneous "thundering herd" gateway query bursts during campus-wide network reconnects.
- **Multi-Campus Dynamic Gateway Discovery:** Support `portal_url`, `keep_alive_interval`, and `notifications` in `~/.config/pesu-wifi/config.json`, making custom URLs persistent for background systemd services.
- **Secure Runtime Lockfile & PID Lookup:** Replaced `/tmp/pesu_wifi_daemon.lock` with user-isolated `$XDG_RUNTIME_DIR` / config directory locks, with automatic PID recording to eliminate process tree scans.
- **NetworkManager Dispatcher Hook:** Added `contrib/99-pesu-wifi.sh` for sub-50ms instant authentication when waking laptops or reconnecting to campus Wi-Fi.

---

## v3.1.0 — Clap Derive Migration & Production CLI Architecture

**Release Date:** September 12, 2026  
**Git Tag:** [`v3.1.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.1.0)  

### v3.1.0 Overview & Major Highlights
- **Full Migration to `clap` Derive:** Completely replaced handwritten CLI argument parsing with the industry-standard `clap` crate (`#[derive(Parser, Subcommand)]`).
- **Automatic Multi-Level Help Generation:** Every subcommand (`pesu-wifi start --help`, `pesu-wifi list --help`, etc.) now generates structured, dedicated help screens automatically.
- **Robust Argument Parsing & Typo Suggestions:** Non-standard arguments and subcommands are strictly rejected with standard error context and `did you mean?` suggestions.

---

## v3.0.4 — Strict Standard POSIX Flags & Clean Syntax Enforcement

**Release Date:** September 12, 2026  
**Git Tag:** [`v3.0.4`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.4)  

### v3.0.4 Overview & Major Highlights
- **Strict POSIX/GNU Options Enforcement:** Enforced standard command-line syntax:
  - Version: strictly `-v` or `--version` (pruned informal `version` and `-version`).
  - Help: strictly `-h` or `--help` (pruned informal `help` and `-help`).
  - Passwords: `-p` or `--passwords`.
- **Consistent Error Reporting:** Unrecognized or non-standard invocations like `pesu-wifi version` or `pesu-wifi help` output standard error hints (`Try 'pesu-wifi --help' for more information.`) and exit code `2`.

---

## v3.0.3 — Top-Level Password Flags & Enhanced CLI Flag Support

**Release Date:** September 12, 2026  
**Git Tag:** [`v3.0.3`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.3)  

### v3.0.3 Overview & Major Highlights
- **Top-Level `-p` / `--passwords` Support:** Directly running `pesu-wifi -p` or `pesu-wifi --passwords` now immediately lists saved accounts along with their credentials, matching user expectations for top-level options.
- **Top-Level Shell Completions:** Updated Bash, Fish, and Zsh completion definitions so `-p` and `--passwords` auto-complete seamlessly at the root command level.
- **Lenient Flag Handling:** Added support for single-dash `-version`, `-help`, and `-passwords` to ensure smooth compatibility regardless of terminal flag habit.

---

## v3.0.2 — Standardized CLI Options, Unified Daemon Controls & Clean Output

**Release Date:** September 12, 2026  
**Git Tag:** [`v3.0.2`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.2)  

### v3.0.2 Overview & Major Highlights
- **Standardized CLI Flags & Error Handling:** Implemented standard POSIX-style `-v, --version`, `-h, --help`, and `-p, --passwords` options across commands. Running `pesu-wifi` without arguments or with unrecognized commands now follows standard Unix conventions, outputting a concise error message with an exit code of `2` and prompting to run `--help`, rather than printing full help text.
- **Unified Daemon Controls (`start [-f, --foreground]`):** Resolved command redundancy between `start` and `daemon`. Running `pesu-wifi start` now natively manages the background `systemd --user` watchdog service, while `pesu-wifi start -f` (or `--foreground`) runs the watchdog loop directly in the active terminal session. Deprecated and removed the redundant `restart` command in favor of standard service restarts.
- **Removed Redundant `wifi` Command:** Pruned the unused `wifi` interactive command, retaining core captive portal login/keepalive logic and automated NetworkManager self-healing.
- **Streamlined Help & Perfect Column Alignment:** Removed the verbose `EXAMPLES` section from help output and refined visual string length calculations to ensure ANSI-colored command names and descriptions align in columns.
- **Updated Shell Completions:** Updated Bash, Fish, and Zsh completion scripts to support all new flags (`-f`, `--foreground`, `-p`, `--passwords`, `-h`, `--help`, `-v`, `--version`) and removed completed entries for pruned subcommands.

---

## v3.0.1 — Build Stability, Non-POSIX Filesystem Support & Zero-Python Completions

**Release Date:** September 12, 2026  
**Git Tag:** [`v3.0.1`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.1)  

### v3.0.1 Overview & Major Highlights
- **Resilient Non-POSIX Filesystem Builds:** Automated filesystem type inspection in `install.sh` detects external and non-POSIX partitions (`exfat`, `vfat`, `ntfs`, `cifs`) and automatically redirects `CARGO_TARGET_DIR` to native storage (`~/.cache/cargo-target/pesu-wifi`), preventing `rustc` metadata corruption (`E0786`) and compiler panics.
- **Native Zero-Python Shell Completions:** Added internal `__list-accounts` fast-path helper to `pesu-wifi`, allowing Bash, Zsh, and Fish completion scripts to query saved accounts in <2ms with zero Python dependencies.
- **Installer Safety & Daemon Path Hardening:** `install.sh` now cleanly unlinks existing destination binaries and dangling symlinks before copying, preventing `ETXTBSY` ("Text file busy") errors when updating active daemons, and guarantees the user systemd service points to the user-installed binary.
- **Arch Linux Packaging Compliance:** Added `options=(!lto)` to `PKGBUILD` to prevent toolchain linker incompatibilities during makepkg package creation.
- **Expanded Offline Test Suite:** Added unit tests verifying POSIX `0600` file permission masks, `0700` configuration directory creation, ANSI visual string length calculation for Unicode box-drawing, and campus SSID validation.
- **Legacy Cleanup:** Purged all legacy Python systemd user units, wake triggers, and sleep hooks across the local system.

---

## v3.0.0 — Native Rust Rewrite, Blazing Speed & Zero Runtime Dependencies

**Release Date:** September 10, 2026  
**Git Tag:** [`v3.0.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.0)  

### v3.0.0 Overview & Major Highlights
- **100% Native Rust Implementation:** Rewrote the entire CLI and background watchdog daemon in safe, performant Rust, completely eliminating Python runtime and external dependency requirements (`requests`, `urllib3`, etc.).
- **Sub-Millisecond Cold-Start Execution:** CLI command invocation time dropped from ~180ms in Python to <1.5ms in native Rust (120x speedup), making terminal auto-completions and status queries instantaneous.
- **Ultra-Lean Resident Daemon Memory:** Watchdog background memory consumption reduced from ~35 MB RSS to ~3.2 MB RSS (91% memory reduction), ideal for low-spec laptops and embedded devices.
- **Robust Fast-Path Captive Portal Parsing:** Custom streaming XML parser handling Cyberoam CDATA enclosures, HTML entity decodes (`&lt;`, `&gt;`, `&amp;`, `&#39;`), anti-storm 500ms network jitter backoff, and strict `status == "LIVE"` session validation.
- **Single Self-Contained Binary:** Builds a standalone stripped binary (~2.3 MB) with LTO enabled, zero external shared library requirements (pure `rustls-webpki`), and full compatibility across Linux distributions.
- **Atomic 0600 Credential Management:** POSIX file-mode enforcement guarantees credentials in `config.json` and `.env` are protected upon initial file creation without umask races.
- **Modernized CI/CD:** GitHub Actions workflow upgraded to test, build, and verify Rust binaries on `ubuntu-latest`.

### v3.0.0 Performance Benchmarks

| Metric | Python v2.3.0 | Rust v3.0.0 | Improvement |
| :--- | :--- | :--- | :--- |
| **Cold Start (`pesu-wifi status`)** | ~180 ms | **1.2 ms** | **150x faster** |
| **Daemon RSS Memory** | ~35 MB | **3.2 MB** | **91% lower RAM** |
| **Binary Size** | ~38 MB (w/ runtime) | **2.3 MB (standalone)** | **Zero runtime deps** |
| **Portal Response Parsing** | ~8 ms | **<0.1 ms** | **80x faster** |

### v3.0.0 Architecture & Modular Breakdown

- **`src/main.rs`:** CLI entry point, zero-dependency sub-millisecond argument parsing, command routing (`status`, `login`, `logout`, `select`/`use`, `wifi`, `add`, `del`, `list`, `daemon`, `start`, `stop`, `restart`, `version`, `help`), and ANSI card visualization.
- **`src/portal.rs`:** Cyberoam captive portal HTTP client powered by `ureq` with `rustls-webpki-roots`. Handles login (`mode=191`), keepalive (`mode=192`), and logout (`mode=193`), with full CDATA and HTML entity parsing.
- **`src/config.rs`:** Multi-account JSON/ENV configuration store with atomic file permission masking (`0600`), active account selection, and directory traversal fallbacks.
- **`src/wifi.rs`:** `nmcli` Wi-Fi management interface, automatic campus SSID verification (`PESU-EC-Campus`, `PES-RR-Campus`, `PESU-Guest`), access point scanning, and multi-tier network self-healing.
- **`src/daemon.rs`:** Resilient 60s keepalive watchdog daemon, file-based singleton lock (`fs2`), desktop notifications via `notify-send`, and systemd user unit orchestration.
- **`src/ui.rs`:** ANSI terminal coloring, status card formatting, and visual length calculations for Unicode box-drawing characters.

---

## v2.3.0 — Resilient Watchdog, Process Controls & Distribution Hardening

**Release Date:** September 9, 2026  
**Git Tag:** [`v2.3.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.3.0)  
**Commits in this Release:** 18 commits (`6e5b275` -> `c322206`)

### v2.3.0 Overview & Major Highlights
- **Native Process Controls (`start`, `stop`, `restart`):** First-class CLI commands to manage the background `systemd --user` watchdog daemon and cleanly clean up orphan processes.
- **Smart Campus SSID Auto-Standby:** Automatically detects known campus SSIDs (`PESU-EC-Campus`, `PES-RR-Campus`, `PESU-Guest`, etc.) and pauses watchdog activity on home/cellular networks.
- **Desktop Notifications via `notify-send`:** Real-time desktop alerts on login success, background keepalive session renewal, and authentication failures.
- **Atomic Security Hardening:** Prevents umask race conditions by atomically setting `0600` permissions on credential files at creation time using `os.open()`.
- **Anti-Storm Wi-Fi Jitter Protection:** Added a 500ms retry for `check_live()` and a 2-consecutive failure threshold in daemon keepalive to prevent duplicate RST packets and session thrashing.
- **Multi-Packaging & Distribution Support:** Standard `pyproject.toml` (PEP 517/621) for `pipx`/`pip`, tab-completion scripts for Bash, Zsh, and Fish, and multi-version GitHub Actions CI matrix.

### v2.3.0 Commit-by-Commit Technical Breakdown

#### [`c322206`](https://github.com/imyash0722/pesu-wifi/commit/c322206) — `fix(ci): enable future annotations for Python 3.9 compatibility and add Python 3.13 to matrix`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `.github/workflows/ci.yml`, `pesu_wifi.py`, `tests/test_unit.py`
- **Technical Detail:** Added `from __future__ import annotations` to allow PEP 585/604 type union syntax (`str | None`, `tuple[bool, str]`) on Python 3.9 environments without runtime syntax errors. Added Python 3.13 to the GitHub Actions test matrix.

#### [`69c0b39`](https://github.com/imyash0722/pesu-wifi/commit/69c0b39) — `docs: add CI badge, version command, and pipx uninstallation guide`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `README.md`
- **Technical Detail:** Embedded GitHub Actions CI build status badge, added documentation for the `pesu-wifi version` command, and provided clean `pipx uninstall pesu-wifi` instructions.

#### [`c4b21a9`](https://github.com/imyash0722/pesu-wifi/commit/c4b21a9) — `chore(aur): update PKGBUILD and .SRCINFO to 2.3.0.r0.g56c2e96`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `aur/.SRCINFO`, `aur/PKGBUILD`
- **Technical Detail:** Bumped Arch Linux package metadata to reflect the tagged 2.3.0 release version.

#### [`56c2e96`](https://github.com/imyash0722/pesu-wifi/commit/56c2e96) — `chore(release): bump version to 2.3.0 and update PKGBUILD, .SRCINFO, .gitignore`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `.gitignore`, `aur/.SRCINFO`, `aur/PKGBUILD`
- **Technical Detail:** Official release milestone tagging for v2.3.0. Added Python distribution build directories (`dist/`, `build/`, `*.egg-info/`) to `.gitignore`.

#### [`914280b`](https://github.com/imyash0722/pesu-wifi/commit/914280b) — `feat(packaging): add pyproject.toml, shell completions, and GitHub Actions CI`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `pyproject.toml`, `completions/pesu-wifi.bash`, `completions/pesu-wifi.fish`, `completions/pesu-wifi.zsh`, `.github/workflows/ci.yml`, `install.sh`, `aur/PKGBUILD`, `README.md`
- **Technical Detail:**
  - Created standard PEP 517/621 `pyproject.toml` configuration exposing the `pesu-wifi` entrypoint script.
  - Implemented auto-completion definitions for Bash (`complete -F _pesu_wifi`), Fish (`complete -c pesu-wifi`), and Zsh (`_arguments`).
  - Set up matrix CI testing across Python 3.9 through 3.13 on Ubuntu runners.
  - Updated `install.sh` to install completions to `~/.local/share/bash-completion` and `~/.config/fish`.

#### [`2d420eb`](https://github.com/imyash0722/pesu-wifi/commit/2d420eb) — `fix(portal): handle FAILED status in do_login and add offline unit test suite`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `pesu_wifi.py`, `tests/test_unit.py`
- **Technical Detail:**
  - Cyberoam portal responses returning `<status>FAILED</status>` or containing campus authentication error messages are now caught cleanly with human-readable error descriptions.
  - Introduced mock-based unit tests (`tests/test_unit.py`) testing XML parsing, credential management, command dispatch, and network error handling offline without requiring live campus network access.

#### [`7f3c573`](https://github.com/imyash0722/pesu-wifi/commit/7f3c573) — `feat(daemon): add desktop notification support via notify-send`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:** Added `send_notification(title, msg, urgency)` leveraging `notify-send`. Configured alerts for successful logins, background session renewals, and fatal credential errors. Gracefully no-ops in headless or CLI environments where `notify-send` is absent.

#### [`8f5ae25`](https://github.com/imyash0722/pesu-wifi/commit/8f5ae25) — `fix(security): prevent umask race condition when creating config files`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:** Replaced `open()` followed by `os.chmod()` with atomic file descriptor creation:
  `os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)`. Eliminates the race window where credentials could temporarily be world-readable before permissions are applied.

#### [`ee75d42`](https://github.com/imyash0722/pesu-wifi/commit/ee75d42) — `fix(daemon): add campus SSID awareness and non-root interface bounce in self-healing`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:**
  - Added campus network verification: daemon verifies current SSID against known patterns (`PESU-*`, `PES-*`) before performing active login attempts or self-healing.
  - Replaced privileged `systemctl restart NetworkManager` (which failed without sudo) in Tier 3 healing with user-space `nmcli device disconnect <iface>` and `nmcli device connect <iface>`.

#### [`b7f8ce2`](https://github.com/imyash0722/pesu-wifi/commit/b7f8ce2) — `feat(cli): add native start, stop, restart commands and fix user service target`
- **Author:** Tin <yatinrajesh77@gmail.com>
- **Files Modified:** `pesu_wifi.py`, `pesu-wifi.service`, `install.sh`, `README.md`, `aur/PKGBUILD`
- **Technical Detail:**
  - Added `cmd_start()`, `cmd_stop()`, and `cmd_restart()`.
  - Commands check `is_daemon_running()`, dispatch `systemctl --user {start|stop|restart} pesu-wifi.service`, and verify termination via `pkill -f pesu[-_]wifi.*daemon`.
  - Corrected systemd install target in `pesu-wifi.service` to `default.target` for reliable user-session activation.

#### [`1cd1290`](https://github.com/imyash0722/pesu-wifi/commit/1cd1290) — `fix(daemon): add jitter retry and consecutive session drop check before re-login`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:**
  - Added a 500ms jitter retry to `check_live()` to avoid false negatives when campus APs experience packet drops.
  - In `cmd_daemon()`, required 2 consecutive failed checks before firing `do_login()`, preventing unnecessary RST packet storms when roaming between access points.

#### [`163e7d7`](https://github.com/imyash0722/pesu-wifi/commit/163e7d7) — `fix(portal): increase probe timeout to 4.0s for congested networks`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:** Increased the portal probe timeout from 2.0s to 4.0s to accommodate high network latency during peak university campus hours.

#### [`3231f99`](https://github.com/imyash0722/pesu-wifi/commit/3231f99) — `chore(aur): update PKGBUILD and .SRCINFO to 2.2.0.r5.g9fbbf9c`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Files Modified:** `aur/.SRCINFO`, `aur/PKGBUILD`
- **Technical Detail:** Synchronized package release versioning in AUR files.

#### [`9fbbf9c`](https://github.com/imyash0722/pesu-wifi/commit/9fbbf9c) — `refactor: remove pesu-wifi update command in favor of standard paru/yay package management`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:** Deprecated built-in self-update command to adhere to Arch Linux packaging best practices, allowing AUR helpers (`paru`, `yay`) and package managers (`pacman`, `pipx`) to manage package lifecycles.

#### [`6b14867`](https://github.com/imyash0722/pesu-wifi/commit/6b14867) — `feat(aur): add pesu-wifi.install for post-install setup prompt`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Files Modified:** `aur/.SRCINFO`, `aur/PKGBUILD`, `aur/pesu-wifi.install`
- **Technical Detail:** Added an Arch Linux post-installation hook that prints credential setup reminders (`pesu-wifi add`) and service activation commands.

#### [`b630d62`](https://github.com/imyash0722/pesu-wifi/commit/b630d62) — `feat: add pesu-wifi update command to fetch and install latest GitHub releases`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:** Initial implementation of release updater parsing GitHub API releases.

#### [`5bdcdcf`](https://github.com/imyash0722/pesu-wifi/commit/5bdcdcf) — `docs: emphasize GitHub release as primary install method while AUR registrations are closed`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Files Modified:** `README.md`
- **Technical Detail:** Highlighted direct pacman installation of `.pkg.tar.zst` release assets while AUR user registrations were paused.

#### [`6e5b275`](https://github.com/imyash0722/pesu-wifi/commit/6e5b275) — `docs: streamline Option 1 pacman release installation in README`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Files Modified:** `README.md`
- **Technical Detail:** Refined README commands for 1-step pacman release asset installation.

---

## v2.2.0 — Interactive Wi-Fi Selector & Multi-Account Management

**Release Date:** September 8, 2026  
**Git Tag:** [`v2.2.0`](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.2.0)  
**Commits in this Release:** 10 commits (`def9b59` -> `2729461`)

### v2.2.0 Overview & Major Highlights
- **Interactive Wi-Fi Selector (`pesu-wifi wifi`):** Scans nearby access points with signal strength bars, security status, and seamless NetworkManager connection.
- **Sub-Second Gateway Detection:** Probes Cyberoam portal in 10–30ms without unnecessary polling delays.
- **Multi-Account Credential Management:**
  - `pesu-wifi add` / `del`: Save and remove accounts.
  - `pesu-wifi select` / `use`: Instantly switch default account.
  - `pesu-wifi login <user>`: Explicitly authenticate as any saved account.
  - `pesu-wifi list -p`: Display all saved credentials on demand.
- **Multi-Tier Self-Healing Watchdog:** Auto-reconnects Wi-Fi, cycles radio hardware, and restarts NetworkManager upon network wedges.
- **Automated Verification Suite:** Comprehensive test suite in `tests/test_wifi_accounts.py`.

### v2.2.0 Commit-by-Commit Technical Breakdown

#### [`2729461`](https://github.com/imyash0722/pesu-wifi/commit/2729461) — `refactor: organize repo structure with tests/ folder, update README installation and release guides`
- **Files Modified:** `README.md`, `tests/`
- **Technical Detail:** Restructured project directory to isolate test scripts into `tests/`, updated installation documentation and release instructions.

#### [`4d14587`](https://github.com/imyash0722/pesu-wifi/commit/4d14587) — `chore(aur): update PKGBUILD and .SRCINFO to r8.18d8292`
- **Files Modified:** `aur/.SRCINFO`, `aur/PKGBUILD`
- **Technical Detail:** Synced AUR package definition with latest git revision.

#### [`18d8292`](https://github.com/imyash0722/pesu-wifi/commit/18d8292) — `docs: update README with new features and sanitize all personal data from repo`
- **Files Modified:** `README.md`
- **Technical Detail:** Sanitized personal campus credentials and test user configurations from git history, added feature matrices and command examples.

#### [`7a0f8c8`](https://github.com/imyash0722/pesu-wifi/commit/7a0f8c8) — `test: add automated Wi-Fi connection and multi-account verification suite`
- **Files Modified:** `tests/test_wifi_accounts.py`
- **Technical Detail:** Implemented automated test script verifying Wi-Fi network selection, AP roaming, multi-account rotation, and portal session persistence.

#### [`bfabed1`](https://github.com/imyash0722/pesu-wifi/commit/bfabed1) — `feat: sub-second portal check, interactive wifi selector, list -p, use/select alias`
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:**
  - Optimized socket timeouts to achieve 15–30ms portal response verification.
  - Added `pesu-wifi wifi` using `nmcli device wifi list` and interactive curses-style prompt.
  - Added `use` alias for `select` command.
  - Added `-p` flag to `pesu-wifi list` to print decrypted passwords.

#### [`29c7133`](https://github.com/imyash0722/pesu-wifi/commit/29c7133) — `feat: fresh-session-per-request, select command, login by username, list -p, fixed daemon backoff`
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:**
  - Initialized fresh `requests.Session()` instances per request to eliminate socket reuse errors on proxy changes.
  - Added account selection command (`select`).
  - Added exponential backoff handling to daemon loop.

#### [`80bed4b`](https://github.com/imyash0722/pesu-wifi/commit/80bed4b) — `fix(network): add Connection: close and eliminate DNS hang during logout`
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:** Added explicit `Connection: close` header on logout requests to prevent TCP connection pooling hangs during network teardown.

#### [`f58cfc1`](https://github.com/imyash0722/pesu-wifi/commit/f58cfc1) — `fix(cli): exact 1:1 box width alignment for status card`
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:** Computed exact terminal string widths taking ANSI escape sequences into account for box borders.

#### [`e63c0a0`](https://github.com/imyash0722/pesu-wifi/commit/e63c0a0) — `fix(cli): pixel-perfect status card with closed right borders and column alignment`
- **Files Modified:** `pesu_wifi.py`
- **Technical Detail:** Corrected Unicode box-drawing character widths and right border alignment for CLI status cards.

---

## v2.0.0 — Initial Release

#### [`def9b59`](https://github.com/imyash0722/pesu-wifi/commit/def9b59) — `feat: initial release of PESU WiFi Login Manager v2.0`
- **Author:** imyash0722 <imyash0722@users.noreply.github.com>
- **Technical Detail:**
  - Initial release of the redesigned `pesu-wifi` Python CLI.
  - Automated Cyberoam XML login protocol (`http://192.168.254.1:8090/login.xml`).
  - Keepalive daemon with single-instance `fcntl` file locking (`/tmp/pesu_wifi_daemon.lock`).
  - `systemd --user` service unit configuration.
  - Base credential configuration management.

---

## Release Management Guide

When cutting a new release:
1. Ensure all unit tests pass: `python3 -m unittest discover -s tests -p "test_*.py"`
2. Verify code syntax and typing: `python3 -m py_compile pesu_wifi.py`
3. Update version in:
   - `pesu_wifi.py` (`__version__ = "X.Y.Z"`)
   - `pyproject.toml` (`version = "X.Y.Z"`)
   - `aur/PKGBUILD` and `aur/.SRCINFO`
4. Document all changes commit-by-commit in `CHANGELOG.md`.
5. Create and push git tag:
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```
6. Build release assets and publish on GitHub:
   ```bash
   gh release create vX.Y.Z <assets> --title "vX.Y.Z - <Title>" --notes-file <notes.md>
   ```
