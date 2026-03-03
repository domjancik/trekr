# Feature Spec: Undo

## Summary

`trekr` should gain undo/redo as a first-class part of the action-driven architecture.

The core requirement is:

- any user-issued action that changes meaningful app state should be reversible

To make that practical, the app should distinguish three state domains:

- `Timeline`: project content and timeline-adjacent editor state
- `Mappings`: mapping definitions and mapping-editor context
- `UI`: page navigation and other non-document interface state

The recommended shape is:

- V1 ships with canonical global `Undo` and `Redo`
- every undo entry is tagged with one domain: `Timeline`, `Mappings`, or `UI`
- the history model is built so domain-specific undo can be added later without redesigning the stack model

This avoids the biggest failure mode of naive undo systems: mixing unrelated transient UI movements with project edits in a way that feels random, while also avoiding three isolated histories that do not respect actual chronological work.

## Problem

The app already routes keyboard, MIDI, pointer, and internal triggers through shared `AppAction` commands, but state mutation remains direct. Once an action fires, the user has no structured way to reverse it.

That becomes a real problem in the current product shape:

- loop edits are fast and easy to overshoot
- note nudges and note-selection actions are iterative
- mapping edits are dense and easy to misconfigure
- page-local editor actions can strand the user in the wrong context after an accidental change

Because the app is action-first, undo should also be action-first. The user should not need separate mental models for keyboard, MIDI, or pointer changes.

## Goals

- provide reliable global undo/redo across current editable app surfaces
- keep undo behavior consistent regardless of action source
- support both document edits and editor-context changes where reversal matters
- keep room for later category-specific undo without invalidating V1
- treat recording commit as undoable
- keep history deterministic and serializable enough for persisted state flows

## Non-Goals

- selective arbitrary undo of an old entry in the middle of history
- perfect DAW-grade transport/session rewind in this slice
- undoing passive runtime updates such as playhead advance, Link refresh, or device scans
- implementing the feature in this change

## State Taxonomy

Undo gets much cleaner if the app stops treating every mutable field the same way.

### 1. Timeline Domain

This domain covers project-affecting state and timeline editor context:

- MIDI notes
- regions
- global song loop
- per-track loop regions and loop-enabled flags
- track arm, mute, solo, passthrough
- routing values
- record mode and loop-recording extension mode
- active track
- note selection and note focus/anchor state

Rationale:

- these states determine musical output, recording targets, or timeline editing context
- users reasonably expect these changes to undo together

### 2. Mappings Domain

This domain covers mapping content and mapping-editor context that should rewind with it:

- mapping rows
- mapping row enabled state
- mapping source kind/device/value
- mapping target and scope
- selected mapping row when caused by a mapping edit
- mapping learn arming when it directly participates in an edit flow

Rationale:

- a mapping change is its own class of work
- users often want to step backward through mapping edits without disturbing timeline work

### 3. UI Domain

This domain covers non-document interface state:

- current page
- page-local focus/selection state not owned by `Timeline` or `Mappings`
- mappings overlay visibility
- discoverability overlay visibility
- timeline flow mode
- MIDI I/O page list focus and selected device rows
- routing field focus

Rationale:

- these changes are still user intent
- they should be reversible if the requirement is "any action should be undoable"
- they should not be stored in the same undifferentiated stream as note edits without domain tagging

### 4. Session/Runtime State

This state is not part of undo history:

- playhead advancement from elapsed time
- transport tick accumulation
- hover state
- status messages
- Link peer/tempo snapshots arriving from runtime refresh
- MIDI device availability refresh
- in-progress held-note capture before record commit
- transient hold flags such as additive-note-selection hold

Rationale:

- these are runtime facts, not user-authored state transitions
- including them would flood history and make undo unusable

## Scope Rule

For this spec, "undoable action" means:

- an explicit user-triggered action that mutates `Timeline`, `Mappings`, or `UI` domain state

It does not mean:

