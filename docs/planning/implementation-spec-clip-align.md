# Implementation Spec: Clip Align

## Purpose

This document translates `docs/specs/feature-spec-clip-align.md` into an implementation-focused plan grounded in the current codebase.

It is intentionally scoped to the current app shape described in `docs/planning/handoff-summary.md`:

- action-driven input model
- selected committed recording clips on the timeline
- keyboard/MIDI/pointer support for non-drag interactions
- global transport tempo with Ableton Link integration

## Implementation Outcome

Deliver a first-pass clip-align workflow that lets the user:

1. select a committed recording clip on the active track
2. open a compact align panel
3. choose source span, target length, destination scope, and apply mode
4. align only that clip's owned notes/region to the chosen loop length
5. optionally commit a new global BPM when local tempo authority allows it

## Explicit V1 Decisions

### Chosen Defaults

- source start: `First Note`
- source end: `Start Of Last Note`
- target length: `4 bars`
- destination: `Track Loop`
- apply mode: `Fit + Tempo`
- enable target loop on apply: `true`

### Supported Source Span Modes

- start: `Clip Start` | `First Note`
- end: `Start Of Last Note` | `Last Note End` | `Clip End`

### Supported Target Length Modes

Use fixed musical lengths first, represented in ticks from current PPQN:

- `1 bar`
- `2 bars`
- `4 bars`
- `8 bars`

Assume 4/4 for V1 because there is no broader time-signature model in the current transport docs/code.

### Supported Destination Modes

- `Track Loop`
- `Song Loop`

### Supported Apply Modes

- `Fit Only`
- `Fit + Tempo`

## Why This Fits The Current Code

The current repository already has the right primitives:

- `Track.recording_clips`, `selected_recording_clip_id`, and clip ownership in `src/project.rs`
- canonical action dispatch in `src/actions.rs`
- pointer hit targets and timeline chrome handling in `src/app.rs`
- mapping exposure in `src/mapping.rs`
- transport tempo state in `src/transport.rs`
- Link runtime and tempo commit path in `src/link.rs` and `src/app.rs`

The missing pieces are:

- align-specific UI state
- clip-native metadata for stable tempo derivation
- clip-owned time projection logic
- action plumbing and discoverability text

## Data Model Changes

## 1. `RecordingClip` metadata

Extend `RecordingClip` in `src/project.rs` with stable native-reference fields.

Recommended fields:

- `native_start_ticks: u64`
- `native_end_ticks: u64`
- `native_duration_ticks: u64`
- `native_capture_tempo_bpm: u16`

Behavior:

- populate these when the clip is first committed
- do not rewrite them during later align operations
- preserve them through save/load

Rationale:

- `native_duration_ticks` and `native_capture_tempo_bpm` provide a stable basis for BPM derivation
- `native_start_ticks` / `native_end_ticks` help future diagnostics and repeated-align UX

## 2. Align panel state

Add app-level transient UI state, likely in `src/app.rs`, for the current align editor session.

Recommended struct:

- `track_index: usize`
- `clip_id: u64`
- `selected_field: ClipAlignField`
- `draft: ClipAlignSettings`
- `preview: ClipAlignPreview`
- `blocked_reason: Option<String>`

Recommended enums/structs:

- `ClipAlignField`
- `ClipAlignSourceStartMode`
- `ClipAlignSourceEndMode`
- `ClipAlignTargetLength`
- `ClipAlignDestination`
- `ClipAlignApplyMode`
- `ClipAlignSettings`
- `ClipAlignPreview`

This state should be modal-ish but lightweight, similar to current field-driven flows.

## 3. Optional status memory

Optionally remember the last-used align settings in app session state, but do not persist them in project save data for V1 unless implementation is already trivial.

## Align Math / Transform Rules

## 1. Resolve the source span

Given a selected clip and its owned notes:

- `Clip Start` => `clip.region.start_ticks`
- `First Note` => minimum owned note `start_ticks`, fallback `clip.region.start_ticks`
- `Start Of Last Note` => maximum owned note `start_ticks`, fallback `clip.region.end_ticks()`
- `Last Note End` => maximum owned note `end_ticks()`, fallback `clip.region.end_ticks()`
- `Clip End` => `clip.region.end_ticks()`

Validation:

- source end must be `>` source start
- otherwise preview is invalid and apply is blocked

## 2. Determine destination span

Destination start:

- `Track Loop` => active track `loop_region.start_ticks`
- `Song Loop` => project `loop_region.start_ticks`

Destination length:

- based on selected target bar count
- `bar_ticks = ppqn * 4`
- `target_length_ticks = bar_ticks * bar_count`

Destination end:

- `destination_start + target_length_ticks`

## 3. Project clip-owned events

Only mutate notes/regions owned by the selected clip id.

Projection formula for each owned event boundary inside the selected source span:

- `normalized = (tick - source_start) / source_length`
- `mapped_tick = destination_start + normalized * destination_length`

Implementation note:

- use integer-safe helper math, likely centralized in `src/timeline.rs` or a new helper section in `src/project.rs`
- use saturating/clamped rounding rules and document them in tests

Recommended event handling:

- clip region becomes exactly the destination span
- owned notes fully outside the source span are deleted
- owned notes crossing source boundaries are clipped to the source span before projection
- projected notes shorter than 1 tick clamp to 1 tick
- note pitch and velocity remain unchanged
- clip muted state remains unchanged

## 4. Tempo derivation for `Fit + Tempo`

Goal: preserve original performed speed.

Use clip-native metadata rather than the already-aligned region size.

Recommended formula:

- `native_beats = native_duration_ticks / ppqn`
- `target_beats = target_length_ticks / ppqn`
- `new_bpm = native_capture_tempo_bpm * (target_beats / native_beats)`

Equivalent simplified form:

- `new_bpm = native_capture_tempo_bpm * target_length_ticks / native_duration_ticks`

Guardrails:

- if native duration is zero, block `Fit + Tempo`
- clamp result to existing transport-safe bounds (`20..400` currently implied by `src/app.rs` Link sync path)
- if tempo authority is external, do not commit the BPM

## Action Surface

Add new canonical actions in `src/actions.rs`.

Recommended set:

- `OpenSelectedRecordingClipAlign`
- `CloseRecordingClipAlign`
- `ApplyRecordingClipAlign`
- `SelectPreviousClipAlignField`
- `SelectNextClipAlignField`
- `AdjustClipAlignFieldBackward`
- `AdjustClipAlignFieldForward`

Optional if implementation wants explicit target-length shortcuts later:

- `SetClipAlignTargetLength1Bar`
- etc.

For V1, field-based navigation is enough.

## Keyboard / Pointer / Mapping Behavior

### Keyboard

Recommended initial default bindings:

- `Shift+Enter` or another free combo: open align for selected clip
- reuse existing field-navigation conventions while panel is open:
  - `Shift+Left` / `Shift+Right` => previous/next field
  - `Q` / `E` => adjust field
  - `Enter` => apply
  - `Escape` => cancel

Before assigning a final shortcut, confirm conflicts in `src/actions.rs`.

### Pointer / Touch

In `src/app.rs` timeline header controls:

- show `ALIGN` chip when a clip is selected
- clicking/tapping opens the panel
- field rows can be tapped to select
- stepper/segmented controls adjust the current value
- `Apply` and `Cancel` buttons dispatch canonical actions

### Mappings

In `src/mapping.rs`:

- expose at least `Open Selected Recording Clip Align` and `Apply Recording Clip Align`
- optionally expose field next/previous and adjust backward/forward

This keeps the feature controllable from hardware without inventing a special path.

## UI Layout Proposal

Render the align UI as a compact modal card anchored near the timeline header rather than a full-page mode.

Recommended content:

- title: `Clip Align`
- target clip summary: track name + clip id or lane order
- fields:
  - `Start`
  - `End`
  - `Length`
  - `Dest`
  - `Mode`
  - `Loop`
- preview lines:
  - source span ticks
  - resulting BPM or `Tempo Locked`
- actions:
  - `Apply`
  - `Cancel`

This should be visually lightweight and consistent with current compact timeline chrome.

## Discoverability / Status Text

Add footer/discoverability support in `src/app.rs` / `src/actions.rs` label helpers for:

- `Clip Align`
- `Apply Clip Align`
- `Adjust Clip Align Field`

Recommended status examples:

- `Clip Align: First Note -> Start Of Last Note, 4 bars, Track Loop, Fit + Tempo`
- `Clip Align blocked: external tempo authority active`
- `Clip Align blocked: source span is empty`

## Implementation Sequence

## Phase 1: Core data + math

1. extend `RecordingClip` with native metadata
2. populate metadata during `Track::commit_take`
3. add helper functions to:
   - gather owned notes for a clip
   - resolve source span from settings
   - project note/region timing into destination span
   - derive tempo preview/result
4. add unit tests in `src/project.rs` and/or `src/timeline.rs`

Exit criteria:

- clip align math works from tests without any UI

## Phase 2: Actions + reducers

1. add new `AppAction` variants
2. add labels/default-binding placeholders
3. add app-level align-panel state
4. implement action handling for open/close/field adjust/apply

Exit criteria:

- app can open a draft align session and apply it from tests or direct reducer calls

## Phase 3: Timeline UI

1. add `ALIGN` chip when a clip is selected
2. render align card/panel
3. wire pointer hit targets
4. show preview/blocked status

Exit criteria:

- desktop pointer and keyboard can complete the full flow

## Phase 4: Mapping exposure

1. expose align actions in mapping target lists
2. confirm direct mapping/discoverability text

Exit criteria:

- align open/apply can be mapped like other timeline actions

## Phase 5: Polish / persistence validation

1. verify save/load for native clip metadata
2. verify behavior with Link enabled/disabled
3. verify stacked and overlay views both remain coherent after align
4. update docs/README if control surface changed materially during implementation

## Acceptance Tests To Add

## Project/model tests

- clip commit stores native metadata
- `First Note` / `Start Of Last Note` / `Last Note End` resolve correctly
- invalid span blocks apply
- align only mutates owned notes/regions for selected clip
- notes outside source span are removed
- notes crossing source span are clipped then projected
- projected note lengths clamp to at least 1 tick
- clip muted state survives align
- track note selection clears after align
- `Track Loop` destination updates track loop only
- `Song Loop` destination updates song loop only
- `Fit Only` keeps tempo unchanged
- `Fit + Tempo` derives expected BPM
- re-align uses native metadata rather than already-warped clip size

## App/UI tests

- open align action fails gracefully with no selected clip
- open align action auto-targets the only clip on active track when unambiguous, if implemented
- apply is blocked with external tempo authority when mode is `Fit + Tempo`
- align panel closes after successful apply
- pointer hit target dispatches canonical action rather than mutating directly

## Edge Cases

- clip with no owned notes: span modes fall back to clip boundaries
- one-note clip with `Start Of Last Note`: invalid zero-length span unless user chooses `Last Note End` or `Clip End`
- polyphonic final chord: `Start Of Last Note` resolves to the latest onset in the clip
- overlapping clips on the same track: only selected clip content moves
- selected destination start beyond current content is allowed
- muted track + clip align still edits data even though playback is muted

## Likely File Touch Points

- `src/actions.rs`
- `src/app.rs`
- `src/project.rs`
- `src/timeline.rs`
- `src/mapping.rs`
- `README.md` if shipped controls change
- `docs/specs/feature-spec-clip-align.md`

## Out Of Scope For This Implementation

- automatic phrase-length guessing
- arbitrary beat counts or free numeric entry
- drag-based trim handles in the timeline
- simultaneous multi-clip align
- per-track local tempos
- audio clip warping
