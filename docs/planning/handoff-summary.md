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
- per-track MIDI FX:
  - ordered input and output chains
  - compact timeline controls plus Routing-page editing
  - enable/disable, reorder, add/remove, parameter adjustment
- shipped MIDI FX kinds:
  - `Arp`
  - `Note Filter`
  - `Transpose`
  - `Velocity`
  - `Duration`
  - `Scale Quantize`
  - `Chord Quantize`
  - `Delay`
  - `Track Clone`
- global harmony root with quantizer `Local | Global` targeting
- track-clone live monitoring that follows destination `Monitor Input FX` without requiring destination passthrough
- stopped-transport live FX clocking for held/live timing effects (notably arp, delay, duration)
- playback timing lookback for timing transforms so delayed/extended note-offs are still emitted after the source note leaves the current dispatch window
- FX reconfiguration safety: changing timeline FX while notes are sounding flushes active notes / timing state to avoid stuck notes
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
  - `docs/specs/product-spec.md`
  - `docs/dev/architecture.md`
  - `docs/planning/implementation-plan.md`
  - `docs/dev/current-mappings.md`
- Core code:
  - `src/app.rs`
  - `src/actions.rs`
  - `src/mapping.rs`
  - `src/midi_fx.rs`
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
  - `Delete` / `Backspace` remove mapping row
  - `Shift+Left` / `Shift+Right` or `Shift+Up` / `Shift+Down` field/context select
  - `Q` / `E` adjust
  - `Enter` activate / learn
  - `Shift+Enter` reverse timeline FX field cycle
- Transport/record:
  - `Space`
  - `R`
  - `Shift+R`
  - `Home`
  - `G`
  - `Shift+G`
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
- Timeline FX:
  - `Enter` cycles forward within the selected FX row
  - `Shift+Enter` cycles backward within the selected FX row
  - `Q` / `E` adjust selected FX kind/value
  - `Shift+M` toggle selected FX enabled/bypassed
  - `Delete` / `Backspace` delete selected FX or mapping row (context-sensitive)

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

- `a6a0d31` `docs: clarify timing guarantees for delay and duration fx`
- `cdf0994` `fix: reset active notes when timeline fx changes mid-playback`
- `860f47d` `fix: cover duration sustain with playback timing lookback`
- `7cc9816` `fix: recover delayed playback note-offs across frame windows`
- `80014f7` `fix: anchor stopped live input fx to live clock`
- `1430b41` `fix: correct live delay and duration scheduling boundaries`
- `b534a89` `feat: make duration absolute and align fx docs`
- `27369a0` `fix: restore CI for preview scheduling and stored-loop hit tests`

## Current Worktree State

Current local branch state when this summary was refreshed:

- branch: `vk/9b67-feature-spec-mid`
- status: clean worktree
- divergence vs remote branch at refresh time:
  - `ahead 74`
  - `behind 64`

Before pushing or opening follow-up PR changes, re-check branch state and whether a rebase onto current `origin/main` is required.

## Important Implementation Considerations

- Live timing effects are split across two clocks:
  - transport/playback timing when playback is running
  - `live_fx_ticks` when transport is stopped
- Stopped-transport live input must be timestamped from `live_fx_ticks`, not `transport_ticks` / `playhead_ticks`, or delay/duration/arp timing will drift or collapse.
- Playback timing transforms (`Delay`, absolute `Duration`) require source-note lookback beyond the current dispatch window so transformed note-offs are not lost after the source note leaves the current frame.
- FX-engine tests alone are not enough for timing bugs; app-level tests that step playback/live windows explicitly were needed to catch the real failures.
- Changing timing-sensitive FX mid-playback should be treated as a state reset boundary:
  - send note-off/all-notes-off as needed
  - reset live FX runtime state
  - let following notes use only the new configuration
- `Duration` now means absolute musical length, not relative percentage scaling.
- `Delay` is delay-only (no negative look-ahead); any future “shift earlier” concept would need a different design because live paths do not have future note knowledge.

## Highest-Value Next Steps

1. Rebase/push the MIDI FX slice and verify CI on the rebased branch state.
2. Decide whether MIDI mapping for FX parameters/kinds/toggles should land after the related mapping branch merges.
3. Move MIDI timing and capture further off the UI loop.
4. Design the timeline note/region editing UX before implementing pointer editing.
5. Refresh screenshots/UI review again if the next FX/timeline polish materially changes the renderer output.

