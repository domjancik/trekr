# Implementation Plan

## Goal

Build a MIDI-first tracker/player/looper with fixed-fit per-track full/detail columns, then extend the same engine toward audio overlay and modular processing.

## Delivery Strategy

Sequence the work so the transport, timing model, and routing are validated before building heavier editing or audio features.

## Milestone 1: Core Skeleton

Deliver:

- app shell
- project model
- transport model
- fixed-fit paired-column timeline mock rendering
- default vertical-time timeline with `full | detail` per visible track
- active-track selection and highlighting
- page shell for switching between timeline and utility pages
- save/load for minimal project state

Exit criteria:

- app launches on the primary desktop target
- each track's full/detail pair stays synchronized
- a saved project restores transport, tracks, and loop region

## Milestone 2: MIDI Engine

Deliver:

- MIDI input enumeration
- MIDI output enumeration
- MIDI I/O page for selecting and inspecting available devices
- per-track input routing
- per-track output device/channel routing
- active-track routing page for editing current track input/output/channel state
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
- mapping model shared across keyboard, MIDI, and OSC
- mappings page for viewing bindings by source, target, and scope
- transport controls
- track arm/mute/solo mapping
- current-track and absolute-track actions
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

## Remaining MVP Checklist

Priority 1: Workflow completion

- visible in-canvas transport strip for play, stop, record, record mode, and loop status
  status: initial slice implemented, needs hierarchy cleanup
- clearer record-target feedback for armed tracks versus active-track fallback
- save/load actions from inside the app UI
- one deterministic empty-project flow and one deterministic fixture-project flow exposed in UI or startup actions

Priority 2: MIDI authoring usability

- editable MIDI regions on the timeline
- note editing inside the detail column
- region move/resize/delete actions
- safer overdub behavior when new notes overlap existing notes
- explicit preview styling for pending recorded notes versus committed notes

Priority 3: Mapping workflow

- real mapping editor instead of overview-only + enabled toggle
  status: initial field editor implemented
- MIDI learn
  status: initial MIDI note/CC learn implemented
- OSC binding input path
- conflict handling and binding replacement rules
- persistent binding management in save/load flows

Priority 4: Timing and engine hardening

- move MIDI timing further off the UI loop
- timestamp live MIDI input against the transport clock instead of UI-frame polling
- improve playback/record jitter behavior on low-end targets
- add device refresh and reconnect handling for hot-plugged MIDI ports

Priority 5: UI cleanup

- timeline header cleanup
- clearer selection hierarchy in timeline columns
- routing page control hierarchy cleanup
- MIDI I/O card hierarchy cleanup
- stronger modal treatment for the mappings overlay

## Near-Term Sync Milestone: Ableton Link

Ableton Link should land soon after the core MVP becomes comfortable to use, not as a distant rewrite item.

Deliver:

- Link session module isolated from the UI layer
- global transport tempo/phase sync with clear local versus external authority rules
- optional start/stop participation
- conversion between Link beat time and internal tick time
- loop interaction policy for global transport loop versus per-track loops

Exit criteria:

- tempo and beat phase can follow or drive a Link session
- local recording and loop workflows remain coherent while Link is active
- Link integration does not require bypassing the canonical action/transport model

Current note:

- the app now exposes Link-facing transport controls and state through a thin native bridge over the official Ableton Link source so Linux, Windows, and macOS can share one integration path

## Module Breakdown

Recommended first-pass modules:

- `actions`: canonical app command model and source-agnostic action dispatch
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
- `pages`: timeline page, mappings page, MIDI I/O page, and routing page state

## Technical Decision Summary

Current decisions locked in:

- V1 is MIDI-first
- audio is phase two and must fit the same routing/timeline model
- default UI is fixed-fit, not scroll-first
- default view is alternating per-track `full | detail` columns
- active-track-relative actions are first-class, with absolute track targeting also supported
- timeline is linear, not scene-launch based
- tracks may loop independently
- record release commits to the nearest quantize boundary
- modular internal interfaces are preferred so plugin-style extension remains possible later
- Rust is the chosen implementation language
- low-end support targets Orange Pi Zero 2W-class devices and above
- low-end optimization should favor snappy MIDI behavior and nearly static summary-based rendering over higher track ceilings
- input handling should be action-driven so keyboard, MIDI, and later touch/remote control map into the same command surface
- utility pages are part of V1: mappings, MIDI I/O, and active-track routing

## First Build Recommendation

Start with the simplest vertical slice:

1. render a fixed-fit dual timeline from static test data
2. add transport and playhead movement
3. add MIDI device enumeration and manual test output
4. add one armed track with passthrough
5. add hold-to-record into a loop region

This validates the hardest product assumptions before audio or advanced editing work begins.
