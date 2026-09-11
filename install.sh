#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

echo "=========================================="
echo " PESU WiFi Rust CLI & Daemon Setup"
echo "=========================================="
echo ""

# 1. Ensure Rust toolchain
echo "[1/4] Checking Rust toolchain..."
if ! command -v cargo &>/dev/null; then
    echo "Cargo is not installed. Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "  ✔ Cargo is installed: $(cargo --version)"
fi

# 2. Build release binary
echo "[2/4] Building high-performance release binary..."
cd "$SCRIPT_DIR"

# Protect against metadata corruption on non-POSIX/exFAT filesystems
if [ -z "$CARGO_TARGET_DIR" ]; then
    TARGET_FS=$(df -T "$SCRIPT_DIR" 2>/dev/null | awk 'NR==2 {print $2}')
    if [[ "$TARGET_FS" =~ ^(exfat|vfat|msdos|cifs|smb|ntfs|fuseblk)$ ]]; then
        export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/pesu-wifi"
        mkdir -p "$CARGO_TARGET_DIR"
    fi
fi

cargo build --release

# Locate built binary
if [ -n "$CARGO_TARGET_DIR" ] && [ -f "$CARGO_TARGET_DIR/release/pesu-wifi" ]; then
    BUILD_BIN="$CARGO_TARGET_DIR/release/pesu-wifi"
elif [ -f "$SCRIPT_DIR/target/release/pesu-wifi" ]; then
    BUILD_BIN="$SCRIPT_DIR/target/release/pesu-wifi"
else
    BUILD_BIN=$(find "${CARGO_TARGET_DIR:-$HOME/.cache/cargo-target}" "$SCRIPT_DIR/target" -name "pesu-wifi" -type f -perm -111 2>/dev/null | grep -E 'release/pesu-wifi$' | head -n 1)
fi

if [ -z "$BUILD_BIN" ] || [ ! -f "$BUILD_BIN" ]; then
    echo "❌ Error: Could not locate built release binary."
    exit 1
fi

mkdir -p "$BIN_DIR"
rm -f "$BIN_DIR/pesu-wifi"
cp "$BUILD_BIN" "$BIN_DIR/pesu-wifi"
chmod +x "$BIN_DIR/pesu-wifi"
echo "  ✔ Installed binary to: $BIN_DIR/pesu-wifi"

# Try installing to /usr/local/bin if writable
if [ -w "/usr/local/bin" ]; then
    rm -f "/usr/local/bin/pesu-wifi"
    cp "$BUILD_BIN" "/usr/local/bin/pesu-wifi"
    echo "  ✔ Copied to /usr/local/bin/pesu-wifi"
fi

# Ensure ~/.local/bin is in shell PATH
for rcfile in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.profile"; do
    if [ -f "$rcfile" ]; then
        if ! grep -q '\.local/bin' "$rcfile"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$rcfile"
            echo "  ✔ Added ~/.local/bin to PATH in $(basename "$rcfile")"
        fi
    fi
done

# Install shell completions
mkdir -p "$HOME/.local/share/bash-completion/completions"
cp "$SCRIPT_DIR/completions/pesu-wifi.bash" "$HOME/.local/share/bash-completion/completions/pesu-wifi" 2>/dev/null || true
if [ -d "$HOME/.config/fish" ]; then
    mkdir -p "$HOME/.config/fish/completions"
    cp "$SCRIPT_DIR/completions/pesu-wifi.fish" "$HOME/.config/fish/completions/pesu-wifi.fish" 2>/dev/null || true
fi
echo "  ✔ Installed shell completions."

# 3. Setup systemd user service
echo "[3/4] Setting up systemd user service..."
mkdir -p "$SYSTEMD_USER_DIR"
sed "s|/usr/bin/pesu-wifi|$BIN_DIR/pesu-wifi|g" "$SCRIPT_DIR/pesu-wifi.service" > "$SYSTEMD_USER_DIR/pesu-wifi.service"

# 4. Configure background service
echo "[4/4] Configuring background service..."
systemctl --user daemon-reload
systemctl --user enable pesu-wifi.service

# Prompt to add credentials if not configured
if [ ! -f "$HOME/.config/pesu-wifi/config.json" ] && [ ! -f "$HOME/.config/pesu-wifi/.env" ]; then
    echo ""
    echo "No saved credentials found. Would you like to configure them now? [Y/n]"
    read -r resp
    if [[ -z "$resp" || "$resp" =~ ^[Yy]$ ]]; then
        "$BIN_DIR/pesu-wifi" add || true
    fi
fi

# Start service if credentials exist
if [ -f "$HOME/.config/pesu-wifi/config.json" ] || [ -f "$HOME/.config/pesu-wifi/.env" ]; then
    systemctl --user restart pesu-wifi.service 2>/dev/null || true
    echo "  ✔ Background service configured and running."
else
    echo "  ⚠ Note: Run 'pesu-wifi add' then 'pesu-wifi start' to start the daemon."
fi

echo ""
echo "=========================================="
echo " Installation Complete!"
echo "=========================================="
echo ""
if [[ ":$PATH:" != *":$HOME/.local/bin:"* && ! -f "/usr/local/bin/pesu-wifi" ]]; then
    echo "⚠ Note: ~/.local/bin is not in your current terminal's PATH yet."
    echo "  Run:  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo "  Or open a new terminal window / reload your shell."
    echo ""
fi

echo "Commands:"
echo "  pesu-wifi status           - Check live connection status"
echo "  pesu-wifi start            - Start background watchdog daemon"
echo "  pesu-wifi stop             - Stop background watchdog daemon"
echo "  pesu-wifi restart          - Restart background watchdog daemon"
echo "  pesu-wifi wifi             - Interactive Wi-Fi network selector"
echo "  pesu-wifi login [user]     - Smart login (or explicit user)"
echo "  pesu-wifi logout           - Clean captive portal sign-out"
echo "  pesu-wifi select [user]    - Set default account (alias: use)"
echo "  pesu-wifi list [-p]        - List saved accounts (-p for passwords)"
echo "  pesu-wifi add              - Save or update credentials"
echo "  pesu-wifi del [user]       - Remove a saved account"
echo "  pesu-wifi daemon           - Run keepalive watchdog in foreground"
echo ""
echo "Logs:"
echo "  journalctl --user -u pesu-wifi -f"
echo ""
