# Feature Spec: Timeline Control Contexts And Direct MIDI FX Editing

## Summary

This spec defines a focused control model for the `Timeline` page so users can work directly inside timeline-resident control areas without immediately jumping to `Routing`.

The first target is the per-track MIDI FX bands already shown on the timeline:

- `Input FX` band above the track pair
- `Output FX` band below the track pair

The control model should generalize to other timeline-local contexts over time, including:

- track timeline/content editing
- input FX
- output FX
- stacked recordings
- future note-edit, loop-edit, and modulation contexts

The design must stay aligned with the current repository architecture:

- action-driven input boundary
- active-track-relative default behavior
- compact fixed-fit timeline rendering
- keyboard, pointer/touch, and MIDI mappings all resolving through the same `AppAction` layer

This spec is focused on timeline controls and context switching. It does not replace the full `Routing` page editor.

Related docs:

- `docs/planning/handoff-summary.md`
- `docs/dev/architecture.md`
- `docs/specs/product-spec.md`
- `docs/specs/feature-spec-midi-track-effects.md`
- `docs/specs/feature-spec-stacked-recordings.md`

## Problem

The repository now shows MIDI FX in the timeline, but editing still depends on the `Routing` page.

That creates several usability gaps:

- timeline-visible FX are informative but not yet manipulable in-place
- users cannot stay in musical context while selecting, toggling, adjusting, or reordering effects
- there is no explicit timeline notion of `control context`, so future editable timeline regions risk using inconsistent controls
- multi-parameter effects need a compact display model that fits the timeline without becoming a second full inspector page

The product already relies on consistent action reuse across keyboard, MIDI mappings, and pointer/touch. Timeline-resident editing should follow the same rule.

## Goals

- define a reusable `timeline context` model for moving between editable areas
- make timeline FX directly selectable and editable in place
- keep controls as consistent as possible across timeline contexts
- preserve active-track-relative behavior by default
- support compact one-line-per-effect rendering with room for two visible parameters
- support overflow/scroll behavior when an effect exposes more than two parameters
- keep `Routing` as the full-detail editor while making the timeline the fast editor
- leave room for iconography without requiring icons for the first implementation

## Non-Goals

- replacing the `Routing` page entirely
- building a modal pop-up editor for every effect in the first slice
- implementing modulation routing UI in this slice
- graphical curve editing, envelopes, or piano-roll editing of FX parameters
- solving all future timeline contexts in code now; this spec only establishes the shared interaction pattern

## Current Repository Grounding

Current implemented behavior relevant to this spec:

- `Timeline` uses paired per-track columns in vertical time (`song | loop detail`)
- per-track MIDI FX are rendered in dedicated timeline bands
- pointer interaction on the FX bands currently routes to `Routing`
- `Routing` already exposes slot, kind, enabled, and value editing for input and output FX
- timeline interaction already has contextual actions for recordings, focus, loops, and track state
- stacked recordings already demonstrate timeline-local selection and action reuse

Current code areas that shape this feature:

- `src/app.rs`
  - timeline drawing
  - timeline pointer hit testing
  - routing field selection
  - active-track and selected-item page state
- `src/actions.rs`
  - canonical action definitions
- `src/pages.rs`
  - page fields and navigation models
- `src/midi_fx.rs`
  - effect kinds, parameter labels, and value adjustment behavior
- `src/project.rs`
  - per-track MIDI FX state
- `src/ui.rs`
  - layout helpers used by timeline and routing rendering

This means the feature should extend the existing action/state model rather than create a separate per-widget editing system.

## Terms

- `timeline context`: the currently active editable region type within the timeline
- `context family`: a logical region such as `Track Timeline`, `Input FX`, `Output FX`, or future contexts
- `context item`: the currently selected row or element inside a context
- `primary parameter`: the most important inline-adjustable parameter for an effect
- `secondary parameter`: the second visible inline parameter when space permits
- `parameter window`: the visible subset of an effect's parameters when it has more than two

## Proposed Timeline Context Model

### Context Families

The timeline should expose a small, reusable set of context families.

Initial target families:

- `Track Timeline`
- `Input FX`
- `Output FX`

Future-compatible families:

- `Loop Controls`
- `Stacked Recordings`
- `Note Selection / Note Edit`
- `Track Modulation`
- `Track Automation`

