# Feature Spec: Clip Align

## Summary

This spec proposes a post-recording **clip align** workflow for turning a freely recorded MIDI take into a usable loop and arrangement anchor **after** capture, even when the user did not know the BPM in advance.

The recommended V1 shape is:

- act on a **selected committed recording clip** on the active track
- trim the musical span to a chosen source window
- project that span onto a chosen loop length
- optionally update the **global tempo** so playback preserves the original performed speed
- keep the workflow action-driven and compatible with the current timeline, mapping, and touch model

This stays aligned with the current product direction in `docs/specs/product-spec.md`: `trekr` is a **linear timeline / loop-region** tool, not a clip-launch or custom per-track timeline system.

## Grounding In Current Repo

This proposal is intentionally based on the current docs and code:

- `docs/planning/handoff-summary.md` says the app is already **action-driven**, keyboard/MIDI-first, and does **not** yet support pointer timeline editing.
- `docs/specs/product-spec.md` says tracks use **linear regions** and **loop ranges**, not scene launching or free-floating clip workflows.
- `docs/specs/feature-spec-stacked-recordings.md` adds stable **recording clip ownership**, active-track-relative clip actions, and timeline clip selection/mute/delete.
- `src/project.rs` already persists `RecordingClip`, per-track `recording_view`, `selected_recording_clip_id`, and per-clip ownership on notes/regions.
- `src/actions.rs`, `src/mapping.rs`, and `src/app.rs` already route timeline commands through canonical `AppAction` values and expose them to keyboard, pointer, and mappings.
- `src/transport.rs` and `src/link.rs` show that tempo is currently a **global transport concern**, so any tempo-preserving clip align feature must respect that authority model.

Because of that existing shape, clip align should be treated as a **clip-to-loop transform** plus an optional **project tempo commit**, not as a new local timeline system.

## Problem

Current recording works well when the user already knows the loop length and tempo context.

It is weaker for this workflow:

1. play or record a sequence by feel
2. decide afterward what musical length it should represent
3. make it loop cleanly
4. derive a usable project tempo from the take
5. build the rest of the arrangement around it

The missing UX is not “custom timelines on tracks.”
The missing UX is a safe way to say:

- “this recorded clip is really a 2-bar or 4-bar phrase”
- “start the loop from here”
- “fit it cleanly”
- “set tempo so it still plays at the original performed speed”

## Design Goals

- support unknown-BPM-first recording
- preserve the current linear timeline and loop model
- reuse current recording clip selection rather than inventing a separate editor object
- keep the feature deterministic and action-driven
- work without drag editing in V1
- support both desktop and touch with the same underlying actions
- avoid destructive side effects on unrelated clips unless explicitly chosen

## Non-Goals

- custom per-track tempo maps or independent track timelines
- audio warping/time-stretch in this slice
- heuristic-heavy auto-detection as the only path
- freehand drag-to-warp interaction in V1
- clip launching or scene workflow

## Options Considered

### Option A: Per-Track Custom Timelines

Let each track have its own local timing base.

**Rejected for V1.**
This conflicts with the current product and architecture docs, which keep tempo global and treat track loops as local ranges on top of one transport.
It would also complicate mappings, playback, Link, and future audio support.

### Option B: Automatic Phrase Detection From Repeated Notes

Infer the loop by detecting a repeated first note or similar musical marker.

**Useful as a future assist, not as the primary UX.**
It is too fragile for polyphonic clips, pickups, held notes, and phrases that do not end on the same pitch.

### Option C: Explicit Clip Align Tool On Selected Recording Clip

The user selects a clip, chooses the source span and target phrase length, previews the resulting loop/tempo, then applies it.

**Recommended.**
It matches the current app model: selected clip, field-based adjustment, explicit apply, and canonical actions.

## Recommended UX

### Entry Point

Clip align should operate on the **selected committed recording clip**.

Entry should be available from:

- a header-level `ALIGN` chip when a recording clip is selected
- a keyboard shortcut on desktop
- a mappable action target
- touch tap on the same `ALIGN` chip

If no clip is selected:

- if the active track has exactly one committed recording clip, the action may target it directly
- otherwise the action is a no-op with status guidance: select a recording clip first

### Align Panel / Popover

Invoking clip align opens a compact field-based panel, similar in spirit to the current mappings write flow.

Recommended fields:

1. `Source Start`
   - `Clip Start`
   - `First Note` (recommended default)
2. `Source End`
   - `Clip End`
   - `Start Of Last Note` (recommended default)
   - `Last Note End`
3. `Target Length`
   - `1 bar`
   - `2 bars`
   - `4 bars`
   - `8 bars`
   - later: arbitrary beat count
4. `Destination`
   - `Track Loop` (recommended default)
   - `Song Loop`