- every passive state change caused by playback, hardware discovery, or external sync refresh

This rule is the practical interpretation of "any action should be undoable." Without it, plain playback would constantly write history entries just by running.

## Undo Model

## Canonical User Actions

Add canonical actions for:

- `Undo`
- `Redo`

Later optional actions:

- `Undo Timeline`
- `Redo Timeline`
- `Undo Mappings`
- `Redo Mappings`
- `Undo UI`
- `Redo UI`

The same actions must be available to keyboard bindings, MIDI mappings, and future OSC mappings.

## Entry Shape

Each committed undo entry should record:

- user-facing label
- domain: `Timeline`, `Mappings`, or `UI`
- source action
- before state for the affected domain slice
- after state for the affected domain slice
- transaction grouping metadata if relevant

Given the current app size, snapshotting the affected domain slice is the recommended implementation starting point. Full inverse-command modeling is not necessary in V1.

## Recommended History Architecture

Use:

- one global chronological log of committed entries
- one cursor per domain
- one global cursor for generic undo/redo

Each entry belongs to exactly one domain in V1.

Generic undo:

- walks the global log backward
- reverts the most recent committed entry regardless of domain

Domain-specific undo:

- walks only entries from that domain
- reverts the most recent entry in that domain

This is only safe if domain ownership is clear and overlapping mutations are avoided.

## Why Not Separate Independent Stacks Only

Three fully separate stacks with no shared chronology sound simple, but they break user expectations:

- a mapping edit made after a timeline edit is still later work
- generic undo becomes ambiguous
- redo becomes difficult once work interleaves across categories

The global log keeps chronology honest. Domain tags keep category-specific behavior possible.

## Why Not Global Undifferentiated History Only

A single untagged stack is simpler, but it causes poor UX:

- page flips can bury real edits
- mapping-page cursor changes mix with note nudges
- future category-specific undo becomes a rewrite

The spec therefore recommends a global log with domain tagging, not a blind monolithic stack.

## Behavior Rules

## Recording

Recording should not write history on every incoming note event.

Rules:

- `StartRecording` begins a session change but does not commit an undo entry yet
- `StopRecording` or `ToggleRecording` commit one compound `Timeline` entry if the take produced a real change
- if a recording attempt produces no region/note change, no history entry is committed
- undo of the committed entry removes the recorded result and restores replaced content where relevant

Replace-mode recording must restore removed notes/regions, not just delete the newly added take.

## Note Selection And Editor Context

Timeline selection actions should be undoable as `Timeline` entries:

- select notes at playhead
- add-to-selection
- next/previous note
- focus first/last selected note
- extend/contract selection
- deselect notes
- active-track changes

Rationale:

- these actions alter the user's editing context
- later timeline edit undo should not restore content but leave selection stranded in an unrelated state

## Loop, Track, Routing, And Transport Edit Flags

These are `Timeline` entries:

- song loop edits
- track loop edits
- track arm/mute/solo/passthrough
- routing field value changes
- record mode changes
- loop-recording extension toggle
- global loop enable toggle

These mutate project behavior and should undo as project-level edits.

## Mappings

These are `Mappings` entries:

- add mapping row
- remove mapping row
- mapping field value changes
- mapping enabled toggle
- MIDI learn commit that updates a row

Pure page navigation inside the mappings page is `UI`, not `Mappings`, unless the selection movement is inseparable from a committed mapping edit.

## UI

These are `UI` entries:

- page changes
- overlay toggles
- timeline flow changes
- MIDI I/O page focus changes
- routing page field focus changes
- mappings write-mode toggle if it changes interface mode only

The intent is literal reversibility of interface actions without polluting `Timeline` or `Mappings`.

## No-Op Rule

No history entry should be committed if an action produces no state change.

Examples:

- extending note selection when no additional note exists
- nudging selected notes when no note is selected
- trying to remove a mapping when the list is empty

## Transaction Rule

Some visible user commands should collapse into one undo step.

V1-required transaction cases:

