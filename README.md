# PESU WiFi Login Manager & Watchdog Daemon (Rust Edition)

A blazing fast, native Rust automated captive portal login manager and keepalive watchdog daemon for PES University campus Wi-Fi networks.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Release](https://img.shields.io/github/v/release/imyash0722/pesu-wifi?color=blue)](https://github.com/imyash0722/pesu-wifi/releases)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Changelog](https://img.shields.io/badge/Changelog-Release%20Notes-green.svg)](CHANGELOG.md)

---

## Why Rust?

- **Zero Runtime Dependencies:** Compiles down to a single standalone static binary (~2.3 MB). No Python runtime, `pip`, or virtualenv required.
- **Microsecond Cold Start:** Launches in **<2ms** instead of ~100ms for Python startup.
- **Ultra-Low Memory Footprint:** Background daemon consumes only **~3 MB RSS** memory (10x lighter than Python's ~40 MB).
- **Rock-Solid Reliability:** Compile-time memory safety prevents runtime crashes during 24/7 background operation.

---

## Features

- **Automated Keepalive Watchdog:** Heartbeat polling keeps your captive portal session alive indefinitely, even when browsers are closed or devices idle.
- **Campus SSID Auto-Standby:** Automatically detects whether you are connected to a campus network. Gracefully pauses watchdog activities when at home or on non-campus Wi-Fi.
- **Desktop Notifications:** Dispatches native system notifications (`notify-send`) on login, session renewal, and authentication errors.
- **Sub-Second Status Detection:** Probes the local gateway in 10–30ms to detect session states without redundant network traffic.
- **Multi-Account Switching:** Save multiple student accounts, switch the active user instantly, or explicitly log in under a specific account.
- **Secure Atomic Credential Storage:** Stores credentials locally in `~/.config/pesu-wifi/config.json` and `~/.config/pesu-wifi/.env` with strict `0600` permissions.
- **Resilient Multi-Tier Self-Healing:**
  - **Tier 1:** NetworkManager connection renegotiation.
  - **Tier 2:** Wi-Fi radio power-cycling.
  - **Tier 3:** Non-root Wi-Fi interface recovery.
- **Shell Auto-Completions:** Native tab-completion support for Bash, Zsh, and Fish shells.
- **Modern CLI:** Clean ANSI status cards, colored indicators, and comprehensive verification suite.

---

## Installation & Building

### Option A: Pre-compiled Standalone Binary (Any Linux x86_64)

```bash
curl -sSL https://github.com/imyash0722/pesu-wifi/releases/download/v3.1.0/pesu-wifi-v3.1.0-linux-x86_64.tar.gz | tar -xz
cd pesu-wifi-v3.1.0-linux-x86_64 && ./install.sh
```

### Option B: Arch Linux / CachyOS (`.pkg.tar.zst`)

```bash
sudo pacman -U https://github.com/imyash0722/pesu-wifi/releases/download/v3.1.0/pesu-wifi-3.1.0-x86_64.pkg.tar.zst
```

### Option C: Build from Source (Requires Cargo / Rust)

```bash
# Clone the repository
git clone https://github.com/imyash0722/pesu-wifi.git ~/pesu-wifi
cd ~/pesu-wifi

# Build release binary
cargo build --release

# Run one-click setup script (installs binary, completions, and systemd service)
./install.sh
```

The compiled release binary is located at `target/release/pesu-wifi`.

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

### Options & Top-Level Flags
```bash
# Display help information (-h, --help)
pesu-wifi -h
pesu-wifi --help

# Print version information (-v, --version)
pesu-wifi -v
pesu-wifi --version

# List all saved accounts and stored passwords directly (-p, --passwords)
pesu-wifi -p
pesu-wifi --passwords

# Subcommands also support contextual help
pesu-wifi <subcommand> --help
```

### Connection & Status
```bash
# View live connection status overview card
pesu-wifi status
```

### Authentication & Sessions
```bash
# Smart login using the active account
pesu-wifi login

# Explicitly login using a specific saved account
pesu-wifi login student1

# Clean captive portal sign-out
pesu-wifi logout
```

### Account Management
```bash
# Add or update an account (interactive prompt)
pesu-wifi add

# Add credentials directly via arguments
pesu-wifi add username password

# Select/switch default account (interactive menu)
pesu-wifi select
pesu-wifi select student2

# List saved accounts
pesu-wifi list

# List saved accounts showing stored passwords
pesu-wifi list -p
pesu-wifi list --passwords

# Delete an account
pesu-wifi del student1
```

### Daemon & Background Service
```bash
# Start background watchdog daemon via systemd user service
pesu-wifi start

# Run watchdog in the foreground (useful for debugging or containers)
pesu-wifi start -f
pesu-wifi start --foreground

# Stop background watchdog daemon
pesu-wifi stop
```

---

## Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PESU_USERNAME` | Override active login username | *(Config file)* |
| `PESU_PASSWORD` | Override active login password | *(Config file)* |
| `PESU_PORTAL_BASE` | Portal gateway URL | `http://192.168.254.1:8090` |
| `PESU_WIFI_CON` | Preferred Wi-Fi connection name | `PESU-EC-Campus` |
| `FORCE_COLOR` | Force ANSI color formatting (`1` / `0`) | Auto-detected |

---

## Repository Structure

```text
pesu-wifi/
├── Cargo.toml               # Rust package & clap dependency definitions
├── Cargo.lock               # Cargo dependency lockfile
├── src/
│   ├── main.rs              # CLI entry point, clap derive parser & command routing
│   ├── portal.rs            # Cyberoam HTTP portal client & XML parser
│   ├── config.rs            # Multi-account configuration & credentials store
│   ├── wifi.rs              # NetworkManager & interface recovery controller
│   ├── daemon.rs            # Background keepalive watchdog & notification loop
│   └── ui.rs                # ANSI styling, cards & terminal formatting
├── install.sh               # Standalone release builder and system installer
├── pesu-wifi.service        # Systemd user service unit definition
├── completions/             # Shell auto-completion scripts
│   ├── pesu-wifi.bash       # Bash completions
│   ├── pesu-wifi.zsh        # Zsh completions
│   └── pesu-wifi.fish       # Fish completions
├── aur/
│   ├── PKGBUILD             # Arch User Repository package build definition
│   ├── .SRCINFO             # Arch User Repository package metadata
│   └── pesu-wifi.install    # Arch package post-install instructions
├── .github/workflows/
│   └── ci.yml               # Automated Rust GitHub Actions CI
├── CHANGELOG.md             # Comprehensive commit-by-commit release notes
├── LICENSE                  # MIT License
└── README.md                # Documentation and guide
```

---

## Release History & Changelog

- [**v3.1.0**](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.1.0) ([Notes](CHANGELOG.md#v310--clap-derive-migration--production-cli-architecture)) — Full migration to `clap` derive parser, automatic multi-level help screens, contextual typo suggestions, and strict POSIX flag validation.
- [**v3.0.4**](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.4) ([Notes](CHANGELOG.md#v304--strict-standard-posix-flags--clean-syntax-enforcement)) — Strict POSIX/GNU options enforcement (`-v`, `--version`, `-h`, `--help`), pruned informal command aliases, and standard exit code `2`.
- [**v3.0.3**](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.3) ([Notes](CHANGELOG.md#v303--top-level-password-flags--enhanced-cli-flag-support)) — Top-level `-p` / `--passwords` flag support and root-level shell completions.
- [**v3.0.2**](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.2) ([Notes](CHANGELOG.md#v302--standardized-cli-options-unified-daemon-controls--clean-output)) — Standardized CLI options, unified foreground/background daemon controls (`start [-f, --foreground]`), and pruned redundant commands.
- [**v3.0.1**](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.1) ([Notes](CHANGELOG.md#v301--build-stability-non-posix-filesystem-support--zero-python-completions)) — Build stability on external/exFAT mounts, zero-Python completions, daemon path hardening, and test suite expansion.
- [**v3.0.0**](https://github.com/imyash0722/pesu-wifi/releases/tag/v3.0.0) — Complete native Rust rewrite with sub-millisecond cold starts, ~3MB resident daemon memory, zero Python runtime dependencies, and standalone binary distribution.
- [**v2.3.0**](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.3.0) — Process controls (`start`/`stop`/`restart`), campus SSID auto-standby, desktop notifications, atomic umask hardening, anti-storm jitter protection, and multi-packaging CI.
- [**v2.2.0**](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.2.0) — Interactive Wi-Fi selector, sub-second gateway detection, multi-account credentials management, and verification suite.
- [**v2.0.0**](https://github.com/imyash0722/pesu-wifi/releases/tag/v2.0.0) — Initial rewritten Python CLI release with automated Cyberoam login & keepalive.

---

## Uninstallation

To remove `pesu-wifi`:

**If installed via AUR or `.pkg.tar.zst`:**
```bash
sudo pacman -R pesu-wifi-git
```

**If installed via `install.sh` / source:**
```bash
systemctl --user disable --now pesu-wifi.service
rm -f ~/.local/bin/pesu-wifi /usr/local/bin/pesu-wifi
rm -f ~/.config/systemd/user/pesu-wifi.service
rm -f ~/.local/share/bash-completion/completions/pesu-wifi
rm -f ~/.config/fish/completions/pesu-wifi.fish
systemctl --user daemon-reload
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.
