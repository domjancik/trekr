# trekr

Native MIDI-first tracker/player/looper for small PCs with a portable path to mobile-class targets.

## Screenshots

Latest renderer-owned captures from the demo state:

### Timeline

![Timeline](artifacts/screenshots/timeline.png)

### Mappings

![Mappings](artifacts/screenshots/mappings.png)

### Mappings Overlay

![Mappings Overlay](artifacts/screenshots/mappings-overlay.png)

### MIDI I/O

![MIDI I/O](artifacts/screenshots/midi-io.png)

### Routing

![Routing](artifacts/screenshots/routing.png)

## Docs

- `docs/product-spec.md`: product behavior, UX model, workflows, and MVP scope.
- `docs/feature-spec-midi-manipulation.md`: action-driven MIDI note selection and editing behavior.
- `docs/architecture.md`: engine architecture, portability constraints, and stack options.
- `docs/implementation-plan.md`: milestone order, module breakdown, and delivery sequence.
- `docs/current-mappings.md`: current keyboard bindings and prototype MIDI/OSC mapping overview.

## Current Direction

- Primary target: small-form-factor desktop systems.
- Secondary target: iOS/Android if the chosen stack supports it cleanly.
- V1 focus: MIDI sequencing, routing, passthrough, and loop-based recording.
- Audio follows MIDI-first V1 and should layer onto the same timeline and routing model later.
- Chosen implementation stack: Rust with a lightweight native rendering and I/O stack.
- SDL3 is built from source in the current scaffold so local builds do not depend on a preinstalled SDL runtime.

## Current Runnable Slice

`cargo run` opens a native SDL3 window with:

- fixed-fit per-track paired columns in the form `full | detail | full | detail`
- default vertical-time layout with time moving downward
- a page shell for `Timeline`, `Mappings`, `MIDI I/O`, and `Routing`
- real MIDI device enumeration via `midir`
- basic routed MIDI note playback on track output ports/channels
- MIDI output runs on a dedicated worker thread so device stalls or hot-plug churn do not block the UI thread
- in-canvas bitmap text labels for pages, tracks, ports, mappings, and routing values
- active-track highlighting
- a moving playhead
- per-track loop preview
- an in-canvas transport strip on the timeline page
- a renderer-level footer/status bar that shows hover mapping summaries and falls back to the last performed action
- hover-driven mapping discoverability for timeline transport, track-state controls, and routing passthrough controls
- an inline mapping discoverability overlay with compact built-in vs user-defined badges
- a field-based mappings editor with MIDI learn for MIDI sources
- a cross-platform Ableton Link transport layer with runtime status in the transport strip
- direct mouse/touch control for tabs, transport controls, mappings, MIDI I/O selection, and routing fields

Launch state:

- `cargo run -- help` prints the CLI reference, option details, and suggested commands
- `cargo run -- commands` prints the recommended documented launch commands only
- `cargo run -- run` explicitly launches the interactive app
- default interactive run uses persisted state from `artifacts/state/last-run.json` when available and saves back on clean exit
- `cargo run -- --state-mode demo` forces the built-in demo state
- `cargo run -- --state-mode empty` forces an empty deterministic state
- `cargo run -- run --state-mode demo` is the subcommand form of the same demo launch
- `cargo run -- --state-file path\\to\\state.json` uses a specific persisted state path
- `cargo run -- --video-mode windowed` keeps the existing resizable desktop window behavior
- `cargo run -- --video-mode fullscreen` requests fullscreen rendering on the active SDL video backend
- `cargo run -- --video-mode kmsdrm-console` requests SDL's `kmsdrm` backend for direct fullscreen rendering from a Linux console session without X11/Wayland
- `cargo run -- capture-ui --state-mode demo --capture-dir artifacts/screenshots` renders deterministic screenshots without opening the interactive app
- `cargo run -- --ui-scale 2.0` forces a larger logical UI scale instead of using the OS-reported display scale
- `cargo run --bin trekr-tui` opens a terminal menu for selecting launch mode, state, video mode, scale, and capture path
- committed fixture state lives in `state-fixtures/ui-looped.json`

