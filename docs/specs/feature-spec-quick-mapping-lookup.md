# Feature Spec: Quick Mapping Lookup

## Summary

Add an inline target lookup/edit mode to the `Mappings` page so a user can select the `Target` field, start typing, and use fuzzy search to choose the intended canonical mapping target without cycling through the full target list one item at a time.

This spec is grounded in the current repository state:

- `docs/specs/product-spec.md`
- `docs/planning/handoff-summary.md`
- `docs/specs/direct-ui-mapping-mode-spec.md`
- `docs/specs/feature-spec-mapping-discoverability.md`
- `docs/dev/current-mappings.md`
- `src/actions.rs`
- `src/pages.rs`
- `src/mapping.rs`
- `src/app.rs`

## Problem

The current mappings editor is row-based and action-driven, but target selection is still list-cycling only:

- write mode must be enabled
- the `Target` field must be selected
- `Q` / `E` or `Enter` cycles through `TARGET_OPTIONS`
- changing target resets scope to that target's default scope

That works for a short list, but `src/mapping.rs` already contains a broad mapping target catalog spanning transport, looping, track state, stored-loop actions, recording-clip actions, note-edit actions, page/overlay actions, and Link controls. The current interaction becomes slow when the user knows roughly what they want (`arm`, `recording`, `slot 4`, `note`, `loop`) but not the exact position in the catalog.

This is especially noticeable now that the product already supports:

- direct UI mapping mode (`F8`)
- discoverability overlay (`F7`)
- keyboard and MIDI mappings targeting the same canonical action layer
- absolute track scopes on top of active-track-relative actions

The mappings page still needs a fast, text-driven lookup path for cases where the user is editing rows directly rather than mapping from the UI surface.

## Goals

- Let the user quickly find a target by typing a few characters instead of cycling the whole list.
- Reuse the current canonical target labels from `src/mapping.rs`; do not create a second target namespace.
- Keep the current row editor and action-driven navigation model intact.
- Keep the mappings editor aligned with the product direction that the app is hyper-mappable: important editor actions should be reachable through the same canonical action surface as transport and track actions.
- Make scope behavior explicit when changing targets through lookup.
- Work on desktop keyboard-first setups first, without blocking touch support on the mappings page.
- Keep conflict/replacement behavior aligned with the existing row model rather than inventing hidden auto-merge behavior.

## Non-Goals

- Replacing direct UI mapping mode from `docs/specs/direct-ui-mapping-mode-spec.md`.
- A full free-form command palette for every page.
- Fuzzy lookup for source device, source value, or scope in this slice.
- Implementing OSC learn or a broader text-entry system for every control.
- Automatic conflict resolution beyond what the mappings page already exposes through row selection and editing.

## Current Baseline

### Action and page model

Current code already provides the right foundation:

- `AppAction` includes page/editor actions plus `ToggleDirectMappingMode` in `src/actions.rs`.
- `AppPageState` in `src/pages.rs` tracks:
  - `selected_mapping_index`
  - `mapping_mode`
  - `selected_mapping_field`
  - `mapping_midi_learn_armed`
- `App::adjust_mapping_field` and `App::activate_mapping_field` in `src/app.rs` currently mutate the selected row.
- Mapping targets and valid scope defaults already come from `src/mapping.rs` through:
  - `cycle_mapping_target_label`
  - `default_scope_label`
  - `cycle_mapping_scope_value`
  - `scope_options_for_target`

### Existing mappings-page UX

Current mappings-page behavior:

- `W` toggles read-only/write mode.
- `Shift+Left` / `Shift+Right` selects the active field.
- `Q` / `E` adjusts the selected field.
- `Enter` activates the selected field.
- `Target` changes are immediate and currently list-cycled.
- Changing target resets scope to the default for that target.
- `SourceValue` already has a special edit state for MIDI learn via `mapping_midi_learn_armed`.

Implication:

- Quick lookup should feel like a sibling to MIDI learn: a temporary field-level editing state inside the existing mappings page, not a separate page.
- Longer-term, mappings-page navigation/editing should also be treated as a mappable action surface rather than a keyboard-only implementation detail.

## Canonical model: hyper-mappable editor actions plus keyboard accelerator

The recommended model is a two-layer interaction design.

### Layer 1: canonical editor actions

The durable, cross-device model should be action-driven:

- select row
- select field
- adjust backward/forward
- activate current field
- commit
- cancel

