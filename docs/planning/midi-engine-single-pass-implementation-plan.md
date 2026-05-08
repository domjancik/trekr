# MIDI Engine Single-Pass Implementation Plan

## Purpose

This document formalizes a single substantial implementation pass to move Trekr from a frame-driven MIDI runtime toward a lower-jitter, audio-style MIDI engine with built-in low-overhead instrumentation.

The goal of this pass is to materially improve:

- live MIDI passthrough latency stability
- scheduled playback timing precision
- live timing FX stability
- runtime observability for delay and jitter attribution

This pass is intended to establish the correct runtime architecture first, with later refinements focused on tuning and ergonomic improvements rather than foundational redesign.

## Context

Trekr already has:

- `midir`-based MIDI input and output
- a dedicated MIDI output worker
- playback scheduling logic
- live input FX, output FX, passthrough, recording, and track clone behavior

However, the current timing path still depends heavily on the app frame loop:

- MIDI input is drained on the app loop
- playback scheduling is dispatched in frame-sized windows
- live timing FX are advanced from the app loop
- the UI thread still performs too much live MIDI decision-making

This architecture adds jitter and can accumulate drift under load, especially on lower-powered targets such as OrangePi.

## Goals

### Functional goals

- Move live passthrough off the frame-polled app loop.
- Move live timing FX advancement off the frame-polled app loop.
- Move playback dispatch to absolute due-time scheduling rather than frame-window emission.
- Preserve current routing, monitor, passthrough, recording, and clone semantics as closely as possible.

### Timing goals

- Keep live passthrough as close as practical to the measured hardware and OS baseline.
- Reduce page-dependent and UI-load-dependent MIDI jitter.
- Eliminate gradual delay growth caused by frame-coupled scheduling or queue buildup.
- Improve p95 and p99 timing behavior, not just average delay.

### Observability goals

- Attribute delay across the full path:
  - input callback receive
  - engine dequeue
  - live processing
  - scheduler wait
  - output queue
  - actual send
- Track queue depth and missed deadlines.
- Enable low-overhead periodic summaries and optional diagnostics dumps.

## Non-goals

This pass should not attempt to solve unrelated or later-stage concerns, including:

- a UI diagnostics page or rich visualization layer
- persistent analytics infrastructure
- a broader project-state threading redesign
- an audio engine or audio/MIDI unification
- broad UI or architecture cleanup unrelated to timing
- hard real-time guarantees

## Success criteria

### Functional

- Live passthrough no longer depends on `poll_midi_input()` frame cadence for its hot path.
- Live timing FX no longer depend on `advance_playhead()` or stopped-frame advancement for emission timing.
- Playback note output is dispatched against explicit deadlines rather than “all notes due this frame.”
- Existing routing and monitor semantics remain behaviorally compatible.

### Timing

- Live passthrough median latency is close to the external harness baseline.
- Live passthrough jitter is materially lower than the current frame-driven path.
- Scheduled playback no longer exhibits obvious frame-sized burst timing.
- Delay does not steadily grow during sustained use under steady load.

### Observability

- Runtime diagnostics can report callback-to-send latency.
- Runtime diagnostics can report due-time miss for scheduled playback.
- Runtime diagnostics can report queue depth high-water marks and deadline misses.

## Current architecture summary

### Live input path today

1. `midir` callback receives MIDI in `src/midi_io.rs`.
2. The callback parses the message and pushes `MidiInputEvent` through `std::sync::mpsc`.
3. The app loop calls `poll_midi_input()` once per frame.
4. `poll_midi_input()` drains pending input events.
5. `handle_midi_input_event()` performs:
   - MIDI learn capture
   - direct mapping capture
   - mapping action resolution
   - track matching
   - recording updates
   - input FX processing
   - clone propagation
   - passthrough and monitor output routing
6. Final note on/off commands are queued to `MidiOutputRuntime`.
7. The output worker sends messages in FIFO order.

