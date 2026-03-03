# Feature Spec: Stored Loops

## Summary

This feature adds per-track stored loop regions that can be saved, labeled, recalled, and optionally quantized on launch.

The goal is to deliver part of a clip-launching workflow without introducing clips or scenes as first-class timeline objects. Instead, the user works with reusable track loop presets:

- each stored loop points at a time range on a track
- recalling a stored loop updates that track's active loop region
- recall can happen immediately or be queued to a quantized boundary
- the UI should expose this compactly inside the timeline workflow, with optional expanded editing for the active track

This keeps `trekr` grounded in its current region-and-loop model while still enabling performance-oriented loop switching.

## Problem

The current app lets the user edit one active loop region per track, but that model is too transient for performance use.

Today, if a user wants a "verse", "fill", and "chorus" loop on the same track, they must manually move and resize the track loop each time. That breaks flow and makes the current loop controls feel like editors rather than launch targets.

The missing capability is a reusable loop bank:

- save multiple meaningful loop regions on a track
- recall them quickly during playback
- queue loop changes to musical boundaries
- show what is active and what is queued without turning the timeline into a clip matrix

## Goals

- allow each track to store multiple named loop regions
- make loop recall usable from keyboard and MIDI mappings, not only pointer UI
- support quantized recall so loop switches land on predictable musical boundaries
- preserve the existing timeline-first model where loops are time ranges on tracks
- keep the default UI compact enough for the fixed-fit paired-column layout

## Non-Goals

- introducing clip objects separate from track regions
- scene launching across multiple tracks in this slice
- replacing the current direct loop start/end editing controls
- freeform drag-and-drop loop bank editing
- solving full arrangement performance mode for song-wide coordinated launches

## User Model

The user treats stored loops as reusable launch targets for one track.

Examples:

- Track 1 stores `Intro`, `Verse`, `Chorus`, and `Break`
- Track 2 stores several drum variations across the same arrangement
- during playback, the user queues `Chorus` on Track 1 for the next bar
- the track keeps playing its current loop until the quantized handoff point, then switches to the recalled loop

Stored loops are not copies of MIDI notes. They are pointers to existing timeline ranges.

## Core Terms

- `stored loop`: a saved loop preset attached to one track
- `active loop`: the loop region currently driving playback for that track
- `queued loop`: a stored loop selected for deferred recall at the next launch boundary
- `launch quantize`: the boundary used when queued recall is enabled
- `loop slot`: the UI and mapping address used to target one stored loop entry

## Proposed Model

### Per-Track Loop Banks

Each track gets its own ordered collection of stored loops.

Each stored loop should contain:

- stable id
- track id
- name
- start tick
- length in ticks
- optional color or accent index for future UI use

Recommended initial limits:

- support at least `8` stored loops per track in the model
- render only the most useful compact subset by default

### Relationship To Current Track Loop

The current editable track loop remains the runtime loop state.

Recalling a stored loop should:

- set the track loop region to the stored loop's range
- enable that track's loop if it was off
- leave song loop state unchanged

This preserves the current transport and playback rules instead of creating a parallel launch engine.

### Save And Update Behavior

V1 should support two ways to populate stored loops:

- save current track loop into an empty slot
- overwrite an existing slot from the current track loop

Optional later improvement:

- create a stored loop directly from a selected region or other timeline selection

## Recall Behavior

### Immediate Recall

Immediate recall updates the track loop region as soon as the action is applied.

Use cases:

- stopped transport
- rehearsal editing
- unquantized performance switching

### Quantized Recall

Quantized recall queues a stored loop and applies it only when the next eligible boundary is reached.

Recommended launch quantize options:

- off
- 1/16
- 1/8
- 1/4
- bar
- loop end

Recommended default:

- reuse the current editor/transport quantize value where that produces a clear musical result
- additionally allow `loop end` as an explicit launch-focused option, since clip-style switching often wants full-loop completion rather than smallest-grid timing

### Boundary Resolution

When quantized recall is enabled:

- the user triggers recall for a stored loop on a track
- that track enters a queued state
- playback continues with the current loop until the next launch boundary
- on that boundary, the queued stored loop becomes the active track loop
- the queued state then clears

Recommended V1 rule:

- only one queued loop per track
- a new recall on the same track replaces the previous queued loop

### Behavior While Stopped

If playback is stopped, quantized recall should resolve immediately and update the active track loop without creating a queued state.

That is simpler and matches user expectation for edit-time preparation.

## Launch Semantics

### Track-Local, Not Scene-Global

Stored loops launch independently per track.

That means:

- Track 1 can queue `Chorus`
- Track 2 can stay on `Verse`
- there is no requirement that launches happen as a coordinated cross-track scene

