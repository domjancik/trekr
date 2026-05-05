# Feature Spec: MIDI Runtime Decoupling and Scheduler Refinement

## Summary

Trekr should decouple timing-critical MIDI playback scheduling and output dispatch from the UI/render loop, using a general app-owned runtime plus an absolute due-time MIDI scheduler.

This canonical spec merges and replaces:

- `docs/specs/feature-spec-midi-runtime-decoupling.md`
- `docs/specs/fix-spec-midi-runtime-scheduler-refinement.md`

The reMarkable branch proved the architectural direction, but the main-oriented implementation must stay general Trekr work: no remarkable display, thin-client, or AppLoad surface; preserve `vendor/ableton-link`; keep the current action/project model; and replace FIFO sleep-based delayed output with absolute due-time scheduling.

## Problem

The current render loop couples timing-critical work to UI cost:

- SDL event polling
- pointer/keyboard handling
- MIDI input drain
- MIDI device refresh
- transport advancement
- playback note dispatch
- live FX tick dispatch
- rendering and display sync
- fixed frame sleep

That means expensive rendering, display sync, or refresh work can become playback jitter and live latency. The prior prototype improved directionally by moving playback ticking off-thread, but it still used delayed output commands that slept inside a FIFO worker, which can block urgent live events and note-offs behind an older delayed event.

## Goals

- keep playback timing stable when rendering is slow or blocked
- move playback scheduling off the UI/render loop
- publish playback snapshots back to the UI so rendering can lag without changing MIDI timing
- replace relative-delay FIFO sleeping with absolute due-time scheduling
- let urgent events preempt older future-due delayed events
- preserve current routing, MIDI FX, track clone, loop, and recording semantics
- preserve `vendor/ableton-link` references and avoid unrelated remarkable/thin-client/AppLoad work
- add opt-in runtime/output diagnostics for Pi-class validation

## Non-Goals

- redesigning the UI renderer
- merging the full reMarkable branch into main
- moving every MIDI control path into the runtime in the first slice
- replacing ALSA/midir on standard Linux/Pi installs
- sample-accurate audio scheduling
- Bluetooth MIDI latency work

## Current Code References

- `src/app/input.rs`
- `src/app/mod.rs`
- `src/app/note_runtime.rs`
- `src/midi_io.rs`

Prototype references to extract from, not merge wholesale:

- `vk/508a-remarkable-thin:src/app/support/playback_runtime.rs`
- `vk/508a-remarkable-thin:src/midi_io.rs`
- `vk/508a-remarkable-thin:docs/planning/remarkable-playback-runtime-decoupling-handoff.md`

## Architecture

### 1. General MIDI runtime helper

Preferred location:

```text
src/app/support/midi_runtime.rs
```

Responsibilities:

- own a background playback runtime loop
- maintain a monotonic transport/playhead clock while background playback is active
- receive explicit `SyncState` updates from the UI-owned app state
- compute playback scheduling independently from renderer cadence
- publish runtime snapshots for UI display
- prewarm active/default output ports

### 2. Runtime state boundary

The runtime must not borrow the `App`.

It should receive a compact immutable state snapshot containing at least:

- `Project`
- current `transport_ticks`
- current `playhead_ticks`
- resolved/default output information needed for routing

UI actions remain authoritative and continue mutating `App`; after relevant changes, the app syncs state into the runtime and consumes runtime snapshots back into UI state.

### 3. Absolute due-time output scheduler

Every scheduled output event should conceptually carry:

```text
due_at: Instant
priority: MidiEventPriority
sequence: u64
payload: MidiEventPayload
```

Ordering:

```text
due_at, then priority, then sequence
```

Recommended priorities:

- `Panic`
- `LiveImmediate`
- `NoteOff`
- `Playback`
- `DelayedFx`

Use a scheduler thread with a min-heap or equivalent priority queue. New urgent events must be able to wake the worker early and preempt older future-due delayed events.

### 4. Conservative first slice

The first main-oriented slice should:

- move playback transport advancement and playback note scheduling off the UI frame loop
- publish `transport_ticks` and `playhead_ticks` snapshots back to the UI
- keep UI rendering best-effort
- keep stopped-transport live FX ticking local for now
- keep broader live passthrough/runtime input decoupling as the next phase

### 5. Live passthrough follow-up

After playback timing is stable, route live input through the runtime path:

```text
midir callback -> runtime input queue -> routing/Fx snapshot -> due-time scheduler
```

The UI should still receive copied/timestamped events for mapping learn, direct mapping, recording state, and status display.

### 6. Device refresh and prewarming

MIDI device refresh should not block the timing-critical runtime path. Output connections should be prewarmed when:

- a default output is selected
- a routed output becomes active
- runtime sync introduces a new active output

Prewarming failure should not block playback dispatch.

## Implementation Plan

### Phase 1: Measurement

- add opt-in timing logs for runtime wake cadence and output send lag
- keep diagnostics simple and Pi-friendly

### Phase 2: Playback runtime

- add `src/app/support/midi_runtime.rs`
- move background playback transport advancement and playback note scheduling there
- sync UI-owned state into the runtime after playback-relevant actions
- consume runtime snapshots in the UI loop before rendering / MIDI input handling

### Phase 3: Scheduler refinement

- replace FIFO sleep-based delayed output with absolute due-time scheduling in `src/midi_io.rs`
- use priority ordering so note-offs and urgent events are not trapped behind delayed playback/FX work
- allow panic/all-notes-off to clear queued delayed events

### Phase 4: Live passthrough runtime

- move callback-driven live passthrough into the runtime input path
- keep recording timestamps anchored to actual arrival/runtime time

## Acceptance Criteria

- playback timing no longer changes materially between Timeline and static pages
- UI rendering stalls can occur without causing playback catch-up bursts
- delayed playback/FX events do not block live passthrough events due now
- note-offs at the same due time sort before note-ons unless a test documents another requirement
- panic/all-notes-off can bypass or clear queued delayed events
- first-note output connection delay is measurable and reduced by prewarming
- implementation remains main-oriented and does not import unrelated remarkable display/thin-client/AppLoad work

## Validation

Minimum local validation:

- `cargo fmt`
- `cargo check`
- focused scheduler ordering tests
- focused note-runtime regression tests for playback/live timing semantics

Minimum Pi validation:

- run with `TREKR_MIDI_RUNTIME_LOG=1`
- run with `TREKR_MIDI_OUTPUT_LOG=1`
- compare Timeline vs MIDI I/O playback jitter
- test playback while delayed FX events are queued
- test USB MIDI live passthrough during playback

## Diagnostics

Suggested env vars:

```text
TREKR_MIDI_RUNTIME_LOG=1
TREKR_MIDI_RUNTIME_LOG_PATH=trekr-midi-runtime.log
TREKR_MIDI_OUTPUT_LOG=1
TREKR_MIDI_OUTPUT_LOG_PATH=trekr-midi-output.log
```