CLI notes:

- `run`, `capture-ui`, `commands`, and `help` are the first-class app commands
- the older flag-only form is still supported for compatibility, so existing commands like `cargo run -- --state-mode demo` still work
- `capture-ui` accepts launch-state options plus `--capture-dir`; `--video-mode` remains interactive-only

Pi console launch on-device:

```bash
./launch-rpi-zero-2w.sh
```

This wrapper starts `trekr` with `--video-mode kmsdrm-console` for a minimal Raspberry Pi console session.
It pins `SDL_VIDEODRIVER=kmsdrm`, `SDL_KMSDRM_REQUIRE_DRM_MASTER=1`, `SDL_KMSDRM_ATOMIC=0`, GLES loader hints, and `LD_LIBRARY_PATH` so the deployed binary uses the shipped SDL runtime and a Pi-oriented KMSDRM launch path.
It prefers `SDL_RENDER_DRIVER=opengles2` and you can override that to `software` only if the Pi image cannot initialize GLES.

Current working KMSDRM init path:

- build with the standard SDL path via `powershell -ExecutionPolicy Bypass -File .\scripts\build-rpi-zero-2w.ps1 -Release`
- deploy with `powershell -ExecutionPolicy Bypass -File .\scripts\deploy-rpi-zero-2w.ps1`
- launch from a Linux virtual console, not from X11 or Wayland, via `./launch-rpi-zero-2w.sh`
- let the launcher provide `SDL_VIDEODRIVER=kmsdrm`, `SDL_KMSDRM_REQUIRE_DRM_MASTER=1`, `SDL_KMSDRM_ATOMIC=0`, `SDL_RENDER_DRIVER=opengles2`, `SDL_EGL_LIBRARY=libEGL.so.1`, `SDL_OPENGL_LIBRARY=libGLESv2.so.2`, and `LD_LIBRARY_PATH`
- `trekr` then sets the SDL KMSDRM hints, creates a fullscreen borderless window, calls `window.sync()`, and uses the renderer-backed KMSDRM loop by default
- keep `TREKR_KMSDRM_PRESENT_MODE=surface` only as a diagnostic fallback when the renderer path is not usable on a given Pi image

Bootstrap and run:

- fresh clones can use `cargo xtask run` as the single setup-and-run command
- `cargo xtask setup` initializes the `vendor/ableton-link` git submodule
- `cargo xtask run-demo` and `cargo xtask run-empty` do the same for the demo and empty launch modes
- `cargo xtask run -- --ui-scale 2.0` forwards extra app flags after `--`
- `cargo xtask check` initializes the submodule if needed, then runs `cargo check`
- the Cargo alias lives in `.cargo/config.toml`, so no extra task runner install is required

Current controls:

- `Tab` / `Shift+Tab`: next/previous page
- `F1` / `F2` / `F3` / `F4`: show timeline, mappings, MIDI I/O, or routing page
- `F5`: toggle the quick mappings overlay from any page
- `F7`: toggle the inline mapping discoverability overlay from any page
- `F6`: toggle Ableton Link participation
- `Shift+F6`: toggle Ableton Link start/stop sync participation
- `Up` / `Down`: select current page item
- `Shift+Left` / `Shift+Right`: select current editable field on the mappings page in write mode
- `Q` / `E`: adjust current page item
- `Enter`: activate/toggle current page item
- `W`: toggle mappings page mode between read-only overview and write mode
- `N`: add a mapping row on the mappings page in write mode
- `Delete`: remove the selected mapping row on the mappings page in write mode
- `Space`: play/stop
- `R`: start/stop recording on armed tracks, or the active track if none are armed
- `Shift+R`: cycle recording mode between `Overdub` and `Replace`
- `C`: clear current track notes/regions and cancel its pending take
- `Shift+C`: clear all track notes/regions and cancel pending takes
- `Home`: reset the global song loop to the full song range
- `[` / `]`: set current-track loop start/end at playhead
- `,` / `.`: nudge current-track loop backward/forward by one quantize step
- `-` / `=`: shorten/extend current-track loop by one quantize step
- `/` / `\`: half/double current-track loop length
- `Shift+[` / `Shift+]`: set global loop start/end at playhead
- `Shift+,` / `Shift+.`: nudge global loop backward/forward by one quantize step
- `Shift+-` / `Shift+=`: shorten/extend global loop by one quantize step
- `Shift+/` / `Shift+\`: half/double global loop length
- `G`: toggle global loop enable
- `L`: toggle current track loop enable
- `A`: arm current track
- `M`: mute current track
- `S`: solo current track
- `I`: toggle current track passthrough
- `Left` / `Right`: select previous/next track directly
- `1`-`9`: select track by absolute index
- `Escape`: quit

The timeline page also exposes a clickable `Reset Song Loop` button that triggers the same action as `Home`.

Mapping discoverability notes:

- hovering supported action elements now uses the in-app footer as the primary mapping status surface
- the footer falls back to the last performed action when nothing discoverable is hovered
- `F7` enables a separate discoverability overlay with compact inline badges
- discoverability badges use different colors for built-in keyboard bindings vs enabled user-defined mappings
- disabled mappings are hidden from the footer and discoverability overlay
- track-column discoverability is active-track scoped in V1, even when hovering non-active columns

Pointer/touch notes:

- tabs are clickable/tappable
- timeline transport chips are clickable/tappable for play, record, record mode, loop-wrap clip extension (`RecWrap Clamp` / `RecWrap Extend`), song loop, Link, and Link sync
- mappings rows and fields are clickable/tappable; in `Write` mode, tapping the selected field activates it
- MIDI I/O rows are clickable/tappable to select and set the default input/output
- routing rows are clickable/tappable; tapping the value area adjusts the field and tapping passthrough toggles it
- timeline note and region editing is still not implemented for pointer/touch input

Recording flow notes:

- armed tracks are the first recording targets; if none are armed, recording uses the active track
- stopping playback while recording commits the active take instead of discarding it
- `RecWrap Extend` is the default and keeps a looped recording going past the loop boundary by rebasing the clip to loop start and extending its length instead of clamping the take at the loop end
- the timeline shows committed regions behind notes and shows the in-progress take as a red preview region
- MIDI note content now comes from live input note-on/off events on each track's routed MIDI input, not a generated placeholder pattern

The `Mappings` page now supports two modes:

- `Read Only`: compact overview
- `Write`: field-based editing for source type, source device, source value, target, scope, and enabled state
- `Write` mode also supports adding/removing rows and cycling track-scoped mappings into concrete `Track 1`, `Track 2`, ... scopes

MIDI learn notes:

- in mappings `Write` mode, move to the `Source` field and press `Enter` to arm MIDI learn for the selected row
- the next incoming MIDI note or CC updates that mapping source and exits learn mode
- learned MIDI mappings store the device name of the input that triggered learn
- live MIDI input now resolves against enabled mappings and can trigger app actions from either `Any MIDI` or a specific device
- `Shift+Left` / `Shift+Right` moves between editable mapping fields

Ableton Link notes:

- Ableton Link now uses the official Ableton source from the `vendor/ableton-link` git submodule through a small native bridge, instead of the broken third-party Rust wrapper
- `cargo xtask run` will initialize that submodule automatically on first run, or you can run `cargo xtask setup` explicitly
- the transport strip shows Link enabled state, start/stop sync state, and peer count/status summary

The app also exposes a generic overlay layer with two independent modes:

- `F5`: quick mappings overlay
- `F7`: inline mapping discoverability overlay

Current planning note:

- the remaining MVP checklist now lives in `docs/implementation-plan.md`
- Ableton Link is planned as a near-term sync milestone after the core MVP workflow is comfortable, and its architecture notes live in `docs/architecture.md`

## Raspberry Pi Zero 2 W Cross-Build

The Raspberry Pi Zero 2 W is a Linux `aarch64` target, so the repo cross-build path is:

- target triple: `aarch64-unknown-linux-gnu`
- host flow: run the build inside WSL from Windows, rather than trying to drive a Linux linker from the Windows Rust toolchain

Repo entrypoint:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-rpi-zero-2w.ps1 -Release
```