These actions already largely exist in `AppAction` and current page-state behavior:

- `SelectPreviousPageItem`
- `SelectNextPageItem`
- `SelectPreviousPageField`
- `SelectNextPageField`
- `AdjustPageItemBackward`
- `AdjustPageItemForward`
- `ActivatePageItem`
- `ToggleMappingsWriteMode`
- `AddMappingRow`
- `RemoveSelectedMapping`

Recommended product direction:

- expose these editor/navigation actions as mapping targets in the future
- treat the mappings page itself as part of the app's hyper-mappable action surface
- ensure MIDI, keyboard, mouse, touch, and future OSC can all drive the same canonical editor model

This is the consistency layer.

### Layer 2: direct-manipulation and text accelerators

Faster input-specific affordances may sit on top of the canonical layer:

- mouse/touch clicking specific rows and fields directly
- keyboard fuzzy target lookup
- direct UI mapping hit-target selection

These accelerators are valuable, but should not become the only way to complete an operation.

### Deliberate exception: keyboard lookup

Target lookup is allowed to be a keyboard-first accelerator because large target sets benefit disproportionately from text search.

However, the canonical model should still exist underneath it:

- `ActivatePageItem` on `Target` opens the picker
- non-text actions should be able to move lookup selection
- non-text actions should be able to commit/cancel

Design rule:

> Keyboard text lookup is an accelerator, not the canonical interaction contract.

This keeps the app mostly consistent with its hyper-mappable philosophy while allowing a practical keyboard-only fast path for large target catalogs.

## Proposed UX

## Entry into target lookup edit mode

When all of the following are true:

- current page is `Mappings`
- page mode is `Write`
- selected field is `Target`

then the user can enter target lookup edit mode by:

- pressing `Enter`, or
- tapping/clicking the selected `Target` cell on desktop/touch

Recommended keyboard shortcut behavior:

- first `Enter` on `Target` opens lookup edit mode
- subsequent `Enter` commits the currently highlighted result
- `Escape` cancels lookup edit mode and restores the original target/scope
- `Up` / `Down` moves within filtered results while lookup is open; reaching the bottom/top of the visible subset should continue through the full filtered result set by scrolling rather than wrapping
- typed printable keys update the query instead of triggering global bindings
- `Backspace` deletes one query character

Recommended compatibility rule:

- `Q` / `E` should continue to cycle targets when lookup is not open
- once lookup is open, text entry owns the keyboard until commit/cancel

Recommended future-compatible rule:

- the picker opened by `Target` activation should also support non-text navigation through canonical editor actions, so MIDI-mapped or pointer-driven operation does not depend on typing

## Inline presentation

The lookup should render inline within the mappings page rather than as a full-screen modal.

Recommended presentation:

- the selected `Target` cell becomes an active text field
- a compact dropdown/list appears under or near the selected row
- the list shows:
  - best-match target labels
  - optional short category/context hint when useful
  - current scope preview or scope consequence when useful

Example queries:

- `arm` -> `Track Arm`
- `loop` -> `Track Loop`, `Song Loop`, `Reset Song Loop`, stored-loop actions
- `slot 4` -> recall/store/clear slot 4 actions
- `record` -> `Record`, `Record Mode`, clip-related recording actions, `Recording View`
- `note` -> note-selection and note-nudge actions

## Fuzzy ranking behavior

Quick lookup should match against canonical target labels only.

Recommended ranking order:

1. exact case-insensitive full-label match
2. case-insensitive prefix match on a word boundary
3. substring match
4. fuzzy ordered-character match

Recommended tie-breakers:

- shorter label first
- more commonly used/global actions first only when the textual score is equal
- stable alphabetical order as final fallback

Important scope rule:

- lookup changes only the target label directly
- scope remains a separate field
- however, if the current scope is invalid for the newly selected target, the app resets scope using `default_scope_label(target, track_count)`
- if the current scope is still valid for the new target, preserve it

This is a better fit than always resetting scope, because it avoids unnecessary loss of `Track N` or `Active Track` intent when the user moves among related track actions.

## Action model reuse

Lookup must continue to use the existing canonical mapping row model:

- `target_label` remains the persisted identifier used by the row
- downstream action resolution still goes through existing `mapping_entry_to_actions` and `mapping_entry_targets_action` helpers
- no new per-widget or per-alias persisted label should be introduced

