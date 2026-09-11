#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  ./stream-rpi-kmsgrab.sh [--transport tcp-listen|udp-push] [options]

Streams the current KMS/DRM display as MPEG-TS for OBS.

Options:
  --transport <mode>     tcp-listen lets OBS connect to the Pi. Default: tcp-listen.
                         udp-push sends packets to --obs-host.
  --obs-host <ip>        OBS receiver address on the Windows machine for udp-push.
  --obs-port <port>      UDP destination port. Default: 1234.
  --framerate <fps>      Capture frame rate. Default: 30.
  --bitrate <rate>       H.264 video bitrate. Default: 3500k.
  --preset <preset>      libx264 preset. Default: ultrafast.
  --drm-device <path>    DRM device. Default: /dev/dri/card0.
  --crtc-id <id>         Optional kmsgrab CRTC id.
  --plane-id <id>        Optional kmsgrab plane id.
  -h, --help             Show this help.

OBS media source input:
  tcp-listen: tcp://<pi-ip>:1234
  udp-push:   udp://@0.0.0.0:1234
USAGE
}

TRANSPORT="${TREKR_KMSGRAB_TRANSPORT:-tcp-listen}"
OBS_HOST="${TREKR_OBS_HOST:-}"
OBS_PORT="${TREKR_OBS_PORT:-1234}"
FRAMERATE="${TREKR_KMSGRAB_FRAMERATE:-30}"
BITRATE="${TREKR_KMSGRAB_BITRATE:-3500k}"
PRESET="${TREKR_KMSGRAB_PRESET:-ultrafast}"
DRM_DEVICE="${TREKR_KMSGRAB_DRM_DEVICE:-/dev/dri/card0}"
CRTC_ID="${TREKR_KMSGRAB_CRTC_ID:-}"
PLANE_ID="${TREKR_KMSGRAB_PLANE_ID:-}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --transport)
            TRANSPORT="${2:-}"
            shift 2
            ;;
        --obs-host)
            OBS_HOST="${2:-}"
            shift 2
            ;;
        --obs-port)
            OBS_PORT="${2:-}"
            shift 2
            ;;
        --framerate)
            FRAMERATE="${2:-}"
            shift 2
            ;;
        --bitrate)
            BITRATE="${2:-}"
            shift 2
            ;;
        --preset)
            PRESET="${2:-}"
            shift 2
            ;;
        --drm-device)
            DRM_DEVICE="${2:-}"
            shift 2
            ;;
        --crtc-id)
            CRTC_ID="${2:-}"
            shift 2
            ;;
        --plane-id)
            PLANE_ID="${2:-}"
            shift 2
            ;;
        -h|--help)
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

if ! command -v ffmpeg >/dev/null 2>&1; then
    echo "ffmpeg was not found on the Pi. Install it with: sudo apt-get update && sudo apt-get install -y ffmpeg" >&2
    exit 127
fi

if [[ ! -e "$DRM_DEVICE" ]]; then
    echo "DRM device not found: $DRM_DEVICE" >&2
    exit 1
fi

case "$TRANSPORT" in
    tcp-listen)
        destination="tcp://0.0.0.0:${OBS_PORT}?listen=1"
        obs_input="tcp://$(hostname -I | awk '{print $1}'):${OBS_PORT}"
        ;;
    udp-push)
        if [[ -z "$OBS_HOST" ]]; then
            echo "Missing --obs-host <windows-ip> for udp-push transport." >&2
            usage >&2
            exit 2
        fi
        destination="udp://${OBS_HOST}:${OBS_PORT}?pkt_size=1316"
        obs_input="udp://@0.0.0.0:${OBS_PORT}"
        ;;
    *)
        echo "Unknown transport: $TRANSPORT" >&2
        usage >&2
        exit 2
        ;;
esac

kmsgrab_args=(
    -f kmsgrab
    -framerate "$FRAMERATE"
    -device "$DRM_DEVICE"
)

if [[ -n "$CRTC_ID" ]]; then
    kmsgrab_args+=(-crtc_id "$CRTC_ID")
fi

if [[ -n "$PLANE_ID" ]]; then
    kmsgrab_args+=(-plane_id "$PLANE_ID")
fi

echo "Streaming KMSDRM capture from $DRM_DEVICE to $destination"
echo "OBS Media Source input: $obs_input"
echo "Press Ctrl+C here to stop."

exec ffmpeg \
    -hide_banner \
    "${kmsgrab_args[@]}" \
    -i - \
    -vf "hwdownload,format=bgr0,format=yuv420p" \
    -an \
    -c:v libx264 \
    -preset "$PRESET" \
    -tune zerolatency \
    -b:v "$BITRATE" \
    -maxrate "$BITRATE" \
    -bufsize "$BITRATE" \
    -f mpegts \
    "$destination"
