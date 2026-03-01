# Direct UI Mapping Mode Spec

## Purpose

Define a follow-on mapping workflow that builds on the current discoverability direction:

- hover-to-footer status for desktop
- a dedicated toggleable inline overlay for mapping hints

The new mode lets the user enter a temporary mapping state, select an actionable UI element, capture the next input event, and create or replace a mapping for that element's canonical action.

This spec is grounded in the current repository state. `docs/handoff-summary.md` is not present in this worktree, so the baseline here comes from `README.md`, `docs/product-spec.md`, `docs/implementation-plan.md`, `docs/current-mappings.md`, `docs/architecture.md`, and the current `src/*` mapping and app code.

## Current Baseline

Current repo behavior relevant to this feature:

- The app already has a shared action boundary through `AppAction`, and keyboard, pointer, touch, and MIDI are expected to converge on that layer.
- The mappings page already exposes a row-based editor with `Overview` and `Write` modes, row selection, field selection, add/remove row, and MIDI learn for the selected row's source field.
- Persisted mappings are still row entries with `source_kind`, `source_device_label`, `source_label`, `target_label`, `scope_label`, and `enabled`.
- MIDI mappings currently resolve by matching a normalized incoming MIDI event to those stored row labels, then converting the row's target and scope labels into one or more `AppAction` values.
- The current quick mappings overlay is read-only and global, not an inline per-control mapping workflow.
- Pointer and touch handling is already centralized in `App::handle_pointer_down`, with per-page hit-testing methods returning app actions or direct state updates.

Implication:

- Direct UI mapping should reuse the existing canonical action model and the existing persisted mapping rows.
- It should not invent a second action namespace tied only to widget ids.
- It does need one new UI-facing descriptor layer so actionable elements can expose their canonical mapping target and scope in a consistent way.

## Goals

- Let the user start mapping from the control they want to affect, instead of starting from an abstract row in the mappings page.
- Reuse the current canonical action model and current mapping storage.
- Make scope resolution obvious when the same action can be global, active-track-relative, or absolute-track-specific.
- Keep the workflow usable on both pointer-hover desktop devices and touch-first devices.
- Prevent silent source conflicts that would currently dispatch multiple actions from one incoming MIDI event.

## Non-Goals

- Full keyboard learn or OSC learn implementation in this slice.
- A general-purpose remapping workflow for non-actionable decorative UI.
- Replacing the mappings page editor; direct mode should complement it.
- Solving every possible multi-action macro use case. This mode is for direct one-element-to-one-canonical-target mapping.

## User Value

The current mappings page is good for overview and field editing, but it requires the user to already know which canonical target label matches the control they want. Direct UI mapping reduces that translation burden:

- find control
- enter mapping mode
- select control
- perform next hardware input
- confirm replacement if needed

That fits the product direction in `docs/product-spec.md`: mappings should stay unified across control surfaces, and the app should expose routing and control state without guessing.

## Proposed UX Flow

### Entry Points

Support two entry points into the same direct mapping mode:

1. Mappings page primary entry
   - Add a `Direct Map` chip near the existing `Tap Mode` and `Tap Learn` controls.
   - Entering the mode from here keeps the mappings page as the current page, but enables cross-page target selection.
2. Inline discoverability overlay entry
   - When the mapping hints overlay is visible, include a clear `Map Control` affordance in the overlay/footer copy for actionable elements.
   - On desktop this can be activated from hover context.
   - On touch it must be reachable via tap without requiring hover.

Recommended action-level behavior:

- Add a dedicated `AppAction::ToggleDirectMappingMode`.
- `Escape` should cancel direct mapping mode without mutating mappings.
- Completing a mapping capture exits the mode by default.

### Mode States

The mode should be modeled as a short explicit state machine:

1. `Idle`
2. `Targeting`
   - The app highlights actionable controls and suppresses normal activation.
   - Footer/overlay copy says "Select a control to map".
3. `AwaitingInput`
   - A specific control is selected.
   - Footer/overlay copy says "Move a MIDI control now" and names the canonical target.
4. `ConflictResolution`
   - A source conflict or ambiguous replacement target was found.
   - The app presents explicit choices.
5. `Committed`
   - Mapping row created or replaced, user gets a short success confirmation, mode exits.

### Selecting Actionable UI Elements

Only controls with a canonical action target should be selectable.

Initial in-scope actionable elements:

- timeline transport chips
- timeline `Reset Song Loop`
- page tabs if the team wants page/overlay mapping exposed through direct mode
- routing page value controls and passthrough toggle
- mappings page mode toggle and learn/direct-map controls only if the team wants self-mapping of mapping UI
- per-track timeline header actions if and only if they already correspond to stable canonical actions

Out of scope for first pass:

- decorative labels
- dense timeline note bodies and region bodies
- passive device availability rows on MIDI I/O
- controls whose effect is not represented by a stable canonical action yet

