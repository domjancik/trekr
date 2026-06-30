#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SETUP_SRC="${SCRIPT_DIR}/setup-mpc-midi-gadget.sh"
SERVICE_SRC="${SCRIPT_DIR}/mpc-midi-gadget.service"
SETUP_DEST="${TREKR_MIDI_GADGET_SETUP_DEST:-/usr/local/sbin/setup-mpc-midi-gadget.sh}"
SERVICE_DEST="${TREKR_MIDI_GADGET_SERVICE_DEST:-/etc/systemd/system/mpc-midi-gadget.service}"
SERVICE_NAME="$(basename "${SERVICE_DEST}")"

log() {
    printf '[trekr-midi-gadget] %s\n' "$*"
}

die() {
    printf '[trekr-midi-gadget] ERROR: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'USAGE'
Usage:
  trekr-midi-gadget.sh run
  trekr-midi-gadget.sh stop
  trekr-midi-gadget.sh enable-boot
  trekr-midi-gadget.sh disable-boot
  trekr-midi-gadget.sh status
  trekr-midi-gadget.sh uninstall

Commands:
  run           Configure the Trekr USB MIDI gadget for this boot only.
  stop          Tear down the current Trekr USB MIDI gadget.
  enable-boot   Install the setup script and systemd unit, then enable/start it.
  disable-boot  Disable/stop the systemd unit and tear down any current gadget.
  status        Show systemd status when installed, then gadget status.
  uninstall     Disable boot setup, remove installed service/script files.

Deploy all three files into one directory, such as ~/trekr-midi-gadget:
  setup-mpc-midi-gadget.sh
  mpc-midi-gadget.service
  trekr-midi-gadget.sh
USAGE
}

have_cmd() {
    command -v "$1" >/dev/null 2>&1
}

as_root() {
    if [[ "${EUID}" -eq 0 ]]; then
        "$@"
    else
        sudo "$@"
    fi
}

require_file() {
    local path="$1"
    [[ -f "${path}" ]] || die "Missing required file: ${path}"
}

require_setup_src() {
    require_file "${SETUP_SRC}"
}

require_service_src() {
    require_file "${SERVICE_SRC}"
}

require_systemctl() {
    have_cmd systemctl || die "systemctl is not available on this system."
}

installed_setup() {
    [[ -x "${SETUP_DEST}" ]]
}

run_setup() {
    require_setup_src
    as_root bash "${SETUP_SRC}" --setup
}

run_teardown() {
    if [[ -f "${SETUP_SRC}" ]]; then
        as_root bash "${SETUP_SRC}" --teardown
    elif installed_setup; then
        as_root "${SETUP_DEST}" --teardown
    else
        die "No setup script found locally or at ${SETUP_DEST}."
    fi
}

install_files() {
    require_setup_src
    require_service_src

    log "Installing setup script to ${SETUP_DEST}"
    as_root install -m 755 "${SETUP_SRC}" "${SETUP_DEST}"

    log "Installing systemd unit to ${SERVICE_DEST}"
    as_root install -m 644 "${SERVICE_SRC}" "${SERVICE_DEST}"
}

enable_boot() {
    require_systemctl
    install_files

    log "Reloading systemd"
    as_root systemctl daemon-reload

    log "Enabling and starting ${SERVICE_NAME}"
    as_root systemctl enable --now "${SERVICE_NAME}"

    log "Boot-time Trekr MIDI gadget setup is enabled."
}

disable_boot() {
    require_systemctl

    if [[ -f "${SERVICE_DEST}" ]]; then
        log "Disabling and stopping ${SERVICE_NAME}"
        as_root systemctl disable --now "${SERVICE_NAME}" || true
        as_root systemctl daemon-reload
    else
        log "Systemd unit is not installed at ${SERVICE_DEST}"
    fi

    log "Tearing down any current Trekr MIDI gadget"
    run_teardown
}

status() {
    if have_cmd systemctl && [[ -f "${SERVICE_DEST}" ]]; then
        as_root systemctl status "${SERVICE_NAME}" --no-pager || true
        printf '\n'
    fi

    if [[ -f "${SETUP_SRC}" ]]; then
        as_root bash "${SETUP_SRC}" --status
    elif installed_setup; then
        as_root "${SETUP_DEST}" --status
    else
        die "No setup script found locally or at ${SETUP_DEST}."
    fi
}

uninstall() {
    disable_boot

    log "Removing ${SERVICE_DEST}"
    as_root rm -f "${SERVICE_DEST}"

    log "Removing ${SETUP_DEST}"
    as_root rm -f "${SETUP_DEST}"

    if have_cmd systemctl; then
        as_root systemctl daemon-reload
    fi

    log "Trekr MIDI gadget service files removed."
}

main() {
    local command="${1:-}"

    case "${command}" in
        run|setup|once)
            run_setup
            ;;
        stop|teardown)
            run_teardown
            ;;
        enable-boot|enable|permanent)
            enable_boot
            ;;
        disable-boot|disable|temporary)
            disable_boot
            ;;
        status)
            status
            ;;
        uninstall|remove)
            uninstall
            ;;
        --help|-h|help|"")
            usage
            ;;
        *)
            usage >&2
            exit 2
            ;;
    esac
}

main "$@"