5. `Apply Mode`
   - `Fit + Tempo` (recommended default)
   - `Fit Only`
6. `Also Enable Loop`
   - `On` by default for the chosen destination scope

The panel should show a preview summary:

- source span ticks
- target loop length
- resulting BPM when `Fit + Tempo` is selected
- warning when tempo authority prevents auto-commit

Recommended interpretation:

- `Start Of Last Note` is the default because it better matches loop creation for repeating phrases such as drum patterns
- `Last Note End` remains available as an explicit option for clips where the note tail should be included in the aligned phrase length

Exact source-span semantics:

- `Clip Start` uses the selected clip region start tick
- `First Note` uses the earliest start tick among notes owned by the selected clip; if the clip owns no notes, fall back to `Clip Start`
- `Start Of Last Note` uses the latest note start tick among notes owned by the selected clip; if the clip owns no notes, fall back to `Clip End`
- `Last Note End` uses the latest note end tick among notes owned by the selected clip; if the clip owns no notes, fall back to `Clip End`
- `Clip End` uses the selected clip region end tick
- if the resolved end is not after the resolved start, the panel should block apply and show an invalid-span warning

## Core Behavior

### What “Align” Means

On apply, the app performs these steps atomically on the target clip:

1. resolve the source span from the selected clip
2. trim or normalize the clip to that span
3. remap the owned note and region timing so the span fills the chosen target length
4. place the aligned start at the chosen destination loop start
5. update the chosen loop range to the target length
6. if `Fit + Tempo` is selected, update global tempo so the phrase still takes the same real time as the original performance

For V1, “remap” should mean **affine time projection inside the selected span**:

- any owned event at the source start maps to destination start
- any owned event at the source end maps to destination end
- owned events inside the span keep their relative proportional position
- owned events completely outside the chosen source span are removed from that clip
- notes crossing the source-span boundary are clipped to the boundary before projection

### Recommended Defaults

The default should optimize for “I recorded a phrase and now I want it to become the loop”:

- start at `First Note`
- end at `Start Of Last Note`
- target length should be the **closest supported bar count at the current project tempo**
- destination `Track Loop`
- apply mode `Fit + Tempo`
- enable the chosen loop if it was off

This trims dead air before the first played note, treats the final onset as the loop boundary, and makes the first suggestion feel musically plausible relative to the current tempo instead of always assuming `4 bars`.

Target-length suggestion rule:

- resolve the source span first using the current start/end settings
- measure the real-time duration that span would take at the **current project tempo**
- compare that duration against the supported loop lengths at the same tempo: `1`, `2`, `4`, and `8` bars
- preselect the closest match
- if two choices are equally close, prefer the shorter one
- if the span is invalid, fall back to the persisted last-used target length, otherwise `4 bars`

## Tempo Model

### Why Tempo Must Be Part Of The Feature

Without a tempo update, shrinking or stretching the clip to a musical loop length changes the performed feel.
That fails the main user goal.

### Tempo Rule

When `Fit + Tempo` is chosen, the app should derive the new global BPM from the clip’s **captured performance duration**, not from the current post-edit region size.

That means implementation should persist enough clip metadata to preserve a stable native reference, such as:

- capture tempo at commit time
- native captured span before align edits
- or a direct real-time duration value

This metadata should stay with the clip so re-aligning the same clip later does not compound error.

Recommended V1 metadata:

- `native_start_ticks`
- `native_end_ticks`
- `native_duration_ticks`
- `native_capture_tempo_bpm`

These fields are enough to support stable tempo derivation, repeat align operations, and future UI copy such as “original 3.27 beats at 96 BPM”.

### Link / Tempo Authority

`src/link.rs` and `docs/dev/architecture.md` make tempo authority important.

V1 rule:

- if local transport owns tempo, `Fit + Tempo` can commit BPM immediately
- if Link or another authority currently owns tempo, `Fit + Tempo` is disabled or downgraded to preview-only with clear status text
- `Fit Only` remains available

The feature should not silently fight external tempo authority.

## Scope Behavior

### Clip Scope

V1 is **active-track-relative first**, following `docs/specs/feature-spec-stacked-recordings.md`.

- the target is the selected recording clip on the active track
- absolute clip addressing is out of scope for the first slice
- mapped controls may later add absolute track variants, but not renderer-level clip ids as user-facing targets

### Loop Scope

Two destination scopes are enough for V1:

- `Track Loop`
- `Song Loop`

Recommended default: `Track Loop`.

Reason:

- it matches the existing per-track loop workflow
- it lets the user stabilize one performance first
- it avoids surprising full-project changes unless explicitly chosen

### Relationship To Stored Loops

Clip align should not automatically store or overwrite stored loop slots.
That should remain a separate explicit action.

## Conflict And Replacement Rules