### Playback path today

1. The app loop computes elapsed frame time.
2. `advance_playhead()` derives advanced musical ticks.
3. `dispatch_midi_notes()` computes all notes due in the frame window.
4. All due events are sent immediately to the output worker.
5. `dispatch_live_arp_events()` also advances live scheduled FX from the app loop.
6. The app loop sleeps for roughly `16 ms`.

## Main timing risks in the current implementation

### 1. Frame-polled input consumption

Input is not processed immediately after callback arrival. This alone can add approximately `0–16 ms` before routing and output processing even begins.

### 2. Frame-window playback dispatch

Scheduled notes are emitted in batches for a frame-sized window rather than at precise due times.

### 3. Frame-coupled live timing FX

Arp, delay, duration, and related live timing behavior advance only when the app loop advances time.

### 4. Overloaded app-thread hot path

The app thread still owns too much timing-sensitive work:

- routing decisions
- live FX processing
- clone propagation
- record capture decisions
- passthrough output generation

### 5. Missing callback timestamp preservation

The current input callback does not preserve the callback timestamp in a way that Trekr can use for end-to-end latency measurement.

### 6. FIFO output without deadline awareness

The output worker isolates sending from the UI thread, but it does not itself represent a precise scheduling layer.

## Target architecture

The single-pass implementation should explicitly separate three timing domains.

### 1. UI and app domain

Responsible for:

- project editing and mutation
- page state
- mapping editor and learn UI
- transport controls
- rendering
- publication of immutable runtime snapshots

### 2. MIDI engine domain

Responsible for:

- live musical input handling
- live passthrough processing
- live timing FX advancement
- playback scheduling
- output dispatch decisions
- low-overhead telemetry capture

### 3. MIDI I/O domain

Responsible for:

- `midir` callbacks
- input parsing
- output port connection ownership
- final message send calls

The UI should configure the engine, not act as the engine’s clock.

## Proposed single-pass design

## A. Add a dedicated MIDI engine module

Add a new module, likely:

- `src/midi_engine.rs`

This module should own:

- the engine thread
- engine command types
- engine scheduling structures
- engine timing state
- engine telemetry structures

## B. Preserve `midi_io.rs` as the I/O boundary

`src/midi_io.rs` should continue to own:

- `midir` input connection setup
- `midir` output connection setup
- raw parse and send behavior

It should be extended to:

- stamp callback receipt time with `Instant`
- include sequence or correlation ids where useful
- forward live input to the engine path with minimal extra overhead

## C. Publish immutable config snapshots from the app

The engine must not depend on reading mutable app state directly.

The app should publish an immutable `MidiEngineConfigSnapshot` that includes, at minimum:

- track routing
- passthrough flags
- monitor-input-fx flags
- record-input-fx mode
- input FX chain snapshots
- output FX chain snapshots
- clone relationships
- global harmony root
- default port selections if required for resolution
- transport tempo and PPQN data required by the engine

Preferred representation:

- `Arc<MidiEngineConfigSnapshot>`
- generation/version counter

The app should republish the snapshot whenever routing, FX, track state, or relevant transport configuration changes.

## D. Introduce an engine command model

The engine should receive commands for:

- input events from callbacks
- config snapshot updates
- transport state updates
- playback schedule updates
- panic/all-notes-off
- shutdown

The engine should emit:

- output commands to the output runtime
- telemetry events or metric samples
- compact runtime status snapshots if needed later

## E. Move live musical input into the engine fast path

The latency-sensitive live path should become:

1. MIDI callback receives input.
2. Callback timestamps and forwards the event to the engine queue.
3. Engine matches track routing and channel filters using the current snapshot.
4. Engine applies input FX semantics.
5. Engine performs clone propagation and output FX processing.
6. Engine emits immediate or timed output commands.
7. Output runtime sends the message.

This should preserve current semantics for:

- passthrough
- monitor input FX
- record-input-fx mode
- track clone behavior

