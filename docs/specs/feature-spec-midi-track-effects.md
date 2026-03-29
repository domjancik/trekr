# Feature Spec: MIDI Track Effects

## Summary

This feature adds per-track MIDI effect chains in two places:

- `input effects` before a track's playable/recordable note stream is committed
- `output effects` after the track's recorded/generated notes are resolved for playback

The design should fit the current `trekr` model:

- action-driven control via `AppAction`
- active-track-relative behavior by default, with absolute-track scope available for mappings
- compact fixed-fit timeline UI
- Routing page as the existing active-track inspector

The output chain is post-track and post-recording-only:

- it affects live passthrough and track playback output
- it does **not** change what gets recorded into the track

The input chain is pre-track:

- it can transform live MIDI input and cloned track input before that material reaches the track
- whether input-chain output is what gets recorded must be explicitly controllable

This spec defines UX, behavior, scope, ordering, conflict rules, and likely code touch points. Implementation is intentionally deferred.

Related docs:

- `docs/specs/product-spec.md`
- `docs/specs/feature-spec-midi-manipulation.md`
- `docs/specs/feature-spec-stacked-recordings.md`
- `docs/dev/architecture.md`
- `docs/planning/handoff-summary.md`

## Problem

The current repository supports:

- per-track MIDI input routing
- per-track MIDI output routing
- passthrough
- recording and playback
- action-driven mappings

But it does not yet support a reusable per-track MIDI processing pipeline. That leaves several gaps:

- no way to non-destructively transform track playback output
- no way to shape track input before recording or monitoring
- no mechanism for additive track-to-track MIDI layering
- no consistent parameter model for future MIDI processors and modulation

## Goals

- add a minimal, extensible MIDI effects model for tracks
- support both `input` and `output` chains
- keep recording semantics explicit and predictable
- keep UI compact in the timeline
- reuse the current active-track and action/mapping model
- support duplicate/additive layering where it is musically useful
- leave room for future modulation and automation without redesigning parameter storage

## Non-Goals

- VST/AU/LV2 plugin hosting
- audio effects
- full piano-roll style graphical editing of effect curves
- a new page dedicated only to effects in the first slice
- deep modulation routing UI in the first slice

## Current Repository Grounding

Current code shape relevant to this feature:

- `src/actions.rs` already centralizes canonical actions
- `src/mapping.rs` already expands active-track and absolute-track mapping scopes
- `src/project.rs` already stores per-track editable state and serialized track data
- `src/routing.rs` currently stores only input/output ports and channels
- `src/app.rs` currently applies MIDI input directly to recording and passthrough, and dispatches scheduled playback directly to MIDI output
- `src/pages.rs` currently exposes `Timeline`, `Mappings`, `MIDI I/O`, and `Routing`

Current live MIDI path in practice:

- incoming MIDI is matched to tracks by input port/channel
- note input can be recorded into `active_take`
- passthrough sends matching note-on/off directly to the track output port/channel
- scheduled playback sends track notes directly to output

This means the feature should insert effect processing into existing input/passthrough/playback paths rather than invent a separate transport model.

## Core Model

Each MIDI track gets two ordered chains:

- `input_chain`
- `output_chain`

Each chain contains ordered `effect instances`.

Each effect instance has:

- stable instance id
- effect kind
- enabled/bypassed state
- ordered parameter set
- compact summary label
- optional warning state

## Processing Model

### Output Chain

The output chain sits at the end of the track.

It processes:

- scheduled playback from the track
- live passthrough output from that track

It does not process:

- recorded note storage
- note selection/edit state
- source-track note data at rest

So output effects are always non-destructive and playback-only.

### Input Chain

The input chain sits before the track's internal note stream.

It processes a merged input bus made from:

- live MIDI input matched by the track's current routing
- any track-clone sources configured in the chain

The input chain feeds:

- monitor/passthrough preview
- optionally recording

## Recording Semantics

To keep recording behavior predictable, the track should expose a track-level record source mode:

- `Record Dry Input`
- `Record Post Input FX`

Recommended V1 behavior:

- default is `Record Dry Input`
- live monitoring/passthrough always reflects the input chain when input effects are enabled
- recording uses the selected record source mode

This avoids a more confusing split where some input effects affect recording and others do not.

Track clone sources follow the same rule:

- in `Record Dry Input`, clone-generated notes are heard but not committed
- in `Record Post Input FX`, clone-generated notes after input-chain processing are recordable

## Effect Categories

### Source Effects

Source effects generate or import note streams into the input bus.

Initial source effect:

- `Track Clone`

Behavior:

- taps another MIDI track as a note source
- multiple clone instances are additive
- each clone instance is independent and may have its own parameters

### Transform Effects

Transform effects accept a note stream and emit a transformed note stream.

Initial target set:

- note filter
- transpose
- velocity control
- duration control
- scale quantize
- chord progression quantize
- time shift

## Initial Effect Definitions

### Track Clone

- source: another MIDI track
- tap point: source track note stream before the source track output chain
- result: cloned notes are merged into the destination track input bus
- layering: multiple clone instances sum additively

Rules:

- clone cycles are invalid
- direct self-clone is invalid
- indirect cycles must be blocked
- muted source-track playback should not suppress clone generation unless a later policy explicitly says so

Recommended tap semantics:

- clone the source track's musical note stream, not its final output-FX result
- input-FX on the source track may influence the source if they are part of what the track itself produces
- output-FX never propagate through cloning

### Note Filter

- filters by note range
- should support inclusive low/high note bounds
- may later support explicit pitch-class or whitelist modes

### Transpose

- semitone delta
- negative and positive values

### Velocity Control

- multiplicative amount around a normalized base
- should support at least `0x` to `2x+`
- clamp to MIDI range

### Duration Control

- multiplicative note length scaling
- minimum post-scale duration clamp required

### Scale Quantize

- remaps notes to a selected scale/root

### Chord Progression Quantize

- remaps notes against a progression-aware harmonic target
- expected to need richer editing than the simple inline controls

### Time Shift

- note start offset
- optional speed/density scaling

## Ordering Rules

- effects run top to bottom in chain order
- later effects see the output of earlier effects
- source effects contribute into the input bus at their position in the chain model
- when multiple source effects exist, their outputs merge additively before reaching later downstream transforms

## Conflict And Replacement Rules

### General Rule

No implicit destructive replacement.

- adding an effect appends by default
- inserting places a new effect at the chosen position
- editing an effect changes that instance in place
- removing deletes only that instance

### Duplicate Instances

Duplicate instances are allowed by default.

Reason:

- duplicates are musically meaningful for transpose, timing, filtering, and especially track cloning

### Warnings Instead Of Forced Replacement

The UI should warn, not auto-rewrite, for likely-confusing stacks such as:

- multiple pitch-quantizers in one chain
- multiple heavy timing transforms in one chain
- duplicate track clones from the same source

Those remain valid configurations.

### Invalid Configurations

These should be blocked, not merely warned:

- clone self-reference
- clone graph cycles
- source track reference to a non-MIDI track type if audio/hybrid arrives later and is unsupported

## Parameter Model

Parameters should use one consistent, extensible representation across all effects.

Recommended shape:

- each effect kind declares a parameter schema
- each parameter has a stable id, display label, type, default, and UI hint
- instance state stores values by parameter id

Parameter types should support at least:

- bool
- discrete enum
- integer range
- normalized scalar
- note/range pair where relevant

UI hint levels:

- `compact` for single-character/fader/chip display
- `inline` for small multi-control summaries
- `expanded` for modal/panel editing

This allows simple effects to stay tiny while richer ones like arp/progression tools can open a deeper editor later.

## UX Model

### Timeline

The timeline should stay minimal.

Per track:

- show `input FX` stack near the track input/header area
- show `output FX` stack at the end/below the track as requested
- each effect renders as a compact chip or short row
- if an effect has one dominant parameter, show only that parameter inline

Example visual shape:

```text
IN
CLN
TRN

...

OUT
FLT
TPS
```

Expanded labels may be shown when focused track view has enough width.

### Routing Page Reuse

The first full editor surface should extend the existing `Routing` page for the active track instead of adding a new page.

Recommended Routing page additions:

- input chain list
- output chain list
- selected effect inspector
- record source mode selector

This fits the page's existing role as the active-track configuration surface.

### Expanded Editing

For effects with more than one compact parameter:

- selecting the effect on the Routing page reveals an inspector panel
- touch may use a modal sheet/full-screen overlay
- desktop may use inline panel space first

## Interaction Model

