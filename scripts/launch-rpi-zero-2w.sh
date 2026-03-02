#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="${TREKR_APP_DIR:-$SCRIPT_DIR}"
APP_BIN="${TREKR_APP_BIN:-$APP_DIR/trekr}"

if [[ ! -x "$APP_BIN" ]]; then
    echo "trekr binary is not executable: $APP_BIN" >&2
    exit 1
fi

cd "$APP_DIR"
exec "$APP_BIN" --video-mode kmsdrm-console "$@"