UI-facing MIDI actions such as learn and mapping capture may remain on the app side or be duplicated to a lighter app-control queue, but they must not block the live output path.

## F. Move live timing FX advancement into the engine

The engine must own a monotonic timing source for live timing FX so they are no longer advanced by app frames.

This should cover:

- arp timing
- delay timing
- duration timing
- related future live timing behaviors

The engine should derive live musical tick progression from:

- `Instant::now()`
- current tempo and PPQN
- transport playing state and linkage state

## G. Move scheduled playback dispatch into engine-side due-time scheduling

Scheduled playback should no longer emit directly from frame-window processing.

A practical one-pass design is:

1. The app continues deriving upcoming playback material from project state.
2. The app publishes a rolling near-future playback plan to the engine.
3. The engine converts planned events to `due_at: Instant`.
4. The engine stores those events in a due-time priority queue.
5. The engine wakes based on the earliest due event and emits it near deadline.

This keeps the project-model traversal largely app-owned while moving the timing-critical dispatch into the engine.

## H. Use explicit output priorities

The engine should prioritize, at minimum:

1. panic / all-notes-off
2. live immediate passthrough
3. note-off events
4. live scheduled FX output
5. playback note-on events
6. diagnostics and low-priority internal work

This prevents live play from sitting behind playback bursts or deferred events.

## I. Keep the output worker simple in the first pass

The output worker does not need to become a second scheduler in this pass.

The engine should hold commands until they are due, then enqueue them to the output worker for prompt send.

The output worker should, however, support telemetry metadata capture so the engine can correlate:

- enqueue time
- dequeue time
- send time

## Instrumentation design

## 1. Timestamped input event model

Extend or replace the current input event so it carries:

- source port
- parsed MIDI meaning
- callback receive time
- optional backend-provided timestamp
- optional correlation id

## 2. Bounded telemetry ring buffer

Implement a bounded in-memory telemetry buffer with:

- fixed capacity
- overwrite-oldest or drop counting
- no per-event heap allocation on the hot path
- no string formatting in the hot path

Telemetry should be opt-in and designed to remain cheap enough for production debugging on constrained devices.

## 3. Telemetry event types

At minimum, support:

- input callback received
- engine input dequeued
- live event processed
- playback event queued
- output command queued
- output command dequeued
- output command sent
- deadline miss
- queue depth sample

## 4. Correlation identifiers

Each note-related event should have enough identity to trace the path through the engine:

- sequence id or source id
- note pitch and channel
- origin kind:
  - live immediate
  - live generated FX
  - playback
  - clone-derived

## 5. Derived metrics

The implementation should support reporting:

- callback receive to engine dequeue
- engine dequeue to live processing completion
- processing completion to output queue
- output queue to actual send
- callback receive to actual send total
- playback due-time miss
- queue depth high-water mark
- count of missed deadlines over thresholds
- count of dropped telemetry records

## 6. Monitoring interfaces

Add environment-variable-based control such as:

- `TREKR_MIDI_DIAG=1`
- periodic summary interval control
- optional dump-on-exit path

The first pass should prefer:

- periodic console summaries
- optional structured dump files

It should avoid:

- per-event `println!`
- synchronous file writes on the hot path
- UI rendering dependencies

## Concrete file and module plan

## New files

Expected additions:

- `src/midi_engine.rs`
- optionally `src/midi_diag.rs` if telemetry deserves its own module

## Existing files likely to change

### `src/midi_io.rs`

- add timestamped input events
- preserve callback timing data
- feed engine-facing input path
- attach telemetry hooks

### `src/app/input.rs`

- remove ownership of latency-critical live musical path
- keep UI/mapping and control-oriented responsibilities

### `src/app/mod.rs`

- create and own the engine instance
- publish config snapshots
- publish transport updates
- publish playback schedule updates
- stop using the frame loop as the authoritative live MIDI clock