- recording commit
- any future multi-field mapping learn commit
- any action that updates both a domain value and its required companion selection/context in the same intent

Optional later coalescing:

- repeated note nudges
- repeated loop nudges/resizes
- repeated page-item adjustments on the same field

V1 does not need aggressive coalescing beyond the required transaction cases.

## Redo Rule

Redo should behave conventionally:

- undo moves the relevant cursor backward
- redo reapplies the next entry if no new conflicting entry was committed
- committing a new entry clears redo for that same domain and any global future path beyond the current global cursor

## Options Considered

## Option A: Global Undo Only

Shape:

- one stack
- no categories
- undo/redo only

Pros:

- lowest implementation cost

Cons:

- UI churn can bury meaningful edits
- category-specific undo later is awkward
- poor long-term fit for the request

Decision:

- not recommended

## Option B: Fully Separate Category Stacks

Shape:

- independent `Timeline`, `Mappings`, and `UI` histories
- no canonical global order

Pros:

- domain-specific undo is easy

Cons:

- generic undo semantics are unclear
- interleaved work does not replay truthfully
- redo behavior becomes surprising

Decision:

- not recommended

## Option C: Global Chronological Log With Domain-Tagged Entries

Shape:

- one chronological history
- entries tagged by domain
- generic undo first
- optional domain-specific undo later

Pros:

- respects actual order of work
- supports generic undo cleanly
- leaves room for category-specific commands

Cons:

- requires cleaner domain ownership
- needs explicit decisions for mixed-context actions

Decision:

- recommended

## UX Recommendation

V1 user-facing behavior should be:

- ship `Undo` and `Redo`
- display the label and domain of the action that was undone/redone in the status area
- do not expose category-specific undo buttons yet
- internally tag every entry with a domain from day one

Later, if users want scoped undo, add:

- dedicated mapped actions for `Undo Timeline`, `Undo Mappings`, and `Undo UI`

Those should be convenience commands over the same underlying history model, not a second undo system.

## Implementation Notes

## Required Refactor Before Undo

The current code mixes project data, editor state, and runtime session state across `App`, `Project`, and `AppPageState`.

Before implementing undo, cleanly define snapshot boundaries for:

- `TimelineUndoState`
- `MappingsUndoState`
- `UiUndoState`
- non-undoable runtime/session state

Important likely refactor:

- move editor-only selection concerns out of persisted project data where that makes domain ownership clearer

At minimum, `active_track_index` should be deliberately classified rather than inherited as "project" merely because of current struct placement.

## Likely Code Touch Points

- `src/actions.rs`
- `src/app.rs`
- `src/project.rs`
- `src/mapping.rs`
- `src/pages.rs`
- `src/state.rs`

## Suggested Rollout

1. Add domain-tagged undo entry types and global/domain cursors.
2. Wire `Undo` and `Redo` actions into the canonical action layer.
3. Integrate `Timeline` mutations.
4. Integrate `Mappings` mutations.
5. Integrate `UI` mutations.
6. Add status reporting for last undo/redo.
7. Evaluate whether scoped undo commands are still necessary after real use.

## Acceptance Criteria

- a user can undo and redo timeline/project edits through canonical actions
- a user can undo and redo mapping edits through the same generic actions
- a user can undo and redo non-document UI changes such as page switches and overlay toggles
- passive runtime updates do not create undo history entries
- recording commit is undone as one timeline transaction
- no-op actions do not create history entries
- every committed entry has a user-facing label and domain tag
- the architecture permits later `Undo Timeline`, `Undo Mappings`, and `Undo UI` actions without replacing the history model

## Open Questions

- should `TogglePlayback` and explicit transport start/stop be treated as undoable `UI` actions, or remain outside history as session controls
- should mapping write-mode and MIDI-learn arming be `UI` or `Mappings`
- should repeated nudges coalesce in V1, or only after basic undo feels correct
- should undo history persist across app relaunch, or reset per session in the first implementation
