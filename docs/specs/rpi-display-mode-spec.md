# Raspberry Pi Display Mode Spec

## Scope

This spec defines how the Pi KMSDRM launch path should choose a display resolution for `trekr`.

The immediate target is small HDMI/MIPI panels such as `1024x600` displays running the deployed launcher:

```bash
./launch-rpi-zero-2w.sh
```

## Problem

The app historically created its SDL window at `1280x720` before requesting KMSDRM fullscreen. On SDL's KMSDRM backend, that initial window size can influence the DRM mode selected for the CRTC. On native `1024x600` panels this can switch the output to 720p or cause unwanted scaling instead of rendering at the panel's native mode.

## Desired Behavior

### 1. Native/Current Display Mode

When `--video-mode kmsdrm-console` is active, `trekr` should query SDL's primary display mode before creating the window and use that mode's width and height as the initial window size.

Acceptance criteria:

- KMSDRM launch no longer hard-codes `1280x720` as the requested fullscreen size.
- The chosen size is logged at startup.
- If SDL cannot report a valid display mode, the app falls back to `1280x720` with a diagnostic message.
- Desktop/windowed behavior keeps the existing `1280x720` default window size.

### 2. Explicit Size Override

KMSDRM launch should support an explicit environment override:

```bash
TREKR_KMSDRM_SIZE=1024x600 ./launch-rpi-zero-2w.sh
```

Acceptance criteria:

- The override accepts `WIDTHxHEIGHT` and `WIDTHXHEIGHT`.
- Width and height must be positive integers.
- Invalid overrides are ignored and the native/current display-mode query remains the fallback path.
- The selected override size is logged at startup.

## Research Needed

### 3. Desktop Fullscreen / No Mode Switch

Investigate whether SDL3's Rust bindings expose a KMSDRM-safe desktop-fullscreen mode that preserves the currently configured CRTC mode without requesting an exclusive fullscreen mode.

Questions:

- Does SDL3 support a `SDL_WINDOW_FULLSCREEN_DESKTOP` equivalent in the current C API, or was this behavior folded into `SDL_SetWindowFullscreen`/fullscreen window flags?
- Does the `sdl3` Rust crate expose that mode directly, or only `WindowBuilder::fullscreen()`?
- On KMSDRM, does a borderless fullscreen window at the current display size avoid a DRM mode switch consistently?
- If Rust bindings lack the needed helper, is using `Window::set_display_mode(None)` or a raw SDL call appropriate?

Research outcome should decide whether KMSDRM launch should eventually prefer "preserve current mode" over "create window at detected native mode."