### `src/app/note_runtime.rs`

- refactor direct-send playback logic into upcoming playback plan generation
- stop sending playback MIDI directly from the app loop

### `README.md`

- update only if developer-facing runtime diagnostics become part of the normal documented workflow

## Implementation sequence

The single-pass implementation should proceed in this order.

### Step 1 — Telemetry primitives and timestamped input

Add:

- timestamped input events
- telemetry event types
- bounded telemetry buffer

This should happen first so subsequent changes are measurable.

### Step 2 — Engine shell and command channels

Add:

- engine module
- engine thread
- engine command/event types
- output command metadata structures

At this stage, behavior can still mostly mirror the existing path.

### Step 3 — Live passthrough fast path

Move the live musical note path into the engine:

- routing/filter matching
- input FX processing
- passthrough handling
- clone propagation
- output FX processing
- immediate output command generation

### Step 4 — Live timing FX engine clock

Move stopped and playing live timing advancement into the engine using a monotonic clock.

### Step 5 — Playback scheduling handoff

Refactor app-owned playback derivation into a rolling published plan and move deadline execution into the engine.

### Step 6 — Diagnostics summaries and dump hooks

Add:

- periodic summary reporting
- dump-on-exit support
- queue depth and missed-deadline visibility

### Step 7 — Remove obsolete frame-driven MIDI responsibilities

Once validated, simplify or remove:

- frame-owned live scheduling responsibilities
- direct playback MIDI send logic from the app loop
- unnecessary app-thread live timing work

## Validation plan

## Automated validation

Preserve and extend tests covering:

- routing behavior
- passthrough behavior
- clone behavior
- record-input-fx semantics
- scheduled event ordering
- panic/all-notes-off precedence
- telemetry buffer behavior

Add targeted tests for:

- callback timestamp propagation
- engine snapshot update handling
- due-time queue ordering
- deadline miss accounting

## Manual validation

Use the dedicated loopback and trigger-based harness to compare:

1. baseline external path
2. Trekr live passthrough on a minimal page
3. Trekr live passthrough on the Timeline page
4. Trekr scheduled playback under load
5. logging enabled versus disabled

Key outputs to compare:

- p50 / p95 / p99 latency
- worst-case spikes
- drift over time
- deadline miss counts

## Acceptance thresholds

This pass should be accepted when:

- live passthrough behaves near baseline and is materially more stable than before
- scheduled playback no longer shows obvious frame-sized dispatch bursts
- instrumentation clearly identifies remaining bottlenecks
- no major semantic regressions appear in routing or FX behavior

## Risks and cautions

### Snapshot correctness risk

If config snapshots omit important routing or FX state, engine behavior may diverge from UI expectations.

### Behavior-parity risk

Track clone, monitor-input-fx, and record-input-fx interactions are subtle. They must be treated as parity-sensitive during the move.

### Instrumentation overhead risk

Poorly designed logging can itself create jitter. All telemetry must be bounded and low allocation.

### Transport anchoring risk

Link and non-Link transport timing must remain coherent when the engine becomes the timing owner for playback dispatch.

## Later refinements after this pass

After this architecture lands, later refinement work can focus on:

- scheduler wake strategy tuning for OrangePi
- telemetry presentation in the UI
- more efficient snapshot publication
- selective lock reduction and allocation trimming
- hot-plug resilience
- optional multi-queue or per-port worker refinements

## Final implementation directive

The single-pass implementation should explicitly choose:

- yes to a dedicated MIDI engine thread
- yes to immutable app-to-engine snapshots
- yes to timestamped input events
- yes to bounded low-overhead telemetry
- yes to engine-owned live timing FX advancement
- yes to engine-owned absolute due-time playback dispatch
- no to a UI diagnostics page in this pass
- no to unrelated structural cleanup

The primary objective is to stop using the app frame loop as Trekr’s effective MIDI clock while preserving current musical behavior as closely as possible.
