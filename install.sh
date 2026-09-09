#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

echo "=========================================="
echo " PESU WiFi Login Manager & Daemon Setup"
echo "=========================================="
echo ""

# 1. Ensure Python dependencies
echo "[1/4] Checking python-requests dependency..."
if ! python3 -c "import requests" &>/dev/null; then
    echo "Installing 'requests' module..."
    if command -v pacman &>/dev/null; then
        sudo pacman -S --needed --noconfirm python-requests || pip install requests --break-system-packages
    elif command -v apt &>/dev/null; then
        sudo apt-get install -y python3-requests || pip install requests --break-system-packages
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y python3-requests || pip install requests --break-system-packages
    else
        pip install requests
    fi
else
    echo "  ✔ python-requests is already installed."
fi

# 2. Setup binary symlink & PATH
echo "[2/4] Setting up CLI symlink..."
mkdir -p "$BIN_DIR"
chmod +x "$SCRIPT_DIR/pesu_wifi.py"
ln -sf "$SCRIPT_DIR/pesu_wifi.py" "$BIN_DIR/pesu-wifi"
echo "  ✔ Symlinked: $BIN_DIR/pesu-wifi -> $SCRIPT_DIR/pesu_wifi.py"

# Try symlinking to /usr/local/bin if accessible (for immediate system-wide PATH availability)
if [ -w "/usr/local/bin" ]; then
    ln -sf "$SCRIPT_DIR/pesu_wifi.py" "/usr/local/bin/pesu-wifi"
    echo "  ✔ Symlinked to /usr/local/bin/pesu-wifi"
fi

# Ensure ~/.local/bin is in shell PATH config files
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
if [ -x "/usr/bin/pesu-wifi" ]; then
    cp "$SCRIPT_DIR/pesu-wifi.service" "$SYSTEMD_USER_DIR/pesu-wifi.service"
else
    sed "s|/usr/bin/pesu-wifi|$BIN_DIR/pesu-wifi|g" "$SCRIPT_DIR/pesu-wifi.service" > "$SYSTEMD_USER_DIR/pesu-wifi.service"
fi

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
    echo "  ✔ Background service configured and started."
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
    echo "  Or simply open a new terminal window / reload your shell."
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
