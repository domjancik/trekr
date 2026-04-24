# Feature Spec: Undo History Timeline Strip

## Summary

`trekr` should add a thin bottom-of-window undo history strip that visualizes recent undoable transactions from the existing persisted undo model.

The strip is a read-first UX surface for the current undo system, not a new history engine. It should make the current global and per-channel undo behavior legible by showing:

- recent transactions in chronological order
- undo domain ownership per transaction: `Timeline`, `Mappings`, `UI`
- whether a transaction is currently applied or currently undone
- the current global head and per-domain availability at a glance

The strip should reuse the existing `AppAction` boundary and current undo transaction model in `src/undo.rs`. It should not introduce ad hoc alternate undo commands, alternate history stores, or page-specific undo logic.

This spec also leaves room for a future integration with persisted points-in-time from `vk/095e-feature-spec-aut`, but that is explicitly a later extension layered on the same strip rather than part of V1.

## Problem

The current undo implementation works, but its visible UI surface is minimal:

- keyboard shortcuts exist
- mappings can target global and per-domain undo/redo
- the footer/status line shows the last undo/redo result

What is missing is a visible explanation of history shape.

That creates several UX gaps:

- users cannot see why scoped undo skips some transactions
- users cannot tell whether the most recent work was `Timeline`, `Mappings`, or `UI`
- users cannot easily see how much redo is available
- the persisted nature of undo history is invisible
- a future persisted-timepoint feature has no obvious place to surface chronology

A compact history strip solves this without changing the core action model.

## Goals

- visualize recent undo history without changing the current undo semantics
- make domain ownership visible and understandable
- show applied vs undone transactions clearly
- preserve one shared chronological history model
- support both desktop pointer and touch interaction
- reuse existing `AppAction` undo/redo commands and domain model
- provide a compatible visual foundation for future persisted timepoint markers

## Non-Goals

- rewriting undo storage from snapshots to event sourcing
- making arbitrary old entries directly clickable for selective undo in V1
- turning the strip into a full timeline editor
- exposing hidden runtime/session events that are intentionally excluded from undo
- implementing AUT/persisted-timepoint restore behavior in this change

## Grounding In Current Implementation

The current repository already provides the model this strip should visualize:

- canonical commands in `src/actions.rs`:
  - `Undo`
  - `Redo`
  - `UndoTimeline`
  - `RedoTimeline`
  - `UndoMappings`
  - `RedoMappings`
  - `UndoUi`
  - `RedoUi`
- mapping exposure in `src/mapping.rs`
- persisted undo model in `src/undo.rs`
- undo integration in `src/app/support/undo.rs`
- shell/footer status rendering in `src/app/shell/ui.rs`

The history strip should be treated as a new shell-level view over that existing system.

## User Experience Model

## Placement

V1 places the strip in the bottom shell/footer area as a thin horizontal band above or integrated with the current footer text row.

Requirements:

- visible on all main pages
- always uses the same geometry regardless of current page
- low vertical cost; it must remain a compact summary surface
- should not replace the existing footer text entirely; the text still carries precise last-action details

Recommended layout:

- footer text remains the lowest textual row
- history strip sits directly above it as a thin row of compact event chips

## Visual Language

Each visible transaction is rendered as a compact chip or segment with:

- domain color/family
- short label
- applied vs undone visual state
- current-history-head boundary

Recommended domain visual separation:

- `Timeline`: timeline family accent already used elsewhere in timeline surfaces
- `Mappings`: mappings page accent family
- `UI`: neutral shell/UI accent family

Applied vs undone:

- applied entries are solid/high contrast
- undone entries remain visible but dimmed/hollow/desaturated

Current head:

- the boundary between applied and undone history should be obvious
- a thin vertical caret, divider, or stronger gap is preferred over relying on color alone

## Labeling

Chip text should reuse the current user-facing transaction label from the undo system.

Examples:

- `Add Mapping`
- `Select Next Track`
- `Show Page`
- `Toggle Focused Track View`

If horizontal space is tight:

- truncate intelligently
- keep the strip legible before showing every full label
- full label can appear in hover/press detail state

## Scope Behavior

The strip visualizes one chronological history, not three separate histories.

Base behavior:

- all transactions appear in chronological order
- each transaction shows its owning domain
- multi-domain transactions, if present, render as mixed/compound markers rather than pretending to be single-domain