Selection behavior:

- Desktop pointer: hover shows the hint text; click selects the actionable element instead of activating it.
- Touch: first tap while direct mapping mode is active selects the element and shows a pinned inline hint card near the selection; it must not rely on hover or footer-only messaging.
- Keyboard: if direct mapping mode is entered while focus is already on a page item, `Enter` may select the focused actionable item, but pointer/touch targeting remains the primary interaction.

### Canonical Target Resolution

Each selectable element must resolve to a canonical mapping descriptor:

- target label
- scope label
- optional preferred source kind default
- optional existing row reference when the element already corresponds to a unique editable row

Use existing target/scope labels wherever possible so persistence stays compatible:

- `Play/Stop` + `Global`
- `Track Arm` + `Active Track`
- `Track Arm` + `Track 3`
- `Passthrough` + `Active Track`
- `Select Track` + `Track 4`

Scope rules:

- Global controls always resolve to `Global`.
- Controls rendered for the current active-track context resolve to `Active Track`.
- Controls rendered inside an absolute track context resolve to `Track N`.
- Relative-only concepts such as next/previous track selection stay out of the first direct-mapping pass unless the UI element itself is explicitly relative.
- If a page exposes only the active track's routing controls, direct mapping from that page should resolve to `Active Track`, not the current absolute track number.

### Input Capture

First implementation scope:

- MIDI note learn
- MIDI CC learn

Capture behavior:

- After the target is selected, the next qualifying MIDI input becomes the proposed source.
- The capture should reuse the current MIDI learn normalization rules:
  - note source labels like `Note C2`
  - channel-qualified labels when needed
  - source device label set to the concrete input device used for learn
- The resulting mapping should be enabled immediately.

Recommended future-compatible rule:

- Generalize the current `mapping_midi_learn_armed` flag into a broader "mapping capture armed" state that can later support keyboard and OSC capture without another modal rewrite.

### Create vs Replace Behavior

Direct mode operates on the canonical action for the selected element, not on a preselected arbitrary row.

Replacement target selection rules:

1. If there is exactly one existing row with the same canonical target and scope and the same source kind being learned, replace that row in place.
2. If there is no matching row, create a new disabled-by-default draft row, populate it from the captured source, then enable it on commit.
3. If there are multiple rows with the same canonical target and scope:
   - show a compact chooser:
     - `Replace Row 1`
     - `Replace Row 2`
     - `Add New`
     - `Cancel`
   - default selection should be `Add New`, because multiple existing rows imply intentional multi-bind behavior.

This keeps the mode aligned with the current row model and avoids hidden row deletion.

### Source Conflict Rules

Current repo behavior allows one MIDI source to trigger multiple actions because all matching rows dispatch. Direct mode should not silently create that condition.

Define a normalized source identity as:

- source kind
- source device label, with `Any MIDI` treated as wildcard only at runtime, not during conflict review
- source label

Conflict rules on commit:

1. If the captured source exactly matches an existing enabled row with the same target and scope:
   - treat as idempotent
   - ensure the row is enabled
   - show `Already mapped`
2. If the captured source exactly matches an existing enabled row with a different target or scope:
   - enter `ConflictResolution`
   - show the current binding and the proposed binding
   - choices:
     - `Move Binding` (recommended): reassign the existing row to the newly selected target, or disable the old row if replacing another target row
     - `Keep Both`
     - `Cancel`
3. If the captured source matches a disabled row:
   - prefer reviving and updating that row instead of creating a duplicate disabled row.

Recommended product default:

- `Move Binding` should be the highlighted option because it matches user expectation for "map this control to that UI element" and avoids accidental multi-action dispatch.

### Success Feedback

On successful commit:

- flash the selected control highlight once
- show footer/overlay confirmation naming source, target, and scope
- select the resulting row on the mappings page state so the user can immediately inspect or refine it later
- exit direct mapping mode automatically

## Desktop vs Touch

### Desktop

- Hover is available, so actionable controls can advertise:
  - control name
  - canonical target/scope
  - whether a mapping already exists
  - shortcut to enter direct mapping from the current hint/overlay
- Click while in direct mapping mode selects, not activates.
- Right click is not required.

### Touch

- No hover dependency.
- Entering direct mapping mode should add visible hit targets and a pinned instruction strip/card.
- First tap selects the control.
- Second tap on the same selected control is not required for input learn; once selected, the app immediately enters `AwaitingInput`.
- Conflict and success messages must be rendered inline or in a compact modal card, not only in the window title or footer.

### Shared Rules

- Pointer/touch source differences should affect presentation only, not the canonical mapping result.
- Once the app is in `AwaitingInput`, normal pointer/touch activation remains suppressed until commit or cancel.

## Reuse of Existing Action Model

