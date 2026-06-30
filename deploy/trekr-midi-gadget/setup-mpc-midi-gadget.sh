#!/usr/bin/env bash
set -Eeuo pipefail

GADGET_NAME="mpc_midi"
GADGET_DIR="/sys/kernel/config/usb_gadget/${GADGET_NAME}"
CONFIGFS_DIR="/sys/kernel/config"
CONFIG_NAME="c.1"
FUNCTION_NAME="midi.usb0"
USB_LANG="0x409"
MIDI_IN_PORTS="2"
MIDI_OUT_PORTS="2"

log() {
    printf '[mpc-midi-gadget] %s\n' "$*"
}

warn() {
    printf '[mpc-midi-gadget] WARNING: %s\n' "$*" >&2
}

die() {
    printf '[mpc-midi-gadget] ERROR: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'USAGE'
Usage:
  setup-mpc-midi-gadget.sh [--setup]
  setup-mpc-midi-gadget.sh --teardown
  setup-mpc-midi-gadget.sh --status

Creates a configfs USB MIDI gadget named mpc_midi for:
  MPC One+ USB-A -> Orange Pi Zero 2W USB0 / OTG / device USB-C

USB1 / host USB-C is not touched by this script and remains available for
class-compliant USB MIDI controllers/interfaces.
USAGE
}