### Context Switching

Users should be able to move between contexts predictably.

Recommended baseline behavior:

- `Left` / `Right`
  - moves between tracks when timeline content is focused
  - moves between sibling contexts on the active track when a timeline control context is focused
- `Up` / `Down`
  - moves between items inside the current context
- `Enter`
  - activates or drills into the selected item
- `Q` / `E`
  - adjusts the selected item or selected parameter, matching the current app editing convention
- pointer/touch tap
  - directly focuses the tapped context and item

Recommended context order on a track pair:

1. `Input FX`
2. `Track Timeline`
3. `Output FX`

This ordering matches the musical signal model and the current visual layout.

### Consistency Rule

Every timeline context should try to reuse the same abstract operations:

- `move context`
- `move item`
- `activate`
- `toggle`
- `adjust backward`
- `adjust forward`
- `secondary adjust` or `reorder`

The label or on-screen affordance may differ, but the interaction shape should stay recognizable.

## MIDI FX Row Layout

Each MIDI FX instance should occupy one compact row in the timeline band.

### Row Structure

Left-to-right layout:

1. `identity zone`
   - effect title, icon, or both
2. `primary parameter zone`
3. `secondary parameter zone`
4. `overflow hint / more-indicator` when more parameters exist

### Identity Zone

The identity zone should be left-anchored.

Preferred variants to explore:

- `title only`
  - e.g. `TRANSPOSE`, `FILTER`, `ARP`
- `icon + title`
  - compact procedural icon plus short title
- `icon only` is allowed only if recognizability remains high in fixed-fit conditions

Recommendation:

- implement `icon + short title` as the target design
- allow `title only` as the first implementation if icon work is not ready
- do not ship `icon only` as the only representation in V1

The current short labels from `midi_fx.rs` can seed the compact text fallback.

### Parameter Zones

The right side of the row is parameter-focused.

Rules:

- primary parameter is always shown when the effect has any adjustable parameter
- secondary parameter is shown when the effect has a meaningful second inline parameter and space permits
- parameter display should prefer compact values already derivable from the effect model, such as:
  - `+12`
  - `120%`
  - `C`
  - `T1`
  - `48-84`

Examples:

- `TRNSP | +12`
- `FILTER | 48-84 | 7 ON`
- `ARP | 120t | UP`
- `CLONE | T2 | POST`

### More-Parameters Window

When an effect has more than two parameters:

- the timeline row shows only a two-parameter window at once
- the row includes a visible overflow hint such as:
  - `>`
  - `2/4`
  - `+2`
- users can scroll the parameter window left/right without changing context family

This should mirror the current stacked-recording horizontal clip-window idea:

- a compact visible slice
- explicit indication that more content exists off to the side
- deterministic controls for paging/scrolling

Recommended parameter-window rules:

- window size: `2`
- window start index stored per selected effect row only, not globally across all tracks
- overflow indicator only shown when hidden parameters remain

## Direct Timeline FX Editing

### Editable Operations

From the timeline, users should be able to:

- select an effect row
- toggle enabled/bypassed state
- switch effect kind
- adjust visible parameters
- reorder effects within the chain
- jump to `Routing` for full editing when needed

### Selection Model

Selection should be explicit.

Rules:

- each active track may have one selected timeline context item at a time
- when the user enters `Input FX`, one FX row is selected if any row exists
- when the user enters `Output FX`, one FX row is selected if any row exists
- switching tracks preserves selection if the destination track has a valid item at the same relative position; otherwise clamp to the nearest valid item
- if the selected effect is deleted or replaced by `None`, selection moves to the nearest remaining row, otherwise clears

### Row States

Each row may visually express:

- `selected`
- `enabled`
- `bypassed`
- `warning`
- `has hidden parameters`
- `reorder target` while moving

### Toggle

Toggling should be available without leaving the timeline.

Recommended behavior:

- `Enter` on a selected row toggles enable/disable when not in a deeper row-edit submode
- pointer/touch may also support tapping a compact power/bypass chip within the row
- timeline toggle must dispatch the same underlying effect-enable action used elsewhere

### Kind Switching

Kind switching should remain possible from the timeline, but treated as a stronger edit than parameter adjustment.