The current architecture requirement is correct: all surfaces should converge on `AppAction`.

Direct mapping should reuse that in two layers:

1. UI elements expose a `MappingDescriptor` derived from the same canonical action concepts already used by `mapping_entry_to_actions`.
2. Captured input still persists into the existing `MappingEntry` row format for now.

Recommended implementation shape:

- Keep `MappingEntry` persistence stable in the first pass.
- Add a small internal descriptor type for direct mapping, for example:
  - `target_label`
  - `scope_label`
  - `action_preview: Vec<AppAction>`
  - `existing_row_indexes: Vec<usize>`
- Derive that descriptor from hit-tested controls.

This avoids a large persistence migration while still getting the mode onto a more canonical footing.

## Scope Behavior Details

Scope behavior must match visible UI context:

- Timeline transport chips: `Global`
- `Reset Song Loop`: `Global`
- Active-track routing controls: `Active Track`
- Absolute per-track header buttons, if exposed: `Track N`
- Page navigation tabs:
  - if included, they should map as `Pages/Overlay` + `Global`
  - they should not create synthetic per-page scopes

Rules to avoid surprises:

- Do not resolve an on-screen active-track control into `Track N` just because track `N` happens to be active at selection time.
- Do not resolve an absolute per-track control into `Active Track`; the visible location already implies a concrete track.
- Keep `Relative` scope out of direct mapping unless the selected UI element is itself a relative command.

## Acceptance Criteria

1. The user can enter direct mapping mode from the mappings workflow without opening the mappings row editor first.
2. While the mode is active, actionable UI elements are visually distinguishable from non-actionable ones.
3. Desktop hover shows discoverability text for actionable controls; touch does not require hover.
4. Selecting a control resolves to a stable canonical target and scope that matches the current mapping row vocabulary.
5. The next MIDI note or CC input creates a new mapping when no matching target row exists.
6. The next MIDI note or CC input replaces a unique existing row for the same target and scope when appropriate.
7. If the captured source is already bound elsewhere, the app shows explicit conflict resolution instead of silently creating a duplicate active source.
8. On success, the resulting mapping is enabled and immediately works through the existing mapping dispatch path.
9. Canceling the mode leaves mappings unchanged.
10. The resulting mapping is visible and editable on the existing mappings page.

## Likely Code Touch Points

### `src/actions.rs`

- Add a direct-mapping mode toggle/cancel action.
- Optionally add confirm/cancel actions for conflict cards if the team wants them in the canonical action layer instead of local pointer-only UI handling.

### `src/pages.rs`

- Extend page/app UI state with a direct-mapping state machine instead of a single boolean.
- Generalize learn state from `mapping_midi_learn_armed` to a broader capture state.
- Track the selected mapping descriptor or pending replacement row list.

### `src/mapping.rs`

- Add helper functions for:
  - normalized source identity comparison
  - locating rows by target/scope
  - locating rows by source identity
  - create/replace/move-binding resolution
- Keep `MappingEntry` storage compatible in the first pass.

### `src/app.rs`

- Add the direct mapping mode UI, drawing, and footer/overlay messaging.
- Centralize actionable-control hit testing so each page can return a mapping descriptor, not only a direct action.
- Update pointer handling so direct mode intercepts activation and turns clicks/taps into selection.
- Expand MIDI learn capture into a shared direct-mapping capture path.
- On commit, update `self.mappings`, select the resulting row, and keep `sync_midi_inputs()` behavior coherent while capture is armed.

### Page-specific hit testing in `src/app.rs`

Likely helper additions:

- timeline actionable descriptor helpers around transport chips and loop reset
- routing actionable descriptor helpers for value/toggle fields
- optional page-tab descriptors for page switching bindings
- mappings-page entry affordances for starting/canceling direct mode

### Tests

Add focused tests for:

- descriptor resolution for global vs active-track vs absolute-track controls
- create-new mapping path
- replace-unique-row path
- source conflict detection
- idempotent remap of an already-mapped source
- touch/pointer selection behavior not triggering the underlying action while in direct mode

## Suggested Delivery Order

1. Add internal descriptor and state-machine types.
2. Implement direct mode entry, cancel, and target highlighting on one page only.
3. Reuse MIDI learn capture for selected descriptors.
4. Implement target-row replacement and source-conflict resolution.
5. Expand actionable coverage across timeline, routing, and optional page tabs.
6. Hook the final success path back into the mappings page selection state.

## Open Questions

- Should page tabs be directly mappable in the first pass, or should direct mode stay focused on transport and track controls?
- Should `Keep Both` remain available given the current multi-dispatch runtime behavior, or should the first pass enforce one active MIDI source binding per normalized source?
- Does the team want direct mapping to stay MIDI-only until keyboard and OSC learn exist, or should the UI copy say "next input" from day one while only MIDI is implemented underneath?
