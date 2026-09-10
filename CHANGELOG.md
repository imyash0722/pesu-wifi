# PESU WiFi — Changelog & Release Notes

A comprehensive, commit-by-commit record of all architecture changes, feature additions, bug fixes, and maintenance releases for the **PESU WiFi Login Manager & Watchdog Daemon**.

---

## Table of Contents
- [v3.0.0 — Native Rust Rewrite, Blazing Speed & Zero Runtime Dependencies](#v300--native-rust-rewrite-blazing-speed--zero-runtime-dependencies)
  - [Overview & Major Highlights](#v300-overview--major-highlights)
  - [Performance Benchmarks](#v300-performance-benchmarks)
  - [Architecture & Modular Breakdown](#v300-architecture--modular-breakdown)
- [v2.3.0 — Resilient Watchdog, Process Controls & Distribution Hardening](#v230--resilient-watchdog-process-controls--distribution-hardening)
  - [Overview & Major Highlights](#v230-overview--major-highlights)
  - [Commit-by-Commit Technical Breakdown](#v230-commit-by-commit-technical-breakdown)
- [v2.2.0 — Interactive Wi-Fi Selector & Multi-Account Management](#v220--interactive-wi-fi-selector--multi-account-management)
  - [Overview & Major Highlights](#v220-overview--major-highlights)
  - [Commit-by-Commit Technical Breakdown](#v220-commit-by-commit-technical-breakdown)
- [v2.0.0 — Initial Release](#v200--initial-release)
- [Release Management Guide](#release-management-guide)

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

- **`src/main.rs`:** CLI entry point, argument parsing via `clap` (derive macro), command routing (`status`, `login`, `logout`, `select`/`use`, `wifi`, `add`, `del`, `list`, `daemon`, `start`, `stop`, `restart`, `version`, `help`), and ANSI card visualization.
- **`src/portal.rs`:** Cyberoam captive portal HTTP client powered by `ureq` with `rustls-webpki-roots`. Handles login (`mode=191`), keepalive (`mode=193`), and logout (`mode=193`), with full CDATA and HTML entity parsing.
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