Pi console / KMSDRM entrypoint:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-rpi-zero-2w.ps1 -Release -SdlUnixConsoleBuild
```

Recommended deployed Pi build:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-rpi-zero-2w.ps1 -Release
```

SSH deployment entrypoint:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\deploy-rpi-zero-2w.ps1
```

Pi runtime package setup:

```bash
sudo ./setup-rpi-zero-2w-runtime.sh
```

Expected artifact:

```text
target\aarch64-unknown-linux-gnu\release\trekr
```

WSL prerequisites:

- a working WSL distro with Rust installed inside that distro
- the Rust target installed inside WSL: `rustup target add aarch64-unknown-linux-gnu`
- Debian/Ubuntu package names for the Linux-side cross toolchain:
  - `gcc-aarch64-linux-gnu`
  - `g++-aarch64-linux-gnu`
  - `binutils-aarch64-linux-gnu`
  - `cmake`
  - `ninja-build`
  - `pkg-config`

Example setup inside WSL:

```bash
rustup target add aarch64-unknown-linux-gnu
sudo apt update
sudo apt install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu binutils-aarch64-linux-gnu cmake ninja-build pkg-config
```

Notes:

- `scripts/build-rpi-zero-2w.ps1` fails fast if WSL is unavailable or the required Linux-side toolchain is missing.
- the normal deployed Pi path should use the standard SDL build. `-SdlUnixConsoleBuild` is retained as a diagnostic/experimental option, but it is not the recommended default for the fullscreen KMSDRM app path.
- Linux MIDI support goes through ALSA via `midir`, so if the final link step reports missing ALSA target libraries, install the matching ARM64 ALSA development package in the WSL distro/sysroot before retrying.
- Runtime on a minimal Pi console is opt-in: launch the binary with `--video-mode kmsdrm-console` to force SDL onto the `kmsdrm` backend. Desktop targets should stay on the default `windowed` mode.
- the deployed Pi launcher currently prefers `SDL_RENDER_DRIVER=opengles2` and sets `SDL_KMSDRM_ATOMIC=0`, which is the first compatibility path to try on Raspberry Pi when KMSDRM presents a black screen.
- `scripts/deploy-rpi-zero-2w.ps1` reads untracked local SSH settings from `scripts/rpi-deploy.local.psd1`. Start from the committed example file at `scripts/rpi-deploy.example.psd1`.
- the deploy flow copies `trekr`, `libSDL3.so.0`, and `launch-rpi-zero-2w.sh` into the remote app directory so the Pi does not need a system-installed SDL3 runtime.
- `scripts/setup-rpi-zero-2w-runtime.sh` installs the minimal Pi runtime packages needed for SDL KMSDRM, EGL/GLES loader discovery, and ALSA on a console-first image.
- `scripts/deploy-rpi-zero-2w.ps1 -InstallRuntimeDeps` can run that package setup remotely. If the local deploy config has no `Password`, the remote user needs passwordless `sudo`; otherwise the configured password is passed to `sudo -S`.
- Leaving `Password` blank in the deploy config uses normal OpenSSH key or agent auth through `ssh.exe` and `scp.exe`.
- Setting `Password` in the deploy config is supported only when `plink.exe` and `pscp.exe` are available on `PATH`.
- this path targets Pi Zero 2 W. The original Pi Zero / Zero W is a 32-bit ARMv6 device and needs a different target strategy.

## UI Review Loop

The repo includes a scripted screenshot-and-review loop for visual QA:

- `scripts/capture-ui-screens.ps1`: asks `trekr` itself to render `timeline`, `mappings`, `midi-io`, and `routing` screenshots into `artifacts/screenshots`
  - capture explicitly uses `--state-mode demo` so screenshots stay deterministic instead of depending on the last persisted interactive state
- `scripts/review-ui-screens.ps1`: calls `codex exec` with those screenshots attached and writes findings to `artifacts/reviews/ui-findings.md`
- `scripts/run-ui-review.ps1`: runs both steps in sequence and archives the results under `artifacts/archive/<git-commit>/`

Tracked artifacts:

- `artifacts/screenshots/`: latest renderer-owned screenshots used by the README
- `artifacts/reviews/ui-findings.md`: latest compact screenshot review findings

Ignored artifacts:

- `artifacts/archive/`: commit-keyed review history
- `artifacts/state/`: last-run persisted state
- `docs/artifacts/` and `scripts/artifacts/`: stray/generated script-state directories

The capture path is renderer-owned rather than desktop-owned:

- screenshots are exported from the SDL drawing layer
- capture runs against an offscreen software surface, so other desktop apps do not leak into the images

Review process:

1. Run `powershell -ExecutionPolicy Bypass -File .\scripts\capture-ui-screens.ps1`
2. Check `artifacts/screenshots\manifest.json` for the exported page/image list
3. Run `powershell -ExecutionPolicy Bypass -File .\scripts\review-ui-screens.ps1`
4. Read `artifacts/reviews/ui-findings.md` for the latest Codex layout findings
5. Use `artifacts/archive/<git-commit>/screenshots` and `artifacts/archive/<git-commit>/reviews/ui-findings.md` for the commit-keyed snapshot

The review script passes the generated screenshots to `codex exec --image ...`, so the analysis step is based on the renderer-level captures rather than a live desktop screenshot.

Example:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-ui-review.ps1
```

