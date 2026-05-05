# Feature Spec: MIDI Runtime Decoupling

## Summary

Trekr should decouple timing-critical MIDI input, passthrough, playback scheduling, and output dispatch from the UI/render loop.

The current Raspberry Pi KMSDRM path runs MIDI polling and playback dispatch once per rendered frame. On low-end targets, expensive rendering, display sync, device refresh, or UI sleeps can directly become MIDI latency and jitter. This is the same architectural problem observed on the reMarkable branches when heavy qtfb rendering destabilized playback timing.

## Current Problem

The current app loop performs these operations on one thread:

- SDL event polling
- pointer/keyboard handling
- MIDI input drain
- MIDI device refresh
- transport advancement
- playback note dispatch
- live FX tick dispatch
- rendering and display sync
- fixed `16ms` sleep

This couples MIDI timing to UI work. A MIDI event received just after `poll_midi_input()` can wait until the next frame before Trekr handles it. Playback events are also emitted only when `advance_playhead()` runs, so the scheduler inherits frame-loop jitter.

Expected symptoms:

- live passthrough feels delayed on the Pi
- playback timing changes depending on which page is rendered
- Timeline page causes more jitter than static pages
- occasional catch-up bursts after a blocked render/display-sync/device-refresh frame
- first output note may be delayed by lazy output connection setup

## Goals

- keep MIDI timing stable when rendering is slow or blocked
- reduce live passthrough latency below one UI frame
- make playback scheduling independent from page rendering cost
- keep the existing action model and project model intact
- preserve current routing, MIDI FX, track clone, loop, and recording semantics
- add timing diagnostics that make latency/jitter visible on-device

## Non-Goals

- redesigning the UI renderer
- replacing ALSA/midir on normal Linux/Pi images
- merging the full reMarkable branch into main
- implementing sample-accurate audio scheduling
- solving Bluetooth MIDI transport latency

## Current Code References

- `src/app/input.rs`: drains queued MIDI input events once per app loop and handles live input/passthrough.
- `src/app/mod.rs`: advances transport/playhead and dispatches MIDI notes from the render loop.
- `src/app/note_runtime.rs`: computes playback events and sends them through `MidiOutputRuntime`.
- `src/midi_io.rs`: owns midir input connections and the output worker thread.

## Branch Research

Relevant local branches:

- `vk/508a-remarkable-thin`
- `vk/5bf4-implement-spec-r`
- `vk/bd2e-test-userspace-m`
- `vk/remarkable-thin-client-discovery`

The useful prior art is the playback runtime work in:

- `src/app/support/playback_runtime.rs`
- `src/app/support/remarkable.rs`
- `docs/planning/remarkable-playback-runtime-decoupling-handoff.md`

That branch identifies the same root cause: qtfb/UI rendering stalls delayed transport advancement and caused batched note dispatch. Its proposed remedy is a dedicated playback runtime thread that owns steady transport ticking and publishes a lightweight snapshot back to the UI.

### Assessment

The approach is directionally correct and should be generalized for Pi:

- move playback timing off the UI/render thread
- use a snapshot bridge so UI rendering can lag without changing MIDI timing
- send state changes from UI to the runtime explicitly
- add timing diagnostics around wake cadence, scheduled events, and output dispatch

Do not copy the branch wholesale:

- it is reMarkable-specific in naming and activation policy
- it carries unrelated display, RTP-MIDI, thin-client, and USB-MIDI spike work
- it clones broad `Project` state into the runtime, which is acceptable as a spike but should be tightened for a general engine boundary
- it adds delayed send support by sleeping inside a single FIFO output worker; that can serialize later urgent events behind an earlier delayed one

The branch should be treated as a prototype proving the architecture, not as the final implementation shape.

## Proposed Architecture

### 1. MIDI Engine Runtime

Add a general app-owned runtime module, for example:

```text
src/app/support/midi_runtime.rs
```

Responsibilities:

- own the high-priority timing loop
- maintain a monotonic transport clock while playback is active
- receive MIDI input events from midir callbacks
- process live passthrough routes and low-latency live output
- schedule playback note events independently from UI frame rate
- publish snapshots for UI display

### 2. Runtime State Snapshot

The runtime should not borrow the UI `App`.

It should receive a compact immutable routing/playback snapshot:

- transport state: playing, tempo, ppqn, loop settings
- current transport/playhead ticks
- tracks needed for scheduling and routing
- MIDI routing defaults
- enabled MIDI FX chains needed for playback/live passthrough
- mappings only if direct MIDI-to-action timing is later moved into runtime

UI actions mutate `App` as today, then mark the runtime snapshot dirty. The runtime receives a `SyncState` command.

### 3. Output Scheduling

Replace frame-driven playback dispatch with runtime-owned due-time scheduling.

Required behavior:

- playback events carry intended musical ticks and computed due `Instant`
- the scheduler wakes near the next due event rather than once per UI frame
- delayed FX events are scheduled by due time, not by sleeping inside a FIFO output worker
- immediate live passthrough events bypass delayed queues where possible

Implementation options:

- a binary heap / priority queue of due MIDI events in the runtime thread
- per-output worker queues that preserve due-time ordering
- ALSA sequencer timestamping later, if the backend exposes a practical path

### 4. Live MIDI Input Path

For live passthrough, avoid waiting for the UI frame loop.

Recommended path:

```text
midir callback -> runtime input queue -> routing/Fx snapshot -> output scheduler/send
```

The UI can still receive a copy of input events for:

- mapping learn
- direct mapping capture
- recording UI state
- status/diagnostics

For recording, the runtime should attach a timestamp/tick to the input event so the UI can commit notes with the time they actually arrived, not the time the next frame processed them.

### 5. MIDI Device Refresh

Device scanning should not block the timing loop.

Recommended path:

- background scan task refreshes the catalog
- UI applies visible catalog updates
- runtime receives resolved port updates
- scan clients/plumbing are filtered per `docs/specs/feature-spec-midi-loopback-port-filtering.md`

### 6. Diagnostics

Add opt-in diagnostics suitable for Pi testing:

- runtime wake interval and max wake delay
- MIDI input callback-to-runtime delay
- live input-to-output dispatch delay
- playback scheduled tick vs actual send time
- output queue depth
- first-note connection time
- MIDI refresh scan duration
- UI frame/present duration

Candidate env vars:

```text
TREKR_MIDI_RUNTIME_LOG=1
TREKR_MIDI_RUNTIME_LOG_PATH=trekr-midi-runtime.log
TREKR_MIDI_OUTPUT_LOG=1
TREKR_MIDI_OUTPUT_LOG_PATH=trekr-midi-output.log
```

## Implementation Plan

### Phase 1: Measure

- add low-risk timing logs to the current Pi path
- log UI frame duration, MIDI input drain cadence, output queue latency, and refresh duration
- verify whether Timeline vs MIDI I/O page changes measured delay

### Phase 2: Playback Runtime

- extract playback scheduling into a general runtime thread
- keep UI authoritative for state edits
- use snapshots from runtime for playhead rendering
- verify playback no longer changes materially with page/render cost

### Phase 3: Live Passthrough Runtime

- route live input through the runtime snapshot
- preserve UI-side mapping learn/control behavior via copied events
- timestamp recording events at arrival/runtime time
- verify live passthrough latency is no longer bounded by the render frame

### Phase 4: Scheduler Quality

- replace FIFO sleep-based delayed output with due-time scheduling
- add priority handling so immediate live events are not blocked by delayed playback/FX events
- consider OS scheduling hints or realtime priority setup for appliance deployments

## Acceptance Criteria

- Live MIDI passthrough does not wait for the UI frame loop.
- Playback note timing remains stable when the Timeline page is visible.
- Playback note timing remains stable if rendering/present occasionally exceeds `16ms`.
- MIDI device refresh cannot block the timing-critical runtime path.
- First-note output connection delay is measurable and can be avoided by prewarming selected output connections.
- UI playhead display can lag or drop frames without changing MIDI output timing.
- Diagnostics can show input-to-output latency and scheduled-vs-actual playback send timing on the Pi.

## Validation

Minimum local validation:

- focused tests for scheduler ordering and delayed event behavior
- focused tests that immediate live events are not blocked behind delayed events
- `cargo check`

Minimum Pi validation:

- deploy Bookworm artifact to Orange Pi Zero 2W
- run with MIDI runtime diagnostics enabled
- compare Timeline page vs MIDI I/O page playback jitter
- compare live passthrough latency before/after runtime decoupling
- test with USB MIDI, not Bluetooth, for baseline measurements