require_root() {
    if [[ "${EUID}" -ne 0 ]]; then
        die "Run as root or via sudo."
    fi
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

load_module_if_available() {
    local module="$1"

    if ! command_exists modprobe; then
        warn "modprobe not found; assuming required kernel support is built in."
        return 0
    fi

    if modprobe "$module" >/dev/null 2>&1; then
        log "Loaded kernel module: ${module}"
        return 0
    fi

    # Many distro kernels build these pieces in, or omit module metadata for
    # built-in functions. Treat this as informational and let configfs checks
    # below decide whether support really exists.
    log "Kernel module not loadable or already built in: ${module}"
}

mount_configfs() {
    if [[ ! -d "${CONFIGFS_DIR}" ]]; then
        die "${CONFIGFS_DIR} does not exist; this kernel may not support configfs."
    fi

    if grep -qsE "[[:space:]]${CONFIGFS_DIR}[[:space:]]+configfs[[:space:]]" /proc/mounts; then
        log "configfs is already mounted at ${CONFIGFS_DIR}"
        return 0
    fi

    log "Mounting configfs at ${CONFIGFS_DIR}"
    mount -t configfs none "${CONFIGFS_DIR}" || die "Failed to mount configfs at ${CONFIGFS_DIR}."
}

verify_gadget_support() {
    if [[ ! -d "${CONFIGFS_DIR}/usb_gadget" ]]; then
        cat >&2 <<EOF
[mpc-midi-gadget] ERROR: ${CONFIGFS_DIR}/usb_gadget is missing.
[mpc-midi-gadget] This is not a gadget setup script problem. The running kernel
[mpc-midi-gadget] is not exposing libcomposite/configfs USB gadget support.
EOF
        exit 1
    fi
}

first_udc() {
    local entry
    shopt -s nullglob
    for entry in /sys/class/udc/*; do
        [[ -e "${entry}" ]] || continue
        basename "${entry}"
        shopt -u nullglob
        return 0
    done
    shopt -u nullglob
    return 1
}

require_udc() {
    local udc
    if ! udc="$(first_udc)"; then
        cat >&2 <<'EOF'
[mpc-midi-gadget] ERROR: /sys/class/udc is empty.
[mpc-midi-gadget] This is not a script problem. The kernel/device tree is not
[mpc-midi-gadget] exposing a USB device controller.
[mpc-midi-gadget]
[mpc-midi-gadget] Orange Pi Zero 2W notes:
[mpc-midi-gadget] - Use the USB0 / OTG / device-capable USB-C port to the MPC One+.
[mpc-midi-gadget] - USB1 is the host-only/peripheral host port and should remain for controllers.
[mpc-midi-gadget] - If USB0 is forced to host mode, missing dr_mode/peripheral role-switch
[mpc-midi-gadget]   device-tree support is the likely problem.
EOF
        exit 1
    fi

    printf '%s\n' "${udc}"
}

stable_serial() {
    local raw=""

    if [[ -r /etc/machine-id ]]; then
        raw="$(tr -cd '[:alnum:]' </etc/machine-id | head -c 24 || true)"
    fi

    if [[ -z "${raw}" && -r /proc/device-tree/serial-number ]]; then
        raw="$(tr -d '\000' </proc/device-tree/serial-number | tr -cd '[:alnum:]' | head -c 24 || true)"
    fi

    if [[ -n "${raw}" ]]; then
        printf 'opiz2w-%s\n' "${raw}"
    else
        printf 'opiz2w-trekr-midi\n'
    fi
}

write_attr() {
    local path="$1"
    local value="$2"

    [[ -e "${path}" ]] || die "Missing required configfs attribute: ${path}"
    printf '%s' "${value}" >"${path}" || die "Failed writing ${path}"
}

write_attr_if_present() {
    local path="$1"
    local value="$2"

    if [[ -e "${path}" ]]; then
        printf '%s' "${value}" >"${path}" || die "Failed writing ${path}"
    fi
}

unbind_gadget_if_bound() {
    if [[ -e "${GADGET_DIR}/UDC" ]]; then
        local bound
        bound="$(cat "${GADGET_DIR}/UDC" || true)"
        if [[ -n "${bound}" ]]; then
            log "Unbinding existing gadget from UDC: ${bound}"
            printf '' >"${GADGET_DIR}/UDC"
        fi
    fi
}

remove_symlink_if_present() {
    local path="$1"

    if [[ -L "${path}" ]]; then
        rm -f "${path}"
    elif [[ -e "${path}" ]]; then
        die "Expected symlink but found non-symlink: ${path}"
    fi
}

teardown_gadget() {
    if [[ ! -d "${GADGET_DIR}" ]]; then
        log "No existing gadget to tear down: ${GADGET_DIR}"
        return 0
    fi

    unbind_gadget_if_bound

    remove_symlink_if_present "${GADGET_DIR}/configs/${CONFIG_NAME}/${FUNCTION_NAME}"

    rmdir "${GADGET_DIR}/functions/${FUNCTION_NAME}" 2>/dev/null || true
    rmdir "${GADGET_DIR}/configs/${CONFIG_NAME}/strings/${USB_LANG}" 2>/dev/null || true
    rmdir "${GADGET_DIR}/configs/${CONFIG_NAME}" 2>/dev/null || true
    rmdir "${GADGET_DIR}/strings/${USB_LANG}" 2>/dev/null || true
    rmdir "${GADGET_DIR}" 2>/dev/null || die "Could not remove ${GADGET_DIR}; inspect remaining configfs entries."

    log "Removed gadget: ${GADGET_NAME}"
}

print_status() {
    log "UDCs:"
    if compgen -G "/sys/class/udc/*" >/dev/null; then
        ls -1 /sys/class/udc
    else
        printf '  (none)\n'
    fi

    if [[ -d "${GADGET_DIR}" ]]; then
        log "Gadget exists: ${GADGET_DIR}"
        printf '  UDC: %s\n' "$(cat "${GADGET_DIR}/UDC" 2>/dev/null || true)"
        printf '  Product: %s\n' "$(cat "${GADGET_DIR}/strings/${USB_LANG}/product" 2>/dev/null || true)"
        printf '  Manufacturer: %s\n' "$(cat "${GADGET_DIR}/strings/${USB_LANG}/manufacturer" 2>/dev/null || true)"
        printf '  Serial: %s\n' "$(cat "${GADGET_DIR}/strings/${USB_LANG}/serialnumber" 2>/dev/null || true)"
    else
        log "Gadget does not exist: ${GADGET_DIR}"
    fi
}

create_gadget() {
    local udc="$1"
    local serial
    serial="$(stable_serial)"

    mkdir -p "${GADGET_DIR}"

    write_attr "${GADGET_DIR}/idVendor" "0x1d6b"
    write_attr "${GADGET_DIR}/idProduct" "0x0104"
    write_attr "${GADGET_DIR}/bcdDevice" "0x0100"
    write_attr "${GADGET_DIR}/bcdUSB" "0x0200"
    write_attr_if_present "${GADGET_DIR}/bDeviceClass" "0x00"
    write_attr_if_present "${GADGET_DIR}/bDeviceSubClass" "0x00"
    write_attr_if_present "${GADGET_DIR}/bDeviceProtocol" "0x00"

    mkdir -p "${GADGET_DIR}/strings/${USB_LANG}"
    write_attr "${GADGET_DIR}/strings/${USB_LANG}/manufacturer" "domj"
    write_attr "${GADGET_DIR}/strings/${USB_LANG}/product" "Trekr"
    write_attr "${GADGET_DIR}/strings/${USB_LANG}/serialnumber" "${serial}"

    mkdir -p "${GADGET_DIR}/configs/${CONFIG_NAME}/strings/${USB_LANG}"
    write_attr "${GADGET_DIR}/configs/${CONFIG_NAME}/strings/${USB_LANG}/configuration" "MIDI"
    write_attr_if_present "${GADGET_DIR}/configs/${CONFIG_NAME}/MaxPower" "100"
    write_attr_if_present "${GADGET_DIR}/configs/${CONFIG_NAME}/bmAttributes" "0x80"

    mkdir -p "${GADGET_DIR}/functions/${FUNCTION_NAME}" || die "Failed to create MIDI function. Is usb_f_midi/configfs support enabled?"

    # Configfs MIDI port direction is named from the USB link perspective and
    # ALSA client labels can look inverted. Use both directions as a pair:
    # - MIDI sent from the MPC host to the gadget is readable by local ALSA apps.
    # - MIDI written by local ALSA apps is sent back to the MPC host.
    write_attr "${GADGET_DIR}/functions/${FUNCTION_NAME}/in_ports" "${MIDI_IN_PORTS}"
    write_attr "${GADGET_DIR}/functions/${FUNCTION_NAME}/out_ports" "${MIDI_OUT_PORTS}"
    write_attr_if_present "${GADGET_DIR}/functions/${FUNCTION_NAME}/id" "Trekr"
    write_attr_if_present "${GADGET_DIR}/functions/${FUNCTION_NAME}/qlen" "32"
    write_attr_if_present "${GADGET_DIR}/functions/${FUNCTION_NAME}/buflen" "512"

    if [[ ! -L "${GADGET_DIR}/configs/${CONFIG_NAME}/${FUNCTION_NAME}" ]]; then
        (
            cd "${GADGET_DIR}"
            ln -s "functions/${FUNCTION_NAME}" "configs/${CONFIG_NAME}/"
        )
    fi

    log "Binding ${GADGET_NAME} to UDC: ${udc}"
    write_attr "${GADGET_DIR}/UDC" "${udc}"
}

setup_gadget() {
    local udc
    udc="$(require_udc)"

    teardown_gadget

    local partial=1
    cleanup_partial() {
        if [[ "${partial}" -eq 1 ]]; then
            warn "Setup failed; removing partial gadget config."
            teardown_gadget || true
        fi
    }
    trap cleanup_partial ERR

    create_gadget "${udc}"
    partial=0
    trap - ERR

    log "USB MIDI gadget is active."
    print_status

    cat <<'EOF'

Verification on the Orange Pi:
  ls /sys/class/udc
  aconnect -l
  amidi -l
  aseqdump -l

Basic MIDI inspection examples:
  aseqdump -p '<client:port from aconnect -l>'
  amidi -l

Host-side check:
  On the MPC One+, look for a class-compliant USB MIDI device named
  "Trekr". From a Linux host, `lsusb` should show a composite
  gadget and ALSA should expose MIDI ports.

If /sys/class/udc is empty, fix kernel/device-tree USB0 OTG/device role support.
USB1 should remain in host mode for USB MIDI controllers/interfaces.
EOF
}

main() {
    local mode="${1:---setup}"

    case "${mode}" in
        --setup)
            ;;
        --teardown|--status|--help|-h)
            ;;
        *)
            usage >&2
            exit 2
            ;;
    esac

    if [[ "${mode}" == "--help" || "${mode}" == "-h" ]]; then
        usage
        exit 0
    fi

    require_root

    load_module_if_available libcomposite
    load_module_if_available usb_f_midi
    load_module_if_available snd_rawmidi
    load_module_if_available snd_seq
    load_module_if_available snd_seq_midi
    mount_configfs
    verify_gadget_support

    case "${mode}" in
        --setup)
            setup_gadget
            ;;
        --teardown)
            teardown_gadget
            ;;
        --status)
            print_status
            ;;
    esac
}

main "$@"