Fixture examples:

```powershell
cargo run -- --state-mode persisted --state-file state-fixtures/ui-looped.json
powershell -ExecutionPolicy Bypass -File .\scripts\run-ui-review.ps1 -StateMode persisted -StateFile state-fixtures/ui-looped.json
```

## Pi Camera Debug Review

The repo also includes a separate physical-camera debug path for reviewing the deployed Pi's actual screen output from the development machine.

This flow is intended for a local capture device such as `Cam Link 4K` connected to the development machine, pointed at the Pi display. It does not run on the Pi itself.

Committed files:

- `scripts/capture-pi-output-camera.ps1`: captures one frame from a local DirectShow camera device into `artifacts/camera-debug`
- `scripts/capture-pi-output-camera-clip.ps1`: records a short local HDMI capture clip into `artifacts/camera-debug`
- `scripts/analyze-pi-output-camera-clip.ps1`: downsamples a captured clip and writes brightness/frame-diff metrics into `artifacts/camera-debug/clip-analysis`
- `scripts/review-pi-output-camera.ps1`: sends the captured image to `codex exec` for a compact diagnostic review
- `scripts/run-pi-output-camera-review.ps1`: runs capture and review together and archives the result under `artifacts/archive/<git-commit>/camera-debug`
- `scripts/pi-camera-debug.example.psd1`: example local camera config

Local setup:

- copy `scripts/pi-camera-debug.example.psd1` to `scripts/pi-camera-debug.local.psd1` if you want to override the default local capture device or format
- the local config is ignored by git
- the current default device name is `usb video`
- set `VideoCodec` for devices that expose compressed capture modes such as `mjpeg`; otherwise use `PixelFormat`
- if DirectShow is flaky with the friendly device name during active capture, set `DeviceInput` to the camera's alternative PnP selector from `-ListDevices`

Useful commands:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\capture-pi-output-camera.ps1 -ListDevices
powershell -ExecutionPolicy Bypass -File .\scripts\capture-pi-output-camera.ps1 -ListOptions
powershell -ExecutionPolicy Bypass -File .\scripts\capture-pi-output-camera.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\capture-pi-output-camera-clip.ps1 -DurationSeconds 10
powershell -ExecutionPolicy Bypass -File .\scripts\analyze-pi-output-camera-clip.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\review-pi-output-camera.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\run-pi-output-camera-review.ps1
```

Notes:

- this flow uses local `ffmpeg` DirectShow capture, not renderer-owned screenshots
- by default it also records a small remote Pi status snapshot into `artifacts/camera-debug/pi-status.txt` using `scripts/rpi-deploy.local.psd1` if that config exists
- if `ffmpeg` reports `Could not run graph`, the capture device is usually already in use by another app such as OBS
