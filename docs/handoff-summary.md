# Handoff Summary

## Project

- Repo: `C:\Users\magne\dev\trekr`
- GitHub: `https://github.com/domjancik/trekr`
- Primary branch: `main`
- Current app: native Rust MIDI-first tracker/player/looper prototype with SDL3 UI, `midir` MIDI I/O, and Ableton Link bridge

## Current Product Shape

- Default timeline layout is vertical time with per-track paired columns:
  - `song | loop detail | song | loop detail ...`
- Pages implemented:
  - `Timeline`
  - `Mappings`
  - `MIDI I/O`
  - `Routing`
- Input model is action-driven:
  - keyboard
  - MIDI mappings
  - mouse/touch for non-timeline chrome
- Recording is MIDI-first:
  - live note-on/off capture
  - record preview
  - overdub/replace modes
  - loop-aware recording and playback

## Key Implemented Features

- MIDI device enumeration and routing
- MIDI playback to routed output ports/channels
- MIDI input capture, passthrough, and recording
- device-aware MIDI mappings that trigger app actions
- mapping editor with:
  - write mode
  - MIDI learn
  - add/remove row
  - absolute track scopes like `Track 1`, `Track 2`, ...
- Link integration through the official Ableton Link source in `vendor/ableton-link`
- persisted or deterministic startup state:
  - `demo`
  - `empty`
  - `persisted`
  - fixture file via `--state-file`
- renderer-owned screenshot capture and Codex screenshot review flow
- mouse/touch support for non-timeline controls:
  - tabs
  - transport strip
  - mappings page controls
  - MIDI I/O lists
  - routing fields

## Not Implemented

- timeline note editing
- timeline region editing
- drag gestures for loop/note editing
- audio track engine
- OSC learn/input path
- robust hot-plug refresh/reconnect UX
- final low-jitter engine timing path outside UI-frame polling

## Important Files

- Product/docs:
  - `README.md`
  - `docs/product-spec.md`
  - `docs/architecture.md`
  - `docs/implementation-plan.md`
  - `docs/current-mappings.md`
- Core code:
  - `src/app.rs`
  - `src/actions.rs`
  - `src/mapping.rs`
  - `src/midi_io.rs`
  - `src/project.rs`
  - `src/transport.rs`
  - `src/timeline.rs`
  - `src/pages.rs`
  - `src/ui.rs`
  - `src/link.rs`
- Native Link bridge:
  - `build.rs`
  - `native/link_bridge.cpp`
  - `native/link_bridge.hpp`
- Artifact/review flow:
  - `scripts/capture-ui-screens.ps1`
  - `scripts/review-ui-screens.ps1`
  - `scripts/run-ui-review.ps1`
  - `artifacts/screenshots/`
  - `artifacts/reviews/ui-findings.md`
- Repo maintenance rules:
  - `AGENTS.md`

## Current Controls

- Page/navigation:
  - `Tab` / `Shift+Tab`
  - `F1`-`F4`
  - `F5` mappings overlay
- Link:
  - `F6` toggle Link
  - `Shift+F6` toggle Link start/stop sync
- Mappings:
  - `W` write mode
  - `N` add mapping row
  - `Delete` remove mapping row
  - `Shift+Left` / `Shift+Right` field select
  - `Q` / `E` adjust
  - `Enter` activate / learn
- Transport/record:
  - `Space`
  - `R`
  - `Shift+R`
  - `Home`
  - `G`
  - `L`
- Loop editing:
  - `[` / `]`
  - `Shift+[` / `Shift+]`
  - `,` / `.`
  - `Shift+,` / `Shift+.`
  - `-` / `=`
  - `Shift+-` / `Shift+=`
  - `/` / `\`
  - `Shift+/` / `Shift+\`
- Track state:
  - `A`
  - `M`
  - `S`
  - `I`
  - `Left` / `Right`
  - `1`-`9`

## Current Verification Baseline

Recent completed checks before this handoff:

- `cargo test` passed with `102` tests
- `cargo run -- --capture-ui --capture-dir artifacts/screenshots --state-mode demo` passed
- latest renderer-owned screenshots exist in `artifacts/screenshots/`

## Screenshot/README Policy

- Latest tracked screenshots are kept in:
  - `artifacts/screenshots/timeline.png`
  - `artifacts/screenshots/mappings.png`
  - `artifacts/screenshots/mappings-overlay.png`
  - `artifacts/screenshots/midi-io.png`
  - `artifacts/screenshots/routing.png`
- `README.md` embeds those images
- `AGENTS.md` instructs future agents to keep those screenshots current when the main screens change

## Recent Relevant Commits

- `ed6a056` `feat: add mouse and touch controls for app chrome`
- `b7960b3` `feat: clarify pointer affordances in mappings and routing`
- `1f9b4d4` `fix: disable device field for non-midi mappings`
- `aad8be0` `fix: subscribe all midi inputs for mapping learn`
- `ee4b832` `feat: add mapping row editing and absolute track scopes`
- `807538e` `feat: trigger actions from device-aware midi mappings`
- `775bdde` `docs: track latest ui artifacts and refresh readme`
- `45ed4ce` `feat: add cross-platform ableton link bridge`

## Current Worktree State

At the time of writing this handoff, the worktree is dirty and includes changes not made as part of this handoff. Do not overwrite them blindly.

Observed modified/untracked items:

- modified:
  - `README.md`
  - `artifacts/reviews/ui-findings.md`
  - `scripts/deploy-rpi-zero-2w.ps1`
  - `scripts/launch-rpi-zero-2w.sh`
  - `src/app.rs`
- untracked:
  - `scripts/setup-rpi-zero-2w-runtime.sh`

Those should be reviewed before any reset, cleanup, or broad staging step.

## Highest-Value Next Steps

1. Tackle remaining timeline clarity issues from `artifacts/reviews/ui-findings.md`, especially the transport strip density and track header readability.
2. Improve `Routing` and `Mappings` further so pointer affordances are more explicit and less text-dependent.
3. Move MIDI timing and capture further off the UI loop.
4. Design the timeline note/region editing UX before implementing pointer editing.
5. Decide whether the in-progress Raspberry Pi Zero 2 W deployment scripts should be committed as a finished supported flow.
