#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
    echo "Run as root or via sudo: sudo ./setup-rpi-zero-2w-runtime.sh" >&2
    exit 1
fi

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y \
    libasound2 \
    libdrm2 \
    libegl1 \
    libgbm1 \
    libgl1 \
    libgles2 \
    libinput10 \
    libudev1 \
    libxkbcommon0