Because current scoped undo only applies to single-domain transactions, the strip should help explain that behavior:

- a `Timeline` scoped undo operates on the nearest visible applied `Timeline` transaction eligible for scoped undo
- it skips `Mappings`, `UI`, and mixed transactions
- the strip should make that skip behavior visually understandable

V1 recommendation:

- show one main chronological lane
- optionally group by subtle domain tint within that lane
- do not split into stacked lanes by default in V1

Optional expanded mode for later:

- show per-domain sublanes under the main lane
- keep the main global lane authoritative

This gives the future “multiple tracks depending on the channel” direction a clean upgrade path without forcing V1 into a taller shell.

## Interaction Model

## Desktop Pointer

Desktop pointer interaction should be lightweight and discoverable.

V1 desktop interactions:

- hover a chip to show full transaction label, domain, and applied/undone state in footer detail text
- click a chip to focus/inspect it visually only
- double-click is out of scope
- direct selective restore by clicking an old chip is out of scope

Desktop affordances may include small inline affordances near the strip for:

- global undo
- global redo
- scoped timeline undo/redo
- scoped mappings undo/redo
- scoped UI undo/redo

However, V1 should prefer action reuse over bespoke strip-local controls. If buttons are shown, they must dispatch the existing `AppAction` variants rather than introduce new logic.

## Touch

Touch must bias toward larger targets and fewer gestures.

V1 touch interactions:

- tap a chip to show its detail in the footer or a compact transient overlay
- tap dedicated undo/redo controls if present
- no drag-scrubbing through history in V1
- no long-press-only essential behavior

Because the strip is thin, touch hit targets may need:

- slightly taller invisible hitboxes than the painted strip
- fallback detail text rather than tiny tooltips

## Keyboard And Mapping Reuse

The strip does not create new primary commands.

The primary commands remain:

- keyboard bindings from `src/actions.rs`
- remappable targets from `src/mapping.rs`
- any future OSC routing through the same `AppAction` boundary

If the strip adds clickable buttons, those buttons must emit existing actions only:

- `AppAction::Undo`
- `AppAction::Redo`
- `AppAction::UndoTimeline`
- `AppAction::RedoTimeline`
- `AppAction::UndoMappings`
- `AppAction::RedoMappings`
- `AppAction::UndoUi`
- `AppAction::RedoUi`

## Conflict And Replacement Rules

## What Replaces What In The Strip

The strip is history-backed, so its content updates when:

- a new undoable transaction is committed
- a global undo/redo occurs
- a scoped undo/redo occurs
- old transactions are pruned by bounded history rules
- persisted undo history is loaded on startup

The strip must not fabricate events for:

- playback advancement
- Link refresh
- MIDI device scans
- hover changes
- temporary status messages
- in-progress recording before commit

## Capacity And Overflow

The underlying undo history is bounded. The strip only shows the most recent visible window of that bounded history.

V1 rules:

- show the newest transactions first in view terms, anchored so the current head is always visible
- older transactions fall off the left side of the strip before any alternative pagination UI is introduced
- no horizontal scroll interaction in V1

If the visible strip cannot show all recent transactions:

- preserve current-head visibility
- preserve at least a few items on both sides of the head when possible
- prefer abbreviated chips over adding another row

## Replacement By New Work

Current undo semantics already clear redo on new conflicting work. The strip should reflect that directly:

- if the user undoes entries, the undone tail remains visible as redoable
- if the user then commits new work, that redo tail disappears from the strip because it was invalidated by the history model

This is important: the strip is descriptive, not archival.

## Persisted Startup Behavior

Undo history already persists in a separate file.

On app launch with persisted state:

- load persisted undo history normally
- render the strip from the loaded history immediately
- do not visually distinguish "loaded from prior session" vs "created this session" in V1

Later AUT integration may add persistent timepoint markers or session separators, but V1 should stay simple.

## Relationship To Future Persisted Timepoints

The future AUT/persisted-timepoint feature can layer onto this strip by introducing an additional marker family.

Conceptual fit:

- undo transactions remain the dense local chronology
- AUT points become sparser anchored markers on the same time axis

Later extension ideas:

- thin vertical save/restore markers behind or above chips
- labeled persisted checkpoints
- hover/tap detail showing saved timestamp or label
- restore actions that are explicitly separate from undo/redo

Important rule:

- AUT markers must not masquerade as undo entries
- undo/redo and restore-to-timepoint are different mental models and must remain visually distinct

## UX Flow Examples

## Flow 1: Mapping Edit Then Mapping Undo

1. User opens `Mappings`
2. User enters write mode
3. User adds a mapping row
4. Strip shows a new `Mappings` chip labeled `Add Mapping`
5. User triggers `Undo Mappings`
6. That chip moves to the undone side of the current-head boundary
7. Footer text confirms `Undid Add Mapping (Mappings)`

## Flow 2: Interleaved UI And Timeline Work

1. User switches from `Timeline` to `Mappings`
2. Strip records a `UI` chip
3. User returns and nudges a loop or track selection
4. Strip records a `Timeline` chip after the `UI` chip
5. User triggers `Undo Timeline`
6. The most recent eligible `Timeline` chip is undone
7. The earlier `UI` chip remains applied
8. The strip makes this skip behavior visible

## Flow 3: Global Undo Through Mixed Recent Work

1. User performs `UI`, then `Mappings`, then `Timeline` work
2. Strip shows all three in order
3. User hits global `Undo`
4. The rightmost applied chip moves into undone state regardless of domain
5. Repeating global undo walks backward through the visible chronology

## Flow 4: Redo Invalidated By New Work

1. User undoes two recent chips
2. Strip shows them dimmed on the redo side
3. User performs a new mapping edit
4. Redo tail disappears
5. New `Mappings` chip appears at the head

## Acceptance Criteria

## Functional

- a compact history strip is visible on all main pages
- the strip renders from the existing undo history model
- each rendered transaction communicates domain and applied/undone state
- the current history head is visually obvious
- the strip updates after global undo/redo
- the strip updates after scoped undo/redo
- the strip updates after new undoable work commits
- the strip reflects redo invalidation after new work
- persisted history appears in the strip after persisted startup

## Interaction

- desktop hover reveals fuller detail without mutating history
- desktop click/touch tap on a chip reveals detail without performing selective undo
- if strip-local undo controls are included, they dispatch existing `AppAction` variants only
- touch targets are usable without requiring pixel-precise taps on tiny painted geometry

## Scope Clarity

- the strip makes it understandable that scoped undo traverses eligible domain transactions only
- the strip does not show session/runtime-only events as undoable chips
- mixed-domain transactions, if present, are visually distinct from single-domain transactions

## Likely Code Touch Points

Primary likely touch points for implementation:

- `src/undo.rs`
  - expose read helpers for visible history slices if needed
  - avoid changing core semantics unless strictly required for view support
- `src/app/support/undo.rs`
  - provide view-facing helpers on `App` for current history state and visible-window derivation
- `src/app/types.rs`
  - add shell view-model structs for strip chips and interaction state if needed
- `src/app/shell/ui.rs`
  - render the strip in the footer/shell area
  - handle hover/detail rendering
- `src/app/shell/layout.rs`
  - reserve space and define footer/strip bounds cleanly
- `src/app/input.rs`
  - wire pointer/touch hit testing for strip-local interactions if added
- `src/actions.rs`
  - likely no new canonical undo actions needed
  - only add actions if a separate “toggle strip mode” or inspect behavior is later desired
- `README.md`
  - update if the visible app shell changes materially
- `artifacts/screenshots/*.png`
  - refresh if the strip materially changes tracked screens

Secondary likely touch points:

- `src/app/support/labels.rs`
  - domain labels or abbreviated chip text helpers
- `src/app/shell/mod.rs`
  - if shell integration helpers need to stay out of `app/mod.rs`

## Recommended Implementation Notes

- keep the strip as a pure projection of undo state
- avoid adding history-specific mutable UI state unless needed for hover/focus
- prefer deriving a compact display model per frame from `UndoHistory`
- if performance becomes a concern later, cache the derived visible chip list inside app shell state
- do not couple strip rendering to any single page module
- keep `src/app/mod.rs` thin; shell rendering belongs with shell ownership and undo projection belongs with undo ownership

## Open Questions For Later Specs

- whether V2 should add a taller expandable mode with per-domain lanes
- whether the strip should expose compact inline scoped undo/redo buttons or remain purely descriptive in V1
- whether persisted AUT markers should appear in the same band or a parallel band
- whether the strip should show session separators across relaunches
- whether clicking the latest applied chip should perform global undo as a convenience, or remain inspect-only
