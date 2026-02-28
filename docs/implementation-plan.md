# Implementation Plan

## Goal

Build a MIDI-first tracker/player/looper with fixed-fit full-song and loop-detail panes, then extend the same engine toward audio overlay and modular processing.

## Delivery Strategy

Sequence the work so the transport, timing model, and routing are validated before building heavier editing or audio features.

## Milestone 1: Core Skeleton

Deliver:

- app shell
- project model
- transport model
- fixed-fit dual-pane timeline mock rendering
- track row/column orientation toggle
- save/load for minimal project state

Exit criteria:

- app launches on the primary desktop target
- full song pane and loop pane stay synchronized
- a saved project restores transport, tracks, and loop region

## Milestone 2: MIDI Engine

Deliver:

- MIDI input enumeration
- MIDI output enumeration
- per-track input routing
- per-track output device/channel routing
- MIDI passthrough
- transport-scheduled MIDI playback

Exit criteria:

- external hardware can be driven from multiple tracks on distinct channels
- live input can be monitored and passed through on armed tracks

## Milestone 3: MIDI Timeline Authoring

Deliver:

- linear MIDI regions on the timeline
- region creation and selection
- latch record
- hold-to-record
- nearest-quantize commit on release
- overdub/replace modes

Exit criteria:

- user can record regions into the selected loop range
- resulting regions replay with stable timing against the transport

## Milestone 4: Control Mapping

Deliver:

- MIDI learn
- note and non-note control mapping
- transport controls
- track arm/mute/solo mapping
- loop-region controls
- macro target abstraction

Exit criteria:

- external controllers can drive the core workflow without mouse-only interaction

## Milestone 5: Overview Rendering Quality

Deliver:

- MIDI density summaries for full-song view
- clearer region rendering in loop-detail view
- track compaction strategy for high track counts
- stable playhead rendering in both panes

Exit criteria:

- the default fit-all layout remains readable as projects grow
- no scrolling is required for normal V1 workflows

## Milestone 6: Audio Foundations

Deliver:

- audio device abstraction
- audio transport synchronization
- shared routing model that can represent fewer audio returns than MIDI tracks
- placeholder audio region model

Exit criteria:

- architecture supports later audio overlay without restructuring the project model

## Module Breakdown

Recommended first-pass modules:

- `app`: startup, configuration, platform integration
- `project`: persistent project model
- `transport`: timing, tempo map, loop state
- `engine`: real-time playback and scheduling
- `midi_io`: device input/output and event translation
- `routing`: per-track routing and passthrough rules
- `timeline`: region model and edit operations
- `ui`: panes, controls, and interaction state
- `render`: fixed-fit timeline drawing and cached summaries
- `mapping`: MIDI learn and action binding

## Technical Decision Summary

Current decisions locked in:

- V1 is MIDI-first
- audio is phase two and must fit the same routing/timeline model
- default UI is fixed-fit, not scroll-first
- detail pane shows the full selected loop region
- timeline is linear, not scene-launch based
- record release commits to the nearest quantize boundary
- modular internal interfaces are preferred so plugin-style extension remains possible later

## First Build Recommendation

Start with the simplest vertical slice:

1. render a fixed-fit dual timeline from static test data
2. add transport and playhead movement
3. add MIDI device enumeration and manual test output
4. add one armed track with passthrough
5. add hold-to-record into a loop region

This validates the hardest product assumptions before audio or advanced editing work begins.
