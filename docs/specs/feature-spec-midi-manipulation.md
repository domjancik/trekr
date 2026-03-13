# Feature Spec: MIDI Manipulation

## Summary

This feature adds action-driven MIDI note selection and manipulation so note editing can be performed from keyboard shortcuts, MIDI controls, and later OSC/touch surfaces through the shared mapping model.

The first slice is intentionally playhead-centric rather than pointer-centric:

- selection starts from notes intersecting the playhead on the target track
- repeated or held actions grow, move, and reshape the current selection
- editing actions operate on the selected notes without requiring direct note dragging

This keeps note editing compatible with the current fixed-fit, performance-oriented UI model and the existing source-agnostic action architecture.

## Goals

- make note editing possible through mappable actions, not just future pointer gestures
- preserve the current select-first interaction model
- let hardware controllers perform usable note editing on the active track
- keep behavior deterministic in the fixed-fit timeline/detail panes

## Non-Goals

- freeform piano-roll mouse editing
- arbitrary lasso or rectangle note selection
- per-note velocity or CC lane editing in this slice
- detailed multi-parameter transform tools

## User Model

The user is editing notes on the active MIDI track, primarily in the detail column.

The playhead is the anchor for selection entry. When the user triggers note-selection actions, the app resolves candidate notes against the current playhead position on the target track. After at least one note is selected, follow-up actions can move the selection focus, expand or contract the selected set, and nudge selected notes in pitch or time.

Selection and manipulation should be available from:

- keyboard shortcuts
- MIDI note or CC mappings
- future OSC mappings

## Core Terms

- `focus note`: the note currently used as the reference for relative selection actions
- `selection anchor`: the stable edge used while extending or contracting a selection span
- `selection span`: the ordered range of selected notes on a track
- `playhead hit`: a note whose start/end range contains the current playhead tick, including notes already sustaining across the playhead

## Selection Rules

### Target Scope

Default target scope is the active track. The action model should still allow mapped variants that target an absolute track when needed.

### Select Notes At Playhead

Add an action that selects notes intersecting the playhead on the target track.

Expected behavior:

- if no modifier/add mode is active, the previous note selection on that track is replaced
- if add mode is active, playhead-hit notes are added to the existing selection
- if multiple notes overlap the playhead, all matching notes are eligible
- notes do not need to start exactly at the playhead; sustaining notes count as hits
- if no notes intersect the playhead, the selection is unchanged

This action should support hold behavior for controller mappings:

- pressing or engaging the action enters additive select mode
- while held, repeated note-at-playhead hits continue adding to the same selection
- releasing ends additive select mode without clearing the accumulated selection

For keyboard use, a discrete additive variant may also exist so the feature is not dependent on a hold-capable control surface.

### Select Next / Previous

Add actions to move selection focus to the next or previous note on the target track.

Expected behavior:

- with no current selection, next/previous selects the first resolvable note in that direction from the playhead
- with an existing selection, next/previous moves focus relative to the focus note
- without additive mode, the selection collapses to the newly focused note
- with additive mode, the newly focused note is added to the selection

Ordering rules:

- primary order is note start time
- pitch breaks ties for notes with the same start time
- if both start time and pitch match, preserve stable project order

### Select First / Last Of Selection

Add actions to focus the first or last note already inside the current selection span.

Expected behavior:

- these actions do not change membership by themselves
- they move the focus note to the earliest or latest selected note
- they are primarily used to define which edge later extend actions grow from

If there is no selection, these actions do nothing.

### Extend Selection

Add actions to extend the current selection:

- forward
- backward
- both

Expected behavior:

- `extend forward` adds the next note after the current trailing edge
- `extend backward` adds the previous note before the current leading edge
- `extend both` grows one note on each side when available
- the selection anchor stays fixed on the opposite side of the extension direction

If there is no selection, extend behaves like select-at-playhead first, then grows from there on subsequent invocations.