### Other Clips And Notes

Aligning one recording clip should only transform content owned by that clip.

- do not delete unrelated recording clips
- do not delete unowned/manual notes
- overlapping results are allowed, because the current timeline already supports layered content

### Existing Destination Content

If the chosen destination loop already contains other material:

- clip align still applies
- no automatic clearing happens
- status text should clarify that other content remains in place

A later “replace destination content” mode can be explored separately, but it should not be implicit.

### Selected Note State

Because note selection is already scoped and clip-aware in stacked view, clip align should clear note selection on the affected track after apply.
This avoids stale selection indexes after remapping.

### Muted Clips

If the selected clip was muted before align, it remains muted after align.

### Replace Record Mode

Transport `Replace` mode affects **record commit**, not post-record align.
Clip align should not inherit replace-style destructive semantics.

## Desktop vs Touch

### Desktop

Recommended baseline:

- select clip in timeline
- press `Align` shortcut or click `ALIGN`
- use field selection and `Q` / `E` style adjustments, or direct click, to edit options
- press `Enter` to apply
- press `Escape` to cancel

### Touch

Because pointer timeline editing is not implemented and small screens favor tap-confirm flows:

- tap clip
- tap `ALIGN`
- use large next/previous or segmented controls for each field
- tap `Apply` or `Cancel`

### Shared Rule

No drag-to-resize or drag-to-warp behavior is required in V1.
Both desktop and touch should use the same explicit field-based model and the same canonical actions.

## Action Model Reuse

The feature must stay action-driven.

Recommended new actions:

- `OpenSelectedRecordingClipAlign`
- `CloseRecordingClipAlign`
- `ApplyRecordingClipAlign`
- `CycleRecordingClipAlignFieldNext`
- `CycleRecordingClipAlignFieldPrevious`
- `AdjustRecordingClipAlignBackward`
- `AdjustRecordingClipAlignForward`

If implementation prefers reusing existing generic field-navigation actions while a modal panel is open, that is also acceptable, but the behavior must still flow through canonical actions rather than direct widget mutation.

Mappings implications:

- the open/apply actions should be mappable
- direct parameter editing from MIDI is optional but should be possible through the same action layer
- discoverability/footer text should describe the current align action and selected parameters like other timeline controls

## Visual / Status Expectations

The timeline should communicate:

- which clip is the align target
- that align mode is open
- the target length and destination scope
- the resulting BPM preview
- whether the tempo commit is blocked by Link or other tempo authority

V1 does not need full ghost-note preview rendering inside the track canvas.
A textual or compact numeric preview in the panel is enough.

If the selected clip is shown in stacked view, the selected lane highlight should remain visible behind the align panel so the target stays obvious.

## Data Model Expectations

Likely additions to clip data:

- stable capture tempo or duration metadata
- optional remembered last align settings per app session or globally
- an internal representation of source trim offsets used for align

The important rule is that repeated align operations should have a stable reference for “original speed.”

## Likely Code Touch Points

- `src/actions.rs`
  - new canonical actions
  - labels / shortcuts / discoverability strings
- `src/mapping.rs`
  - expose align actions to mappings and direct mapping surfaces
- `src/project.rs`
  - clip metadata for native duration / capture tempo
  - clip align transform logic on owned notes and regions
  - loop-scope update rules
- `src/timeline.rs`
  - helper math for span normalization and tick remapping if kept separate
- `src/app.rs`
  - panel state
  - rendering of `ALIGN` affordance and preview
  - pointer hit targets
  - action application and status text
- `src/transport.rs`
  - helper math for BPM derivation if centralized there
- `src/link.rs`
  - tempo-authority guardrails when Link is active
- persistence/state fixtures
  - serialization for any new recording clip metadata

## Acceptance Criteria

- A user can invoke clip align for the selected committed recording clip on the active track.
- A user can choose source span, target length, destination scope, and apply mode without drag editing.
- Applying align remaps only the selected clip’s owned notes and region.
- Applying align can update the target track loop or song loop as chosen.
- `Fit + Tempo` preserves the clip’s original performed speed by deriving a new global BPM from clip capture metadata.
- When tempo authority is external, the UI clearly blocks or downgrades the tempo-commit path instead of silently fighting it.
- Keyboard, pointer, touch, and mappings all reach the feature through canonical actions.
- Applying align does not delete unrelated clips or unowned notes.
- Re-aligning a clip does not rely on the already-warped region size as the only source of tempo truth.

## Follow-On Ideas

Out of scope for the first implementation but worth keeping in view:

- heuristic suggestions such as “repeat of first note detected; suggest 2 bars”
- “create stored loop from aligned result” shortcut after apply
- compare multiple target lengths before commit
- optional pickup preservation presets
- audio clip align / warp using the same conceptual workflow later
