# Feature Spec: Stored Loops (Shipped V1)

## Summary

Stored loops provide per-track loop slots that can be saved, recalled, and queued with launch quantization.

V1 is intentionally loop-model-native:

- stored loops are slot-indexed loop ranges on each track
- recall updates the track loop region instead of creating clip/scene objects
- recall can be immediate or quantized
- UI stays compact and timeline-local

This document is implementation-aligned for the current shipped behavior.

## Goals

- support a per-track bank of stored loop slots
- allow fast recall on the active track from keyboard and mapping actions
- support quantized recall for musical loop switching
- preserve timeline-first loop workflow
- keep the UI compact for fixed-fit and focused-track timeline views

## Non-Goals

- clip/scene launcher model
- multi-track scene launch coordination
- replacing direct loop start/end editing
- full loop-bank editor workflow in V1

## Data Model (V1)

Each track has `8` stored loop slots.

Each stored slot stores:

- start tick
- length in ticks

Runtime-only state per track:

- active stored loop slot (if current loop came from a slot)
- queued stored loop recall target (single pending target)

Queued recall state does not persist across restart.

## Runtime Behavior

### Recall

- recalling a stored slot sets the track loop to that slot range
- recalling a valid stored slot enables that track loop
- recalling an empty slot is a no-op
- recall is blocked on the actively recording track

### Quantized Recall

Global controls:

- `stored_loop_recall_quantized` (on/off)
- `stored_loop_launch_quantize` (`Off`, `1/16`, `1/8`, `1/4`, `Bar`, `LoopEnd`)

Rules:

- one queued recall per track
- a new queued recall on the same track replaces the previous queued target
- if transport is stopped, recall resolves immediately

Boundary resolution:

- `1/16`/`1/8`/`1/4`/`Bar`: resolved on global transport grid
- `LoopEnd`: resolved on the track clip-cycle boundary (`transport_ticks % clip_loop_length`)
- `LoopEnd` launch timing is independent from song-loop wrap

## Song Loop and Clip Loop Semantics

Timing is treated as two related playheads:

- song playhead: global transport with optional song-loop wrapping
- clip playhead: per-track loop phase driven from transport ticks and track loop length

Stored-loop launch uses clip-cycle semantics for `LoopEnd`, while song loop behavior remains unchanged.

## UI Behavior (V1)

Timeline behavior:

- stored loop slot labels are shown on the left side of track loop UI
- slot labels are clickable direct recall targets
- show as many slot buttons as fit in the available width
- focused-track view can show all `8` slots

Track-canvas behavior:

- stored loops are drawn as subtle thin loop markers with slot labels
- overlap is visually encoded
- active/queued state remains distinguishable

## Actions and Mapping

Implemented action families:

- `Recall Stored Loop Slot 1..8`
- `Store Current Loop To Slot 1..8`
- `Clear Stored Loop Slot 1..8`
- `Toggle Stored Loop Recall Quantize`
- `Cycle Stored Loop Launch Quantize`

Default keyboard bindings:

- recall: `Numpad1..Numpad8` and fallback `Alt+1..Alt+8`
- store: `Shift+Numpad1..Shift+Numpad8` and fallback `Shift+Alt+1..Shift+Alt+8`
- quantize toggle: `Shift+L`
- launch quantize cycle: `Shift+Q`

Note: clear-slot actions are available through the shared action/mapping system; no default keyboard clear binding is required in V1.

## Persistence

Persisted:

- per-track stored slot ranges
- slot ordering
- active slot marker

Not persisted:

- queued recall state

## Acceptance Criteria (V1)

- users can store and recall multiple loop slots per track
- slot recall is available via defaults on the active track
- quantized recall queues and switches at predictable boundaries
- `LoopEnd` switching follows clip-cycle timing and is not shifted by song-loop wrap
- active and queued states are visually distinguishable
- slot UI remains compact and usable in timeline and focused-track views
