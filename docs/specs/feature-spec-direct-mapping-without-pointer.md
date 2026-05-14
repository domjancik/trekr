# Feature Spec: Direct Mapping Without Pointer

## Summary

This spec defines a keyboard-first direct-mapping targeting flow that works without mouse hover or pointer selection.

It is a follow-on to the current shipped direct-mapping baseline in `docs/specs/direct-ui-mapping-mode-spec.md` and is grounded in the current repo state:

- action-driven input boundary in `src/actions.rs` and `docs/dev/architecture.md`
- current mappings vocabulary in `docs/dev/current-mappings.md`
- current direct-mapping state and capture flow in `src/app/direct_mapping_ui.rs`, `src/app/input.rs`, and `src/app/types.rs`
- current discoverability target model in `src/app/discoverability_ui.rs`, `src/app/timeline/ui.rs`, `src/app/routing_ui.rs`, and `src/page_widgets.rs`
- current product and handoff shape in `docs/specs/product-spec.md` and `docs/planning/handoff-summary.md`

The current implementation already supports:

- `F8` direct mapping mode entry
- pointer/touch target selection for supported controls
- next-input capture for MIDI note, MIDI CC, and keyboard keys
- row reuse and row selection after commit

The missing piece is target selection without a pointer. This spec fills that gap.

## Problem

Today, direct mapping is only practically targetable by pointer/touch hit testing:

- `DirectMappingMode` only distinguishes `Inactive`, `Targeting`, and `AwaitingInput`
- `handle_direct_mapping_pointer_down` is the only shipped target-selection path
- `direct_mapping_targets()` already exposes selectable controls, but keyboard users cannot traverse or jump across them

That leaves a usability hole for:

- keyboard-first desktop use
- hardware-controller-first setups where touching a mouse breaks flow
- touchless stage/live use on small displays
- future kiosk or focused-track workflows where visible targets should be navigable by actions, not pointer geometry alone

## Goals

- Let the user select a direct-mapping target with keyboard or mapped actions, without needing a mouse.
- Reuse the current direct-mapping target model rather than inventing a separate accessibility tree.
- Keep the final mapping result identical regardless of whether the target was chosen by pointer, directional navigation, or hint jump.
- Preserve current create/replace/conflict semantics from direct mapping.
- Work cleanly across desktop, touch, and controller-driven flows.

## Non-Goals

- Full accessibility/focus redesign for every page.
- A general-purpose keyboard focus model for all UI interactions.
- Replacing the existing mappings-page row editor.
- Replacing pointer targeting where pointer use is convenient.
- Solving note-body or region-body targeting in the timeline.

## Current Baseline

The current repository already gives this feature a strong foundation:

- `AppAction` is the canonical control surface boundary.
- `DiscoverabilityTarget` already describes actionable page controls in action terms, plus allowed mapping scopes.
- `DirectMappingTarget` is already derived from discoverability targets and canonical mapping labels.
- Direct mapping already keeps capture armed across successive commits by returning to `Targeting`.
- Keyboard capture already reserves `Escape` and `F8` for cancel instead of capturing them as mapping sources.

Important current limitations:

- `direct_mapping_tab_targets()` currently returns no tab targets.
- `page_discoverability_targets()` only exposes direct-mapping targets on `Timeline` and `Routing` today.
- `StatusState.hovered_target` is pointer-centric, not navigator-centric.
- There is no persisted or transient "current direct-mapping target index" while in `Targeting`.

## Design Principle

Direct mapping without a pointer should reuse the same target list that pointer direct mapping already uses.

That means the feature should not introduce a second set of target ids or page-specific shortcut tables. Instead:

1. pages expose actionable controls through discoverability targets
2. direct mapping derives canonical mapping descriptors from those targets
3. a keyboard/controller navigator chooses among those targets
4. existing capture, replacement, and commit logic handles the rest

This keeps the final mapping semantics consistent across all selection methods.

## Proposed UX Model

### Entry

Entry stays the same as the current shipped baseline:

- `F8`
- mappings-page `Direct Map` affordance
- any future inline `Map Control` affordance

Once direct mapping is active, the user gets three equivalent targeting methods:

1. pointer/touch target selection
2. directional navigation
3. hint jump

### Targeting Presentation

When `DirectMappingMode::Targeting` is active, actionable controls should be exposed in two layers:

