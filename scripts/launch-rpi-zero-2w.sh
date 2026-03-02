#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="${TREKR_APP_DIR:-$SCRIPT_DIR}"
APP_BIN="${TREKR_APP_BIN:-$APP_DIR/trekr}"
export LD_LIBRARY_PATH="$APP_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export SDL_VIDEODRIVER="${SDL_VIDEODRIVER:-kmsdrm}"
export SDL_RENDER_DRIVER="${SDL_RENDER_DRIVER:-software}"

if [[ ! -x "$APP_BIN" ]]; then
    echo "trekr binary is not executable: $APP_BIN" >&2
    exit 1
fi

cd "$APP_DIR"
exec "$APP_BIN" --video-mode kmsdrm-console "$@"