### Contract Selection

Add an action to contract the current selection span.

Expected behavior:

- contraction removes one note from the currently focused edge of the selection span
- if the span has more than one note, it shrinks toward the focus note
- if only one note remains selected, contract does nothing

The implementation should not depend on remembering extension history across editor-state persistence. A deterministic focus-edge rule is sufficient as long as it is documented and consistent.

## Manipulation Rules

### Nudge Selection In Time

Add actions to move the selected notes earlier or later in time.

Expected behavior:

- movement applies to every selected note on the target track
- when quantize is on, the default delta is one current quantize step
- when quantize is off, the default delta is the unsnapped base note-move step chosen for the editor
- negative movement clamps at tick zero unless later timeline rules allow earlier pickup space
- note durations are preserved
- moving notes should preserve relative spacing inside the selection

Mapped variants may later expose larger or smaller step sizes, but the canonical action should follow the current quantize toggle.

### Nudge Selection In Pitch

Add actions to move the selected notes up or down in pitch.

Expected behavior:

- movement applies to every selected note
- the default delta is one semitone
- resulting pitches clamp to the valid MIDI note range
- note timing and duration are preserved

Mapped octave variants can be added later as separate actions if needed.

## Interaction Constraints

- actions must route through the canonical action layer, not page-specific shortcut code
- note selection should persist per track when the active track changes
- the action set should include an explicit deselect-track-notes action so persistence is reversible from mapped controls
- selection state must be serializable with project/editor state if other timeline selections are already persisted
- selection behavior must remain deterministic across keyboard and MIDI-triggered control paths
- note selection should be visually obvious in the detail column and readable in the full column when density allows

## UI Expectations

The timeline/detail renderer should eventually communicate:

- selected notes
- focus note
- selection anchor or leading/trailing edge when extend/contract matters
- additive-select held state when active

The initial implementation does not need a new page. It belongs in the timeline workflow and should coexist with the existing active-track emphasis.

## Mapping Implications

The mappings system should expose note-editing actions as first-class targets, including:

- `Select Notes At Playhead`
- `Select Notes At Playhead Add`
- `Deselect Track Notes`
- `Select Next Note`
- `Select Previous Note`
- `Focus First Selected Note`
- `Focus Last Selected Note`
- `Extend Note Selection Forward`
- `Extend Note Selection Backward`
- `Extend Note Selection Both`
- `Contract Note Selection`
- `Nudge Selected Notes Earlier`
- `Nudge Selected Notes Later`
- `Nudge Selected Notes Up`
- `Nudge Selected Notes Down`

Hold-capable inputs should be able to map to additive selection behavior without requiring a separate modal editor state.

The action model should leave room to experiment with selection-targeted transforms in two forms:

- actions that operate on all currently selected notes across tracks
- actions that operate only on the selected notes for the addressed track

## Edge Cases

- overlapping notes at the same pitch and time should all be selected if they intersect the playhead
- empty tracks ignore note-selection and note-nudge actions
- selecting across non-adjacent notes is out of scope for this slice; the model is span-based
- nudging into collisions with unselected notes should not silently delete data
- switching the active track does not implicitly clear note selections on other tracks
- replace/overdub recording behavior must define whether an existing note selection is preserved, cleared, or suspended while recording

## Acceptance Criteria

- a user can select playhead-hit notes on the active track from a mapped action
- a user can keep adding notes to the same selection while holding an additive-select action
- a user can step selection forward and backward without using the pointer
- a user can extend and contract the selected note span predictably
- a user can transpose or time-nudge the selected notes through mapped actions
- the same canonical actions are available to keyboard bindings and MIDI mappings

## Open Questions

- what should the exact unsnapped base time-nudge step be when quantize is off
- should note-transform actions default to all selected notes across tracks, addressed-track selection only, or expose both forms side by side
- how should record/overdub flows affect existing per-track note selections during capture and commit