Recommended behavior:

- selected row + secondary action enters `Kind` edit submode
- `Q` / `E` cycles effect kind in that submode
- pointer/touch may use compact left/right arrows or a dedicated kind chip on the selected row

Kind switching should use the same replacement semantics already defined in the MIDI track FX spec:

- changing kind replaces the instance payload with defaults for the new kind
- slot position is preserved
- enabled state should be preserved when practical

### Parameter Adjustment

The selected row should expose parameter adjustment inline.

Recommended behavior:

- default timeline FX editing adjusts the visible primary parameter with `Q` / `E`
- a row-level subfocus can move between visible parameter slots:
  - `param 1`
  - `param 2`
  - `more`
- pointer/touch may select the left or right parameter zone directly
- if only one parameter exists, all adjustment targets collapse to the primary parameter

### Reordering

Reordering should be supported directly in the timeline because chain order matters musically.

Recommended behavior:

- selected row enters `Move` submode
- `Up` / `Down` swaps with previous/next occupied slot
- moving into an empty slot is allowed if the implementation models fixed slots rather than dense rows
- pointer/touch may use drag later, but V1 should use discrete move controls or action-based reordering only

## Shared Action Model Reuse

The timeline should not mutate MIDI FX directly from rendering code.

Required rule:

- pointer, keyboard, MIDI mappings, and future touch gestures all resolve through canonical `AppAction` values

Recommended new action families:

- context navigation
  - `SelectPreviousTimelineContext`
  - `SelectNextTimelineContext`
  - `SelectPreviousTimelineContextItem`
  - `SelectNextTimelineContextItem`
- timeline FX
  - `SelectTimelineInputFxRow(index)` or relative row actions
  - `SelectTimelineOutputFxRow(index)` or relative row actions
  - `ToggleSelectedTimelineFxEnabled`
  - `AdjustSelectedTimelineFxParameterBackward`
  - `AdjustSelectedTimelineFxParameterForward`
  - `SelectPreviousTimelineFxParameter`
  - `SelectNextTimelineFxParameter`
  - `ScrollSelectedTimelineFxParameterWindowBackward`
  - `ScrollSelectedTimelineFxParameterWindowForward`
  - `CycleSelectedTimelineFxKindBackward`
  - `CycleSelectedTimelineFxKindForward`
  - `MoveSelectedTimelineFxEarlier`
  - `MoveSelectedTimelineFxLater`
  - `OpenSelectedTimelineFxInRouting`

Exact naming may differ, but the core requirement is shared action reuse.

## Scope Behavior

### Default Scope

Timeline FX editing should be `Active Track` scoped by default.

Reason:

- that matches the current timeline interaction model
- it keeps keyboard and MIDI mapping semantics predictable
- it aligns with existing track-relative editing in recordings and note selection

### Absolute Scope

Absolute-track scope may still be available through mappings.

Recommended rule:

- the new timeline-FX actions should support the same mapping-scope expansion pattern already used elsewhere
- `Track 1`, `Track 2`, etc. should remain possible through the mappings system where implementation cost is reasonable
- pointer/touch continues to target the clicked track directly, then dispatches the same action against that track context

## Conflict, Replacement, And Overflow Rules

### Conflicts

Timeline direct editing should honor the same conflict rules already established in `feature-spec-midi-track-effects.md`.

That means:

- duplicate effects are allowed
- likely-confusing combinations warn instead of auto-rewriting
- harmonic conflicts such as stacked quantizers should keep warning states rather than silently replacing each other

### Replacement

Changing effect kind from the timeline:

- replaces only the selected row instance
- does not affect neighboring rows
- preserves row order
- resets parameters to defaults for the new kind

### Overflow

When the row cannot show all identity and parameter content:

- parameter values take precedence over long titles
- title may shorten to the existing compact label
- icon may remain while title truncates
- overflow indicator must remain visible if hidden parameters exist

## Pointer / Touch / Desktop Differences

### Desktop Keyboard

Desktop keyboard should remain the most complete direct-edit path.

Recommended baseline:

- context switch
- row select
- toggle
- parameter adjust
- kind cycle
- reorder
- open in routing

### Pointer / Mouse

Mouse should support:

