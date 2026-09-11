#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: sudo ./install-trekr-ap-service.sh --passphrase <WPA2 passphrase> [--ssid trekr] [--activate]

Creates the `trekr-ap` NetworkManager profile and the `trekr-ap.service`
systemd unit. The access point uses 2.4 GHz channel 6 and NetworkManager's
shared IPv4 mode (Pi: 10.42.0.1) so Ableton Link peers have a local LAN.

--activate switches wlan0 from its current Wi-Fi network to the access point
immediately. This will disconnect any SSH session that currently reaches the
Pi through wlan0.
EOF
}

ssid="trekr"
passphrase=""
activate=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --ssid)
            ssid="${2:-}"
            shift 2
            ;;
        --passphrase)
            passphrase="${2:-}"
            shift 2
            ;;
        --activate)
            activate=true
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ "${EUID}" -ne 0 ]]; then
    echo "Run as root, for example: sudo $0 --passphrase <passphrase>" >&2
    exit 1
fi

if [[ -z "$ssid" || ${#ssid} -gt 32 ]]; then
    echo "SSID must contain between 1 and 32 characters." >&2
    exit 2
fi

if [[ ${#passphrase} -lt 8 || ${#passphrase} -gt 63 ]]; then
    echo "WPA2 passphrase must contain between 8 and 63 characters." >&2
    exit 2
fi

if ! command -v nmcli >/dev/null; then
    echo "NetworkManager (nmcli) is required to create the trekr access point." >&2
    exit 1
fi

if ! nmcli -g GENERAL.TYPE device show wlan0 2>/dev/null | grep -qx 'wifi'; then
    echo "wlan0 is not an available Wi-Fi interface." >&2
    exit 1
fi

if ! nmcli -t -f NAME connection show | grep -Fxq 'trekr-ap'; then
    nmcli connection add type wifi ifname wlan0 con-name trekr-ap ssid "$ssid"
fi

nmcli connection modify trekr-ap \
    connection.autoconnect yes \
    connection.autoconnect-priority 100 \
    802-11-wireless.mode ap \
    802-11-wireless.band bg \
    802-11-wireless.channel 6 \
    802-11-wireless.powersave 2 \
    802-11-wireless-security.key-mgmt wpa-psk \
    802-11-wireless-security.psk "$passphrase" \
    ipv4.method shared \
    ipv4.addresses 10.42.0.1/24 \
    ipv6.method disabled

install -d -m 0755 /etc/systemd/system
cat >/etc/systemd/system/trekr-ap.service <<'EOF'
[Unit]
Description=Trekr Ableton Link Wi-Fi access point
Wants=NetworkManager.service
After=NetworkManager.service

[Service]
Type=oneshot
ExecStart=/usr/bin/nmcli --wait 30 connection up trekr-ap ifname wlan0
ExecStop=/usr/bin/nmcli connection down trekr-ap ifname wlan0
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

cat >/usr/local/sbin/trekr-ap <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  sudo trekr-ap on
  sudo trekr-ap off
  sudo trekr-ap connect <SSID> [WPA2-passphrase]
  trekr-ap status

`on` makes the trekr access point the boot default and starts it now.
`off` stops it now and lets normal saved Wi-Fi profiles connect at boot.
`connect` turns the access point off, then joins a normal Wi-Fi network.
USAGE
}

case "${1:-}" in
    on)
        nmcli connection modify trekr-ap connection.autoconnect yes
        systemctl enable --now trekr-ap.service
        ;;
    off)
        systemctl disable --now trekr-ap.service
        nmcli connection modify trekr-ap connection.autoconnect no
        ;;
    connect)
        ssid="${2:-}"
        passphrase="${3:-}"
        if [[ -z "$ssid" ]]; then
            usage >&2
            exit 2
        fi
        systemctl disable --now trekr-ap.service
        nmcli connection modify trekr-ap connection.autoconnect no
        if [[ -n "$passphrase" ]]; then
            nmcli device wifi connect "$ssid" password "$passphrase" ifname wlan0
        else
            nmcli device wifi connect "$ssid" ifname wlan0
        fi
        ;;
    status)
        systemctl is-enabled trekr-ap.service 2>/dev/null || true
        systemctl is-active trekr-ap.service 2>/dev/null || true
        nmcli -f GENERAL.CONNECTION,IP4.ADDRESS device show wlan0
        ;;
    --help|-h|help)
        usage
        ;;
    *)
        usage >&2
        exit 2
        ;;
esac
EOF
chmod 0755 /usr/local/sbin/trekr-ap

systemctl daemon-reload
systemctl enable trekr-ap.service

if [[ "$activate" == true ]]; then
    systemctl start trekr-ap.service
fi

echo "trekr access-point service installed. SSID: $ssid"
echo "The Pi will use 10.42.0.1/24 when the access point is active."
