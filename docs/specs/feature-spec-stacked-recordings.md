# Feature Spec: Stacked Recordings

## Summary

Users should be able to switch a track between the current overlaid recording view and a stacked recording view.

In stacked view, committed recordings on the track are shown side by side as ordered recording lanes rather than drawn into one merged visual layer. The user can select an individual recording, mute/unmute it, and delete it without leaving the timeline.

This feature should follow the app's existing control standard:

- timeline interaction still resolves through canonical actions
- pointer, keyboard, and MIDI mappings can target the same recording-management actions
- the default timeline remains fixed-fit and performance-oriented

## Problem

The current track rendering shows recorded content overlaid in the same visual space. That is compact, but it hides the distinction between separate takes once multiple recordings overlap in time.

That creates three practical issues:

- a user cannot quickly tell how many distinct recordings exist on a track or in what order they were captured
- a user cannot target one recorded take for muting or deletion without using broader track-level actions
- overdub- and loop-heavy workflows accumulate content in a way that is visible musically but not manageable structurally

The missing piece is not clip launching. It is per-recording inspection and management inside the timeline.

## Goals

- let the user toggle a track between `overlay` and `stacked recordings` display
- show each committed recording as its own selectable lane in stacked view
- preserve recording order visually so older and newer takes are easy to distinguish
- allow per-recording mute and delete from the timeline
- keep the interaction action-driven so mappings and pointer input use the same command path
- avoid breaking the current compact overview when stacked view is off

## Non-Goals

- clip-launch or scene-based workflow
- freeform drag editing, resize, or move of recordings in this slice
- note-level editing inside a selected recording
- automatically separating one recording into multiple sub-clips
- redesigning track-level mute/solo/arm behavior
- changing record commit rules, quantize rules, or loop-wrap behavior

## Terms

- `recording`: one committed record pass on a track
- `recording clip`: the timeline-visible representation of one committed recording
- `overlay view`: the current mode where recordings share the same render space
- `stacked view`: a mode where recordings render in separate per-recording lanes on the same track

For this feature, `recording clip` is a management term for a committed recording in the timeline. It does not imply a separate clip-launch system.

## User Stories

- As a user recording multiple passes on one track, I can switch to stacked view and see each recording as a separate lane.
- As a user comparing takes, I can select a specific recording without guessing which notes belong to it.
- As a user auditioning alternatives, I can mute one recording while leaving other recordings on the same track audible.
- As a user cleaning up a track, I can delete one recording without clearing the whole track.
- As a user relying on mapped controls, I can trigger stacked-recording actions through the same action system used elsewhere in the app.

## Proposed UX

### 1. Track View Toggle

Each track should expose a display toggle:

- `Overlay`
- `Stacked`

Behavior:

- the toggle is per-track, not global
- default remains `Overlay`
- switching modes does not change playback data, only visualization and clip-targeting affordances
- if a track has zero or one recording, stacked mode may still render but should not waste excessive space

Recommended label in timeline chrome:

- `Rec View Ovr`
- `Rec View Stk`

Exact copy can be finalized during implementation, but the state should be explicit and compact.

### 2. Stacked Lane Layout

In stacked view, each committed recording on a track renders in its own lane.

Rules:

- lane order is recording order
- oldest committed recording appears first
- newest committed recording appears last
- the track loop/detail pairing remains intact; stacking happens inside the track's content area, not by changing the whole page structure
- each lane uses the same time scale as the parent track view
- a selected recording is visually distinct
- a muted recording is visually dimmed and clearly marked

If the track contains notes or regions that were not created by recording, V1 should keep them in the normal overlay layer and reserve stacking for committed recordings only.

### 3. Selection

The user should be able to select one recording clip at a time per active track.

Behavior:

- pointer hit testing may select a recording clip directly
- keyboard and MIDI control should use relative selection actions such as previous/next recording clip
- selection is contextual to the active track
- selecting a different track clears recording-clip selection unless that track already has a valid selected recording
- if the selected recording is deleted, selection should move to the next newer recording when possible, otherwise the previous one, otherwise clear

Selection is required before destructive clip actions.

### 4. Per-Recording Mute

Each recording clip should support independent mute state.

Behavior:

- muting a recording clip silences only content owned by that recording
- track mute still overrides clip mute because track mute is broader
- muted recordings remain visible in both overlay and stacked views
- muted state should be toggleable from pointer affordances and action-driven controls

Muted clip state is part of project state, not transient UI state.

### 5. Per-Recording Delete

The user should be able to delete the selected recording clip.

Behavior:

- delete removes the recording clip and the notes owned by that recording
- delete does not affect other recordings on the same track unless they share the same recording identity
- delete is scoped to the selected recording, not the whole track
- if no recording clip is selected, the delete-recording action is a no-op

Pointer affordances may expose a delete target, but the mutation must still route through the action layer.

## Interaction Model

### Action-Driven Requirement

Stacked recordings should not introduce direct state mutation from the renderer.

New controls should resolve into canonical actions first, following the current app pattern already used for transport, loops, mappings, and track state.

That means:

- pointer interaction dispatches actions
- keyboard shortcuts dispatch actions
- MIDI mappings dispatch the same actions
- status text and discoverability can describe these actions like any other command

### Recommended Action Shape

V1 should add dedicated actions for recording-clip management. Recommended baseline:

- `ToggleCurrentTrackRecordingView`
- `SelectPreviousRecordingClip`
- `SelectNextRecordingClip`
- `ToggleSelectedRecordingClipMute`
- `DeleteSelectedRecordingClip`

If implementation needs absolute addressing later, that should be added separately rather than replacing the active-track-relative baseline.

### Pointer Behavior

Recommended pointer behavior:

- click a recording clip to select it
- click the track's recording-view control to toggle overlay/stacked mode
- click a muted/active badge or compact action chip to toggle selected clip mute if space allows
- click a delete affordance only when the clip is already selected, to reduce accidental deletion

Pointer affordances should be available, but they are not the source of truth. They are one frontend for actions.

### Keyboard/MIDI Baseline

The exact default shortcuts can be finalized during implementation, but the feature should be designed so:

- view toggle can be triggered without pointer input
- recording-clip selection can move deterministically through recording order
- mute/delete can operate on the current selection
- mappings page can expose the new actions like any other action

## Data And Playback Rules

### Recording Identity

The current track model stores:

- `regions`
- `midi_notes`

That is not enough for per-recording mute/delete because notes from different recordings are flattened into a shared note list.

The implementation should therefore introduce stable recording ownership so one committed recording can be:

- rendered as one recording clip
- selected
- muted
- deleted along with its owned notes

Recommended rule:

- every committed recording receives a stable recording id
- the committed region and all notes produced by that record pass carry that id

### Ordering

Recording order should be commit order.

Rules:

- the first committed recording on a track gets the first lane
- each later committed recording appends after the previous ones
- deleting a recording closes the visual gap and preserves relative order of the survivors
- mute does not affect ordering

### Replace Mode

Replace mode already removes overlapping content during record commit.

For stacked recordings, replace mode should preserve a coherent ownership story:

- any content removed by replace should remove the affected recording ownership records too
- the newly committed recording becomes its own clip with its own recording id

If a replace pass partially removes an older recording, the implementation must choose one of two consistent behaviors:

1. delete the entire overlapped recording clip if any owned content is removed
2. split ownership and create derived recording clips

V1 should choose option 1 for simplicity unless code inspection during implementation shows that full-clip removal is materially worse for user expectation.

### Non-Recorded Content

Seeded demo notes or later imported/manual notes may exist without recording ownership.

V1 rule:

- only committed recordings participate in stacked recording management
- non-recorded notes continue to render in the base track layer
- mute/delete recording actions do not target unowned notes

## Visual Constraints

- stacked lanes must preserve fixed-fit rendering at the supported demo viewport
- the default page layout remains `song | loop detail` pairs per track
- stacked view must not require scrolling in V1
- lane headers or badges should stay compact and text-light
- selected and muted states must remain legible even on narrow track columns
- overlay view should remain visually unchanged when stacked view is off

If a track contains more recordings than can be comfortably displayed, the renderer may compress lane height, but selection and order must stay readable.

## Implementation Notes

Suggested implementation sequence:

1. add recording ownership to committed regions and notes
2. add per-track view state for `overlay` versus `stacked`
3. add clip selection state scoped to the active track
4. add actions and reducers for toggle/select/mute/delete
5. render stacked lanes from recording ownership groups
6. route pointer hit targets through the new actions
7. expose new actions to mapping/discoverability surfaces

Likely code touch points:

- `src/actions.rs`: new clip-management actions and labels
- `src/project.rs`: recording ownership, mute state, delete behavior, track view state
- `src/timeline.rs`: clip/ownership data structures if separated from project types
- `src/app.rs`: rendering, hit testing, action dispatch, selection visuals
- `src/mapping.rs`: mapping support for any new actions

## Acceptance Criteria

- A track can be toggled between overlay and stacked recording view.
- In stacked view, each committed recording on the track renders in its own lane.
- Lane order matches recording commit order.
- The user can select an individual recording clip from the timeline.
- The user can mute/unmute the selected recording clip without muting the whole track.
- The user can delete the selected recording clip without clearing unrelated track content.
- Pointer interactions for stacked recordings dispatch canonical actions rather than mutating state directly.
- The new actions are available to the same mapping/discoverability systems used by the rest of the app.
- Overlay view remains the default and continues to work for existing tracks.

## Scope Decision: Active-Track Relative First

V1 should make recording-clip actions active-track-relative rather than introducing absolute clip addressing.

Rationale:

- the current action model is primarily active-track-relative for timeline operations
- relative clip selection is easier to map onto keyboard and MIDI controls
- absolute clip ids would leak renderer/data-model detail into the action layer too early
- active-track-relative actions are enough to validate the feature before expanding the command surface

## Follow-On Work

Out of scope but naturally adjacent:

- duplicate selected recording clip
- reorder recordings manually rather than by commit order
- rename or color-tag recordings
- direct compare or solo-audition mode for recordings
- clip-level discoverability badges showing mappings for recording actions