Allowed search aliases may exist in memory for matching only, but commit must always write the canonical existing label.

Examples of safe non-persisted aliases:

- `play` -> `Play/Stop`
- `thru` -> `Passthrough`
- `mute clip` -> `Recording Clip Mute`
- `clear slot 3` -> `Clear Stored Loop Slot 3`

## Scope behavior

Scope behavior should stay explicit and predictable.

### Reuse existing scope rules

The lookup feature should continue to rely on the existing target-to-scope rules in `src/mapping.rs`.

That means:

- global-only targets remain global-only
- track actions continue to support `Active Track` plus absolute `Track N`
- note-edit actions continue to use active-track/absolute-track scopes as already defined
- `Select Track` keeps its relative/absolute scope model

### When target selection changes scope

On commit from lookup edit mode:

1. if the row's current scope is valid for the chosen target, keep it
2. otherwise replace it with `default_scope_label` for that target
3. if scope changes automatically, show that change clearly in the row/footer copy for that commit

Example:

- row currently `Track Arm | Track 3`
- user searches `Track Mute`
- `Track 3` remains valid, so preserve it

Example:

- row currently `Track Arm | Track 3`
- user searches `Play/Stop`
- `Track 3` is invalid for a global target, so scope becomes `Global`

## Conflict and replacement rules

Quick lookup edits the currently selected row. It is not a direct-map capture flow.

Therefore:

- selecting a target through lookup updates only the selected row
- it must not silently rewrite other rows
- it must not disable sibling rows automatically
- it must not deduplicate bindings automatically

However, the spec should keep conflict visibility aligned with the current mappings model.

Recommended behavior after commit:

- if another enabled row already has the same source + target + scope, allow it but surface a lightweight duplicate/conflict hint in the mappings footer or row styling later
- if another enabled row shares the same source but points elsewhere, do not auto-resolve here; this remains row-editor behavior
- if the edited row becomes a likely replacement candidate for an existing identical row, the implementation may add a later follow-up prompt, but that is out of scope for this slice

Rationale:

- direct UI mapping mode is the place for source-capture replacement logic
- mappings-page lookup is the place for fast row editing

## Desktop and touch behavior

### Desktop

Primary interaction is keyboard-first:

- open lookup with `Enter` on the selected `Target` field
- type to filter
- use arrow keys to choose
- `Enter` commits
- `Escape` cancels

Mouse support on desktop:

- click selected target cell to open lookup
- click result to commit
- click outside closes lookup and preserves or cancels according to final implementation choice; recommended default is cancel without mutation

Recommended future addition:

- visible previous/next result affordances or scroll affordance for pointer-only operation when the desired target is not in the initial visible subset

### Touch

Touch is secondary but should not be blocked by the model.

Recommended touch behavior:

- tapping the selected target cell opens the same lookup panel
- panel includes a focused text field when an on-screen keyboard is available
- tapping a result commits immediately
- a visible cancel affordance is required; touch should not rely on `Escape`

Because the current app is still desktop-oriented and uses SDL UI primitives, the first implementation may support pointer opening and result tapping while leaving full soft-keyboard validation to later device testing.

### MIDI and other mapped input behavior

For long-term consistency, MIDI-mapped editor actions should be able to operate the mappings page and the target picker without requiring keyboard text entry.

Recommended picker behavior once editor navigation targets are exposed:

- `ActivatePageItem` on `Target` opens lookup
- `AdjustPageItemForward` / `AdjustPageItemBackward` moves highlighted lookup result while lookup is open
- `ActivatePageItem` commits highlighted result while lookup is open
- a future explicit `Cancel` action closes lookup without mutation

This would let:

- MIDI-only rigs navigate and edit mappings using mapped buttons/encoders
- mouse users rely on direct manipulation without being forced into typing for completion
- keyboard users retain the faster text-search path

## State model

Recommended state addition:

Add a dedicated field-level edit state instead of overloading `mapping_midi_learn_armed`.

Suggested shape:

- new mappings-page editor substate, for example:
  - inactive
  - target_lookup { original_target, original_scope, query, highlighted_result }
  - midi_learn (existing behavior, possibly migrated into the same enum later)

Reasoning:

- `mapping_midi_learn_armed` already behaves like a mini modal state for `SourceValue`
- target lookup is another field-specific modal state
- consolidating these into one editor-state concept reduces future branching when more field editors appear

## Likely implementation touch points

