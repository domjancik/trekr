# Trekr MIDI Gadget

This directory is a self-contained Orange Pi Zero 2W USB MIDI gadget deployment bundle for using the board as a USB MIDI sidecar for an Akai MPC One+.

Cable topology:

```text
MPC One+ USB-A -> Orange Pi Zero 2W USB0 as MIDI gadget
Orange Pi Zero 2W USB1 -> optional USB MIDI controller/interface
```

`setup-mpc-midi-gadget.sh` configures the Orange Pi Zero 2W USB0 / OTG / device-capable USB-C port as a class-compliant USB MIDI gadget named `Trekr` using the Linux libcomposite/configfs stack. It creates one bidirectional MIDI function with two input ports and two output ports, so the MPC can send MIDI to local ALSA clients and local apps can send MIDI back to the MPC on separate logical ports.

USB1 is not touched and should remain available as a normal host port for class-compliant USB MIDI controllers or interfaces.

## Deploy

Copy this directory to the Orange Pi:

```powershell
ssh <user>@<orange-pi-host> "mkdir -p ~/trekr-midi-gadget"
scp .\deploy\trekr-midi-gadget\* <user>@<orange-pi-host>:~/trekr-midi-gadget/
ssh <user>@<orange-pi-host> "chmod +x ~/trekr-midi-gadget/*.sh"
```

## Run Once

Run for this boot only, without enabling autostart:

```bash
cd ~/trekr-midi-gadget
./trekr-midi-gadget.sh run
```

Stop and remove the current gadget:

```bash
cd ~/trekr-midi-gadget
./trekr-midi-gadget.sh stop
```

## Boot Service

Enable boot-time setup after the manual test works:

```bash
cd ~/trekr-midi-gadget
./trekr-midi-gadget.sh enable-boot
```

Disable boot-time setup and tear down the current gadget:

```bash
cd ~/trekr-midi-gadget
./trekr-midi-gadget.sh disable-boot
```

Inspect service logs:

```bash
journalctl -u mpc-midi-gadget.service -b
```

Remove the installed service and setup script entirely:

```bash
cd ~/trekr-midi-gadget
./trekr-midi-gadget.sh uninstall
```

## Verify

Verification commands on the Orange Pi:

```bash
ls /sys/class/udc
aconnect -l
amidi -l
aseqdump -l
```

Basic MIDI inspection examples:

```bash
aseqdump -p '<client:port from aconnect -l>'
amidi -l
```

From the MPC or another USB host, look for a class-compliant USB MIDI device named `Trekr`. From a Linux host, `lsusb` should show a composite gadget and ALSA should expose MIDI ports.

Expected Orange Pi ALSA shape for the two-port gadget is one card/client with two logical ports, for example:

```text
client 28: 'f_midi' [type=kernel,card=3]
    0 'f_midi-0'
    1 'f_midi-1'

amidi -l:
IO  hw:3,0    f_midi (2 subdevices)
```

The USB product and ALSA card id are `Trekr`, but this kernel still exposes the raw MIDI port label as `f_midi`.

## Orange Pi Notes

If `/sys/class/udc` is empty, this is not a script problem: the running kernel/device tree is not exposing USB device controller support.

On Orange Pi Zero 2W/H618:

- connect the MPC to USB0, not USB1
- USB0 must be in OTG/peripheral role
- USB1 should remain host for USB MIDI controllers or interfaces
- USB-C port roles are board-specific and the two ports are not equivalent
