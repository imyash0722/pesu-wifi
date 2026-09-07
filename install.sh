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

# 3. Setup systemd user service
echo "[3/4] Setting up systemd user service..."
mkdir -p "$SYSTEMD_USER_DIR"
if [ -x "/usr/bin/pesu-wifi" ]; then
    cp "$SCRIPT_DIR/pesu-wifi.service" "$SYSTEMD_USER_DIR/pesu-wifi.service"
else
    sed "s|/usr/bin/pesu-wifi|$BIN_DIR/pesu-wifi|g" "$SCRIPT_DIR/pesu-wifi.service" > "$SYSTEMD_USER_DIR/pesu-wifi.service"
fi

# 4. Enable and restart service
echo "[4/4] Enabling and starting background service..."
systemctl --user daemon-reload
systemctl --user enable --now pesu-wifi.service

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

# Prompt to add credentials if not configured
if [ ! -f "$HOME/.config/pesu-wifi/config.json" ] && [ ! -f "$HOME/.config/pesu-wifi/.env" ]; then
    echo "No saved credentials found. Would you like to configure them now? [Y/n]"
    read -r resp
    if [[ -z "$resp" || "$resp" =~ ^[Yy]$ ]]; then
        "$BIN_DIR/pesu-wifi" add || true
    fi
fi

echo "Commands:"
echo "  pesu-wifi status   - Check live session status"
echo "  pesu-wifi login    - Trigger manual login"
echo "  pesu-wifi logout   - Trigger manual logout"
echo "  pesu-wifi add      - Save login credentials"
echo "  pesu-wifi del      - Remove a saved account"
echo "  pesu-wifi list     - List saved accounts"
echo "  pesu-wifi daemon   - Run keepalive loop in foreground"
echo ""
echo "Logs:"
echo "  journalctl --user -u pesu-wifi -f"
echo ""
