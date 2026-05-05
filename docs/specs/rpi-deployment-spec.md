# Raspberry Pi Deployment Spec

## Scope

This spec defines the intended Raspberry Pi deployment workflow for `trekr`, especially the split between normal app deploys, first-time runtime provisioning, and boot-time service setup.

The current target is an Armbian-based Raspberry Pi or compatible ARM64 board running the console/KMSDRM build through `scripts/launch-rpi-zero-2w.sh`.

## Design Principles

- Normal deploys should stay fast, repeatable, and low-risk.
- Package installation and machine provisioning should be explicit, idempotent operations.
- Autostart should be managed through systemd, not shell profile hooks or manual terminal sessions.
- Scripts should be safe to rerun after reflashing the image.
- Scripts should support key-based SSH as the preferred path and keep password-based SSH as a fallback where existing tooling already supports it.

## Deployment Modes

### Normal Deploy

The default deploy command should build/copy/finalize only:

- build the ARM64 release artifact unless `-SkipBuild` is supplied
- copy `trekr`, `libSDL3.so.0`, `launch-rpi-zero-2w.sh`, and runtime helper scripts to the configured remote directory
- set executable bits on copied binaries/scripts

The default deploy command should not run `apt-get`, alter system services, reboot the device, or start long-running background services.

Rationale: normal iteration should not mutate the target operating system or block on package manager state.

### Build Baselines

The default WSL cross-build path may use the host distro's ARM64 glibc baseline. This is acceptable for fast local iteration, but it can produce artifacts that do not run on older target images.

The repo should also support a Debian Bookworm build path:

```powershell
.\scripts\build-rpi-zero-2w-bookworm.ps1 -Release
```

This path should build inside a Debian Bookworm container and place artifacts under:

```text
target/bookworm/aarch64-unknown-linux-gnu/release/
```

The Bookworm container build should default SDL to the console/KMSDRM configuration so it does not require X11 or Wayland development packages. Desktop SDL variants may be supported through an explicit opt-in switch, but they are not the default deployment target.

The Bookworm build is the compatibility baseline for Pi deployment because Debian 12 ships glibc 2.36. Artifacts built against this baseline should run on Bookworm and newer glibc images. Deploy should expose an explicit switch for that artifact set:

```powershell
.\scripts\deploy-rpi-zero-2w.ps1 -BookwormBuild
```

### Runtime Dependency Setup

Runtime package installation should remain explicit:

```powershell
.\scripts\deploy-rpi-zero-2w.ps1 -InstallRuntimeDeps
```

This mode should run `setup-rpi-zero-2w-runtime.sh` remotely with root privileges. The setup script must remain idempotent and suitable for first provisioning or post-reflash repair.

The setup script owns OS package dependencies needed by the KMSDRM console build, including SDL runtime dependencies such as DRM, GBM/EGL/GLES, input, udev, ALSA, and xkbcommon packages.

### Start After Deploy

Manual launch after deploy should remain explicit:

```powershell
.\scripts\deploy-rpi-zero-2w.ps1 -StartAfterDeploy
```

This is an operator convenience for immediate testing. It should run the launcher in the deployment directory, but it should not install or enable any persistent boot-time service.

## Autostart

`trekr` should support a systemd-based autostart path for appliance-style operation.

The preferred implementation is a separate idempotent remote helper, for example:

```text
scripts/install-rpi-zero-2w-service.sh
```

The deploy script should expose explicit service-management switches rather than enabling autostart by default. Candidate switches:

- `-InstallService`: copy/install the systemd unit and any service helper files
- `-EnableService`: enable the service for boot
- `-DisableService`: disable the service without deleting app files
- `-RestartService`: restart the service after deploying new artifacts

The systemd unit should:

- run as the configured deploy user unless root is strictly required
- set the same SDL/KMSDRM environment as `launch-rpi-zero-2w.sh`, preferably by invoking that launcher
- start after local filesystems and basic device/session readiness
- restart on failure with a bounded delay
- write logs to journald

The service installer should be safe to rerun and should not require a reboot to update the installed unit.

## First-Boot Image Preparation

MicroSD image preparation is separate from app deployment.

Armbian first-boot preseeding may configure Wi-Fi, locale, timezone, and final user/root credentials, but some images still require the initial factory login before consuming those presets. Scripts and docs must not promise that Armbian first-login automation always skips the initial `root / 1234` gate.

Any direct rootfs modification mode that bypasses first-login entirely should be clearly separate from deploy and should validate the mounted filesystem before writing.

## Acceptance Criteria

- Running `deploy-rpi-zero-2w.ps1` with no provisioning switches does not install packages or alter systemd service state.
- `-InstallRuntimeDeps` remains the explicit path for package installation.
- Autostart installation and enablement are explicit and idempotent.
- Deploying new binaries can be combined with `-RestartService` without reinstalling packages.
- The runtime setup script can be rerun after a reflash without failing on already-installed packages.
- Documentation distinguishes image first-boot preparation from application deployment.