This is the critical scope boundary that keeps the feature loop-based rather than scene-based.

### Interaction With Song Loop

Stored loop recall operates on track loop state, not song loop state.

If the song loop is enabled, existing playback rules still apply. The feature should not silently disable or rewrite the global loop. If the current transport behavior makes song loop dominate track loop behavior in some cases, that should remain true until a later transport-policy change is designed explicitly.

### Interaction With Recording

During active recording on a track, quantized stored-loop recall should be conservative.

Recommended V1 behavior:

- allow immediate or queued recall on non-recording tracks
- block recall on the recording target track while capture is active
- surface a clear status message instead of partially switching loop context mid-take

This avoids ambiguous commit behavior and protects recorded data.

## UI Recommendation

### Primary UI: Compact Slot Strip In Track Chrome

The recommended default UI is a small stored-loop strip attached to each track's timeline presentation rather than a separate launch page.

Suggested compact rendering:

- a short row or column of numbered slots such as `1 2 3 4`
- active slot is visibly filled
- queued slot pulses or uses an outline treatment
- empty slots render as dim placeholders
- if more slots exist than fit comfortably, show a compact overflow marker such as `+4`

This gives the user direct track-local awareness without consuming the whole detail pane.

Preferred placement:

- inside track header/chrome, adjacent to existing track-state controls or loop status
- not over the note field itself

### Secondary UI: Expanded Stored-Loop Inspector For Active Track

Because compact slots cannot carry enough editing detail, the selected track should also have an expanded editor surface.

Recommended shape:

- an expandable section in the timeline page, or
- a lightweight overlay/panel tied to the active track

That expanded view should show:

- slot number
- loop name
- start and end or start and length
- active and queued state
- save/overwrite/clear actions

This two-layer approach is the best fit for the current UI:

- compact on-track status for performance
- expanded active-track editing when needed

### Why Not A Separate Page First

A separate page would hide loop-launch state from the moment of performance and break the "launch from the timeline" goal.

The compact strip plus active-track inspector is more consistent with the current fixed-fit timeline workflow.

## Visual States

Each stored loop slot should communicate one of these states:

- empty
- stored
- active
- queued
- active-and-queued replacement pending is not needed in V1 because each track only has one active loop and one queued loop target

Additional visual rules:

- labels must degrade gracefully when track columns are narrow
- the active state must remain legible without relying on color alone
- queued state should read distinctly from active state

## Action And Mapping Implications

Stored loops should be first-class actions in the existing mapping system.

Recommended action families:

- `Recall Stored Loop Slot 1..8`
- `Recall Stored Loop Slot 1..8 Quantized`
- `Store Current Loop To Slot 1..8`
- `Clear Stored Loop Slot 1..8`
- `Queue Mode Toggle` or `Stored Loop Recall Quantized Toggle`
- `Next Stored Loop`
- `Previous Stored Loop`

Scope rules:

- default actions target the active track
- the action model should leave room for absolute-track variants later

For performance workflows, the most important mapped actions are recall actions, not edit actions.

## Persistence

Stored loops should persist with the project.

Minimum saved data:

- per-track stored loop list
- ordering
- names
- tick ranges

Transient runtime data may also be persisted if useful, but V1 only requires deterministic restoration of the stored loop bank itself. Queued state should not survive restart.

## Edge Cases

- recalling an empty slot does nothing and surfaces clear status feedback
- recalling a stored loop whose range extends beyond current project content is allowed if the loop range itself is valid
- deleting notes or regions inside a stored loop does not delete the stored loop entry; it remains a pointer to that time range
- if the current quantize mode is changed after a loop is queued, the queued recall should resolve using the quantize rule active at queue time, not a moving target
- if a track loop is disabled and a stored loop is recalled, the track loop becomes enabled
- repeated trigger of the already active stored loop should refresh queued state only if quantized recall is explicitly requested

## Acceptance Criteria

- a user can store multiple loop regions per track and persist them with the project
- a user can recall a stored loop on the active track without manually redefining loop start and end
- recalled stored loops update the existing track loop system rather than creating a separate clip object
- quantized recall visibly queues and then switches at a predictable musical boundary
- queued and active states are distinguishable in the UI
- stored-loop recall is available through the shared action and mapping model
- the timeline page can show compact stored-loop state per track without breaking the fixed-fit layout

## Open Questions

- should `loop end` be a distinct launch-quantize mode even if transport quantize already exists
- what is the right default visible slot count per track in the compact UI: `4`, `6`, or `8`
- should stored loops support duplicate ranges with different names for performance labeling, or should identical ranges be collapsed in the editor UI
- when a stored loop is overwritten, should the active or queued state follow the slot identity immediately if that slot is currently referenced