### `src/pages.rs`

Likely additions:

- mappings-page editor substate persisted or session-only
- selected result index/query state if stored outside `App`

### `src/mapping.rs`

Likely additions:

- expose target catalog in a lookup-friendly form instead of cycle-only access
- helper to enumerate canonical target labels
- helper to validate whether a scope is allowed for a target
- fuzzy/substring scoring helper and optional search aliases

### `src/app.rs`

Likely additions:

- event handling for text input and backspace while lookup is open
- open/close/commit/cancel methods for target lookup
- mappings-page rendering for inline query field and result list
- commit logic that preserves scope when still valid and resets only when invalid
- suppression of normal page/global key handling while lookup owns focus

### `src/actions.rs`

Possible additions or clarifications:

- action labels/help text may need to reflect that `Enter` on `Target` now opens lookup edit mode instead of always cycling
- built-in keyboard help text may need a lookup-specific hint if surfaced anywhere

### Docs likely to update during implementation

Not part of this spec-only commit, but likely follow-up implementation updates:

- `README.md` if the mappings-page interaction surface changes materially
- `docs/dev/current-mappings.md` for the new target-edit workflow
- screenshots/review artifacts if the mappings page layout changes visibly

## Acceptance Criteria

- In mappings write mode, selecting the `Target` field and pressing `Enter` opens a target lookup edit state.
- While target lookup is open, typed keyboard input updates a query instead of triggering normal app actions.
- The result list filters canonical mapping targets from the current `src/mapping.rs` catalog.
- Fuzzy or substring matching allows queries like `arm`, `slot 4`, `record`, or `note` to surface relevant targets quickly.
- `Up` / `Down` changes the highlighted result and `Enter` commits it.
- Hitting `Next` on the last visible row or `Previous` on the first visible row should move through the remaining filtered results by scrolling the list window; it should not wrap to the opposite end.
- `Escape` cancels lookup edit mode without mutating the row.
- Committing a new target preserves the current scope when that scope remains valid for the new target.
- Committing a new target resets scope only when the previous scope is invalid for the chosen target.
- The committed row still persists the canonical existing `target_label`; no new persisted alias format is introduced.
- Existing `Q` / `E` target cycling continues to work when lookup edit mode is not active.
- The feature does not alter direct UI mapping replacement/conflict semantics.

## Suggested implementation order

1. Add target-catalog enumeration and target/scope validation helpers in `src/mapping.rs`.
2. Add mappings-page target-lookup state and query/result bookkeeping.
3. Add inline rendering for the lookup field and filtered results.
4. Route text input/backspace/up/down/enter/escape to the lookup state while active.
5. Update commit logic so target changes preserve valid scopes instead of always resetting.
6. Refresh docs and screenshots only if the implemented UI changes the visible mappings-page layout materially.

## Follow-on plan: expose editor actions as mapping targets

Recommended follow-on slice after the current lookup implementation:

1. Add mapping targets for editor/navigation actions already represented in `AppAction`, especially:
   - `SelectPreviousPageItem`
   - `SelectNextPageItem`
   - `SelectPreviousPageField`
   - `SelectNextPageField`
   - `AdjustPageItemBackward`
   - `AdjustPageItemForward`
   - `ActivatePageItem`
   - `ToggleMappingsWriteMode`
   - `AddMappingRow`
   - `RemoveSelectedMapping`
2. Keep those targets canonical and shared across keyboard, MIDI, and future OSC.
3. Teach the target picker to interpret canonical adjust/activate/cancel actions while open.
4. Add pointer-visible navigation affordances so pointer-only use does not depend on typing.
5. Update `docs/dev/current-mappings.md`, `README.md`, and screenshots when those actions become user-exposed mapping targets.

## Open questions

These now have implementation recommendations for the current shipped behavior:

- Aliases in V1:
  - recommendation: defer aliases
  - implemented behavior: fuzzy matching runs against canonical target labels only
- `Tab` while lookup is open:
  - recommendation: suppress it so the temporary editor keeps focus
  - implemented behavior: `Tab` does nothing until explicit commit/cancel
- Clicking outside the lookup:
  - recommendation: cancel immediately without mutation
  - implemented behavior: outside click/tap cancels lookup and restores the original row target/scope
- Query persistence:
  - recommendation: start fresh each time
  - implemented behavior: reopening lookup starts from an empty query rather than preserving the previous query
