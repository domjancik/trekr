# Raspberry Pi KMSDRM Window Size Extraction Handoff

## Goal

Extract the Raspberry Pi KMSDRM window-size fix into its own branch and PR.

The intended PR should make Trekr use the native KMSDRM display mode, such as `1024x600`, instead of starting the Pi console path with the desktop fallback size `1280x720`.

## Problem

The current mainline Pi/KMSDRM startup path creates the SDL window with a hardcoded `1280x720` size before entering fullscreen/borderless KMSDRM mode.

On small HDMI panels, this can make Trekr appear to switch the display path to a `720p`-shaped mode rather than respecting the panel's native mode, for example `1024x600`.

## Desired Behavior

For `--video-mode kmsdrm-console`:

- query SDL's current primary display mode before window creation
- create the window with that detected width/height
- allow an explicit environment override via `TREKR_KMSDRM_SIZE=WIDTHxHEIGHT`
- preserve the previous `1280x720` default for non-KMSDRM/windowed modes
- fall back to `1280x720` only if KMSDRM mode detection fails and no valid override is provided

## Changes To Extract

### Required Code

Extract the `src/app/mod.rs` changes that:

- replace the hardcoded `video.window("trekr", 1280, 720)` call with:

```rust
let (window_width, window_height) = initial_window_size(&video, options.video_mode);
let mut window_builder = video.window("trekr", window_width, window_height);
```

- add helper functions:

```rust
fn initial_window_size(video: &sdl3::VideoSubsystem, video_mode: VideoMode) -> (u32, u32)
fn parse_kmsdrm_size_override() -> Option<(u32, u32)>
fn parse_window_size(value: &str) -> Option<(u32, u32)>
```

- implement the KMSDRM lookup behavior:
  - first use `TREKR_KMSDRM_SIZE` if valid
  - otherwise call `video.get_primary_display().and_then(|display| display.get_mode())`
  - otherwise log a fallback message and return `(1280, 720)`
- keep non-KMSDRM modes returning `(1280, 720)`
- include focused tests for `parse_window_size`, including:
  - accepts `1024x600`
  - accepts whitespace and uppercase `X`
  - rejects missing separator
  - rejects zero dimensions
  - rejects non-numeric dimensions

### Required Docs

Extract the README note in the Pi launch section that documents:

- KMSDRM now queries SDL's current primary display mode
- native panels such as `1024x600` should not be forced through `1280x720`
- override example:

```bash
TREKR_KMSDRM_SIZE=1024x600 ./launch-rpi-zero-2w.sh
```

Extract or create the display spec:

- `docs/specs/rpi-display-mode-spec.md`

It should describe:

- current implemented behavior for KMSDRM native-size detection
- `TREKR_KMSDRM_SIZE=WIDTHxHEIGHT`
- fallback behavior
- deferred research for desktop fullscreen behavior and platform-specific mode switching

Update `docs/README.md` only enough to link the display spec, if the target branch does not already have that link.

## Changes Not To Include

Do not include unrelated session changes:

- Bookworm Docker build/deploy path
- Armbian microSD configuration script
- MIDI loopback filtering spec
- MIDI runtime decoupling specs
- deploy script SSH exit-code fix
- Magic Keyboard `hid_apple` device setup
- untracked local state files such as `beepgood.json`

The branch/PR should be focused on the KMSDRM window-size fix only.

## Suggested Branch

```bash
git switch main
git pull --ff-only
git switch -c fix/rpi-kmsdrm-native-window-size
```

If extracting from a dirty working tree, prefer applying only the relevant hunks rather than committing every modified file.

Useful review commands:

```bash
git diff -- src/app/mod.rs README.md docs/README.md docs/specs/rpi-display-mode-spec.md
git status --short
```

## Validation

Run:

```bash
cargo fmt
cargo check
cargo test parse_window_size
```

Optional Pi validation:

```bash
TREKR_KMSDRM_SIZE=1024x600 ./launch-rpi-zero-2w.sh
```

Then test without the override and confirm the app logs the detected KMSDRM display mode.

## PR Description Outline

Use this structure:

```text
Summary
- Use the current KMSDRM display mode as the initial Pi console window size.
- Add TREKR_KMSDRM_SIZE=WIDTHxHEIGHT override for panels that SDL reports incorrectly.
- Document fallback and override behavior.

Validation
- cargo fmt
- cargo check
- cargo test parse_window_size
```

## Notes For Review

This change is expected to affect only startup sizing on the KMSDRM console path. It should not materially change desktop/windowed rendering and should not require screenshot artifact refresh.