1. **Directional highlight**
   - exactly one target is considered the current navigator target
   - it receives the strong active outline
2. **Explicit jump hints**
   - each visible actionable target can render a short label such as `A`, `B`, `C` or `AA`, `AB`
   - these labels are only shown while the user is in the explicit jump submode
   - jump hints should use the same clear chip treatment as discoverability overlays: high-contrast background, explicit border, and off-body placement that does not cover the control's own label text

Recommended default:

- always show the active highlight
- show hint labels only after the user explicitly enters jump mode

This keeps the baseline understandable while still supporting a Vimium-like jump path with an unmistakable mode change.

## Targeting Methods

### 1. Directional Navigation

Directional navigation is the baseline no-pointer path.

Recommended controls while `DirectMappingMode::Targeting` is active:

- `Left` / `Right` / `Up` / `Down`: move to the nearest eligible target in that direction
- `Tab` / `Shift+Tab`: move to next/previous eligible target in stable reading order
- plain key press: map that key immediately to the current highlighted target
- `Enter`: explicitly arm the current target and advance to `AwaitingInput` for the next non-reserved input event
- `Escape` / `F8`: cancel direct mapping

Recommended spatial rule:

- use actual rendered target rectangles from `DirectMappingTarget.hit_rect`
- prefer the nearest target whose center lies in the requested direction
- use distance plus directional alignment, not raw list order, for arrow-key movement

Recommended fallback rule:

- if no good geometric candidate exists in that direction, do nothing rather than wrapping unpredictably
- `Tab` remains the explicit wrap-safe sequential navigation path

### 2. Hint Jump

Hint jump is the fast dense-page path, inspired by Vimium.

Behavior:

- while in `Targeting`, the user can press `/` to enter a temporary jump submode
- visible actionable targets render short labels
- typing the label retargets to that target
- a plain non-jump key press outside jump mode must not be consumed as a jump prefix, because those keys are reserved for direct key mapping to the current highlighted target

Recommended first-pass scope:

- visible targets only
- letters only; no punctuation or modifiers
- no cross-page hidden targets

Recommended label rules:

- deterministic per render pass
- stable while the visible target set is unchanged
- prefix-free so no shorter exact label can block a longer one
- fixed-width for the current visible target count:
  - up to 26 visible targets: `A` ... `Z`
  - more than 26 visible targets: `AA`, `AB`, ... `AZ`, `BA`, ...

This should reuse the repo's existing lookup mindset from `src/app/mapping/lookup.rs`: a lightweight transient selection mode with explicit cancel behavior and no persistent text field state.

### 3. Pointer/Touch Selection

Pointer/touch selection remains valid and should behave exactly as today.

The no-pointer feature is additive, not replacement.

## Target Ordering and Navigation Graph

The navigator needs a stable ordering in addition to geometric movement.

Recommended target ordering:

1. page tabs, if direct mapping expands to include them
2. top transport strip controls
3. page-local primary controls from top-to-bottom, then left-to-right
4. secondary micro-controls after the primary controls within the same page region

Implementation note:

- each page target producer should be able to return targets in a stable semantic order
- the app can still compute arrow-key neighbors from geometry on top of that order

This gives:

- predictable `Tab` traversal
- predictable first target when entering `Targeting`
- consistent hint-label assignment

## Suggested Reuse of Existing Action Model

The feature should stay action-driven.

### Reuse first

Prefer reusing existing canonical page-navigation actions where it stays understandable:

- `SelectPreviousPageItem`
- `SelectNextPageItem`
- `ActivatePageItem`
- `CancelCurrentMode`

Those actions already express navigation/activation intent and fit the architecture rule from `docs/dev/architecture.md`.

### Add only what current actions cannot express

Arrow-style direct-target movement is more spatial than the current page-item model. If current page-item actions are too row-centric, add a small direct-mapping-only action family such as:

- `SelectDirectMappingTargetNext`
- `SelectDirectMappingTargetPrevious`
- `SelectDirectMappingTargetLeft`
- `SelectDirectMappingTargetRight`
- `SelectDirectMappingTargetUp`
- `SelectDirectMappingTargetDown`
- `ActivateDirectMappingTarget`
- `StartDirectMappingHintJump`

Recommendation:

- reuse `ActivatePageItem` and `CancelCurrentMode`
- add direct-mapping-specific directional actions rather than overloading page-item semantics beyond recognition

That keeps remapping support intact while avoiding leaky abstractions.

## Scope Behavior

Scope resolution should not change based on targeting method.

The current direct-mapping rule remains correct:

- the selected control determines the canonical `target_label` and `scope_label`
- pointer, touch, directional navigation, and hint jump all produce the same descriptor for the same visible control

Scope requirements:

- transport and song-level controls resolve to `Global`
- active-track controls resolve to `Active Track` or `Armed/Active` only when that is the current control's visible semantic scope
- absolute per-track controls resolve to `Track N`
- the currently active track number must not silently rewrite an active-track control into `Track N`

This is especially important for no-pointer selection because keyboard navigation may move across repeated-looking controls; the footer and highlight must always show the resolved scope before input capture begins.

## Selection Flow

### Entering Targeting

When direct mapping enters `Targeting`:

1. gather current visible direct-mapping targets from the page
2. choose an initial current target
3. draw all eligible targets with a passive outline
4. draw the current target with a strong active outline
5. show footer text naming:
   - control label
   - scope
   - available selection methods
   - whether the mode is plain targeting or explicit jump mode

Recommended initial-target rule:

- preserve the previous target if it is still visible after a successful commit
- otherwise choose the first target in stable order
- if entered from the mappings page, prefer the `Direct Map` affordance or first primary page target rather than an arbitrary mid-page control

### Entering AwaitingInput

Selecting a target by any method should:

- set `DirectMappingMode::AwaitingInput(current_target)`
- show the existing capture footer with the target label and scope
- suppress ordinary activation of the selected control

Clarified activation rule:

- `Enter` is the explicit path that converts the current target selection into armed next-input capture
- plain keyboard key presses while in `Targeting` should map immediately to the current highlighted target instead of first entering `AwaitingInput`

### Retargeting While Awaiting Input

The current shipped retargeting behavior should stay intact and extend to keyboard navigation:

- arrow, tab, pointer, or jump selection while `AwaitingInput` should switch to another target immediately
- the user should not have to cancel capture first

## Conflict and Replacement Rules

This spec should preserve the current direct-mapping commit model unless the existing behavior is later revised by the broader direct-mapping spec.

For no-pointer selection, the important requirement is consistency:

- choosing a target by keyboard must reuse the same create/replace/source-conflict rules as choosing it by pointer
- there must be no "keyboard-targeted mappings" special case in persistence

Required behavior:

1. unique existing row for same target/scope/source kind -> replace/update that row
2. exact existing source bound elsewhere -> use the same move/replace/conflict handling as pointer direct mapping
3. commit result selects the resulting mappings row and returns to `Targeting`
4. origin behavior stays unchanged:
   - mappings-page origin returns to `Mappings`
   - in-place origin stays on the current page

## Desktop vs Touch vs Controller-Driven Use

### Desktop Keyboard-First

This is the primary target for the feature.

- user enters direct mapping
- directional highlight appears
- user moves among targets with arrows or `Tab`
- user either:
  - presses a plain key to map that key directly to the current target, or
  - presses `Enter` and then sends the next MIDI/keyboard source
- mapping commits and targeting stays active

### Desktop Pointer+Keyboard Hybrid

- pointer hover and click remain available
- after a pointer selection or commit, the navigator current target should synchronize to that same target
- the user can continue with arrows from there

### Touch

Touch still cannot rely on hover, but it benefits from the same visible target exposure.

Recommended touch behavior:

- active direct-mapping target is still visible even if the user has not tapped yet
- hint labels, if shown, are presentation only; touch selection still occurs by tap
- the success/footer language should not say hover or mouse-only terms

### Controller-Driven / No Pointer Attached

The feature should assume a live-performance scenario where the user may have:

- a keyboard only
- a controller plus a small keyboard
- a handheld device with no mouse

In this mode, every required step must be available without pointer affordances:

- enter mode
- reveal/select target
- confirm target
- capture source
- cancel
- continue to next target

## Visual and Copy Rules

Footer/status copy while targeting should name the current selected target, not just generic mode text.

Recommended example:

`Direct Map | Play/Stop (Global) | Arrows move, Tab cycles, press a key to map, Enter arms next input, / shows hints, Esc cancels`

Recommended example during jump mode:

`Direct Map Jump | Type hint letters to retarget, Enter arms next input, Esc cancels`

Required visual distinction:

- passive eligible outline for all actionable controls
- active outline for the current navigator target
- awaiting-input outline stronger than targeting-only outline
- explicit jump hints reuse discoverability-style floating chips with clear background and border
- hint labels must not permanently shift layout

## Acceptance Criteria

1. The user can fully complete a direct-mapping flow without mouse or touch.
2. Entering direct mapping chooses a visible current target automatically when at least one target exists.
3. `Left` / `Right` / `Up` / `Down` move among eligible direct-mapping targets without activating the underlying controls.
4. `Tab` / `Shift+Tab` traverse eligible targets in a stable order.
5. Pressing a plain keyboard key while targeting maps that key immediately to the current canonical target and scope.
6. `Enter` on the current target enters `AwaitingInput` for the same canonical target and scope that pointer selection would produce.
7. Hint labels are shown only during explicit jump mode entered with `/`.
8. Hint labels are prefix-free for the current visible target set.
9. While awaiting input, selecting another target by keyboard retargets capture immediately.
10. Hint-jump selection produces the same canonical target descriptor as directional or pointer selection.
11. Commit, replacement, and conflict behavior is identical regardless of target-selection method.
12. Mappings-page-origin direct mapping still returns to the mappings page after commit.
13. In-place-origin direct mapping still keeps the user on the current page after commit.
14. Touch presentation does not depend on hover-only instructions.
15. If no eligible targets exist on the current page, the app surfaces that state explicitly instead of silently doing nothing.

## Likely Code Touch Points

### `src/app/types.rs`

Add transient navigation state for direct mapping, for example:

- current target index or current target id
- optional jump-label/session state
- possibly a richer direct-mapping mode enum if jump mode becomes explicit

### `src/app/input.rs`

Add keyboard/controller target navigation while direct mapping is active:

- directional movement
- sequential traversal
- target activation
- optional hint-jump text handling
- synchronization between pointer-selected target and keyboard-selected target

### `src/app/direct_mapping_ui.rs`

Keep commit logic shared across all targeting methods.

Possible additions:

- helper to set current direct target
- helper to preserve/restore current target after commit
- helper to resolve the current visible target list

### `src/app/discoverability_ui.rs`

Extend rendering to support:

- active navigator outline distinct from passive eligible outline
- hint-label rendering for jump mode
- footer copy for current target and jump mode

### `src/page_widgets.rs`

Potentially extend the page target contract so targets can be returned with:

- stable semantic order
- optional navigation grouping metadata
- optional jump-label priority

### Page-specific target producers

Likely touched files:

- `src/app/timeline/ui.rs`
- `src/app/routing_ui.rs`
- later `src/app/mapping/page.rs` if mappings-page controls become direct targets

These producers may need to expose targets in stable order rather than only as unordered hit-test rectangles.

### `src/actions.rs`

Either:

- explicitly reuse existing page-item actions for part of the flow, or
- add direct-mapping-specific navigation/jump actions

Keyboard defaults should be documented in `docs/dev/current-mappings.md` once implemented.

## Testing Focus

Add focused tests for:

- initial-target selection when entering `Targeting`
- stable `Tab` traversal order
- arrow-key movement picking the intended geometric neighbor
- retargeting while `AwaitingInput`
- hint-jump label matching and cancel behavior
- identical scope resolution across pointer and keyboard selection of the same control
- no underlying action activation while navigating direct-mapping targets
- no-target pages surfacing an explicit status message

## Delivery Notes

Recommended implementation order:

1. add transient current-target state and footer copy
2. implement stable sequential traversal with `Tab`/`Shift+Tab`
3. implement `Enter` activation and retargeting while awaiting input
4. add arrow-key geometric movement
5. add hint-jump labels and jump-session input handling
6. expand target coverage beyond current timeline/routing support as needed

## Relationship To Existing Specs

This spec is intentionally narrower than `docs/specs/direct-ui-mapping-mode-spec.md`.

- `direct-ui-mapping-mode-spec.md` defines the broader direct-mapping workflow and mapping semantics
- this spec defines the no-pointer target-selection layer that should plug into that workflow

If implementation reveals that the broader direct-mapping spec's conflict model needs revision, that should be updated in the broader spec rather than forked here.
