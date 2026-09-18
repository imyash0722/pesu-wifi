#!/usr/bin/env bash
# NetworkManager Dispatcher Script for PESU WiFi
# Automatically triggers instant captive portal login upon connecting to PESU campus Wi-Fi.
# Install to: /etc/NetworkManager/dispatcher.d/99-pesu-wifi.sh
# Permissions: chmod +x /etc/NetworkManager/dispatcher.d/99-pesu-wifi.sh

INTERFACE="$1"
ACTION="$2"

if [ "$ACTION" = "up" ]; then
    # Check if the active connection is a campus network
    SSID=$(nmcli -t -f active,ssid dev wifi 2>/dev/null | grep '^yes:' | cut -d: -f2)
    if [[ "$SSID" =~ (PESU|PES-WIFI|PES_WIFI|PESUNIVERSITY) ]]; then
        # Wake or trigger pesu-wifi login for active logged-in graphical user(s)
        for user in $(loginctl list-users --no-legend 2>/dev/null | awk '{print $2}'); do
            su - "$user" -c "pesu-wifi login" 2>/dev/null || true
        done
    fi
fi