- selecting the row by clicking it
- selecting parameter zones directly
- toggling enabled state by clicking a compact toggle affordance
- opening `Routing` with double-click or explicit deep-edit affordance

Drag-to-reorder is optional and should be deferred unless it can be done without making the fixed-fit layout unstable.

### Touch

Touch should prioritize larger deterministic targets.

Recommended touch rules:

- tap row to select
- tap selected row again to activate the most likely edit target
- use left/right chips for parameter stepping rather than relying on hover-only affordances
- long-press may open `Routing` or row options, but should not be required for core editing
- reorder in V1 should use explicit move controls, not drag

## Visual Hierarchy Rules

- context family should be obvious before row-level detail
- selected row should have stronger contrast than enabled-only state
- enabled/bypassed should be distinguishable even when the row is not selected
- primary and secondary parameter zones should align consistently across rows
- the `more` hint should live at the far right edge so hidden-parameter affordance is predictable
- iconography, if present, should not reduce text legibility below the current bitmap-font baseline

## Relationship To Routing

The `Routing` page remains the full-detail editor.

Timeline direct editing should be optimized for:

- fast musical adjustment
- quick toggle and reorder
- staying in the timeline

The `Routing` page remains appropriate for:

- full slot inspection
- rare or high-risk edits
- parameter sets that exceed compact timeline affordances
- future richer editors such as arp-pattern or harmonic configuration

## Acceptance Criteria

1. Timeline exposes explicit editable contexts at least for:
   - `Track Timeline`
   - `Input FX`
   - `Output FX`
2. A user can switch between those contexts without leaving the timeline.
3. In the timeline, a user can select a specific FX row on the active track.
4. The selected FX row can be toggled enabled/bypassed from the timeline.
5. The selected FX row can adjust at least its primary parameter from the timeline.
6. Effects with two meaningful compact parameters can show both inline.
7. Effects with more than two parameters show a visible overflow indicator and support deterministic parameter-window scrolling.
8. The selected FX row can be reordered from the timeline without opening `Routing`.
9. Timeline operations dispatch through reusable actions rather than direct renderer mutation.
10. Pointer/touch interaction can select a track's input or output FX row directly.
11. Mapping scope behavior remains consistent with existing active-track-relative defaults and optional absolute-track mapping support.
12. Conflict and replacement behavior remains aligned with the MIDI track FX spec.

## Likely Code Touch Points

- `src/actions.rs`
  - new canonical timeline-context and timeline-FX actions
- `src/app.rs`
  - timeline selection state
  - timeline context switching
  - FX row hit testing
  - keyboard handling for context/item/parameter movement
  - row drawing and selected/edit-submode rendering
  - dispatch to existing FX adjust/kind/toggle helpers
- `src/pages.rs`
  - page-field/state extensions if timeline context selection is modeled alongside other page selections
- `src/project.rs`
  - persistent or semi-persistent per-track timeline-FX selection state if selection should survive page switches/state saves
- `src/midi_fx.rs`
  - explicit primary/secondary parameter metadata
  - compact inline parameter labels
  - ordered parameter descriptors for windowed display
- `src/ui.rs`
  - row layout helpers for identity/parameter/overflow zones

## Deferred / Open Design Notes

- whether iconography ships in the first implementation or as a later visual pass
- whether parameter-window position should persist per effect instance or only for the currently selected row
- whether `Track Timeline` and `Stacked Recordings` should share one context family or split into separate families once note editing lands
- whether a small context badge or breadcrumb should appear in the footer/status bar while editing inside the timeline
- whether direct kind switching should be enabled in V1 timeline editing or deferred to `Routing` while toggle/adjust/reorder ship first

## Current Direction Updates

The current agreed direction for implementation is:

- include `kind switching` in timeline direct editing
- include parameter-window `scrolling` in timeline direct editing
- prefer an overflow indicator that can also behave like a compact `scrollbar`, aligned with the stacked-recordings visual language when space allows
- persist timeline FX context, selected row, selected field, and parameter-window position rather than treating them as purely transient
- keep the code shape modular by preferring feature- or module-oriented files for shared timeline-FX logic instead of growing a single monolithic app file further

Icon work should be explored in parallel, but timeline direct editing should not wait on final icon assets. The text-first row layout remains the required fallback.