### Desktop

Desktop should support:

- keyboard navigation through effect lists and effect parameters
- pointer tap/click to select effect
- pointer click on compact controls for bypass/add/remove
- wheel or existing `Q/E`-style adjust actions for selected values where consistent with the current page model

### Touch

Touch should prioritize:

- tap to select effect
- tap hold or secondary tap to open expanded editor
- larger hit targets than the compact desktop chips

Touch should not require drag-reorder in the first slice.

### Reordering

Reordering matters, but first-slice interaction should avoid fragile gestures.

Recommended V1 control model:

- move selected effect up
- move selected effect down

through canonical actions and optional pointer buttons.

## Action Model Reuse

Effects should be controlled through canonical actions, not page-local imperative logic.

Likely action families:

- select previous/next input effect
- select previous/next output effect
- add input effect
- add output effect
- remove selected effect
- move selected effect up/down
- toggle selected effect bypass
- adjust selected parameter backward/forward
- enter/exit expanded parameter editing
- cycle record source mode

Mapping scope should follow the current repository convention:

- default scope is `Active Track`
- mappings may target absolute track numbers

This matches the existing behavior in `src/mapping.rs`.

## Scope Behavior

### Default Scope

Effects editing and toggling are active-track-relative by default.

### Absolute Scope

Mappings should be able to target:

- `Track 1`
- `Track 2`
- etc.

for effect management actions, matching current track-scoped mapping behavior.

### Track Selection Interaction

Changing active track changes which chain is shown in the Routing page and which compact stack is emphasized in the timeline.

It does not alter effect order, bypass, or parameters on other tracks.

## Monitoring And Playback Rules

### Live Input

When MIDI enters a track:

- it first resolves against the track's input routing
- it enters the input bus
- input-chain processing is applied
- result is used for monitor/passthrough output
- recording uses dry or post-input-FX material based on record source mode

### Playback

When track notes play from the arrangement:

- recorded/stored track notes are scheduled as today
- output-chain processing is applied before MIDI output send

## Serialization

Effect chains and parameters should be serialized with the track in project state.

Runtime-only fields may include:

- temporary warning/cache state
- cycle-detection cache
- live modulation runtime phase

## Future Modulation

The parameter model should leave room for:

- procedural modulation
- recorded automation
- mapping-driven macro control

Recommended principle:

- modulation targets parameters by stable parameter id on stable effect instance id

This avoids redesigning storage when automation arrives.

## Acceptance Criteria

- a MIDI track can contain an ordered input effect chain and an ordered output effect chain
- output effects alter playback/passthrough output without changing recorded note data
- input effects can be heard before recording and support an explicit dry-vs-post-input-FX record mode
- a user can add more than one track clone and the result is additive
- clone cycles and self-clones are prevented
- effect order is user-controlled and musically significant
- effects are manageable from a compact timeline presentation and an active-track editor on the Routing page
- effect control routes through the canonical action layer and supports active-track and absolute-track mapping scope
- parameter storage is schema-driven enough to support future effects and modulation without per-effect ad hoc UI state

## Likely Code Touch Points

- `src/project.rs`
  - add serialized track effect-chain state
  - add per-track record source mode
- `src/routing.rs`
  - extend routing/processing model beyond ports/channels
- `src/app.rs`
  - insert input-chain processing into MIDI input handling
  - insert output-chain processing into playback dispatch
  - extend Routing page rendering and interaction
  - add compact timeline stack rendering
- `src/actions.rs`
  - add canonical effect-management actions
- `src/mapping.rs`
  - expose new effect actions and scopes in mapping labels/options
- `src/pages.rs`
  - likely extend routing page field/state enums, unless effect editing gets its own sub-state
- `src/midi_io.rs`
  - likely broaden output send path if processed streams emit transformed note events beyond the current direct note-on/off helpers

Likely new module(s):

- `src/midi_fx.rs`
- `src/midi_fx/` with effect definitions, schemas, and processing utilities

## Open Questions

- should source-track mute/solo influence track-clone generation, or should clone tapping be independent of destination playback state
- should input-chain monitoring be independently bypassable from recording source mode
- should the first implementation include an arp, or reserve richer sequence-generating effects for a later spec
- when both scale quantize and chord progression quantize are present, should the UI surface a stronger conflict warning than a generic stack warning
