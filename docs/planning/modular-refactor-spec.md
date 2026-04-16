# Modular Refactor Spec

## Summary

Refactor the current app so feature logic is organized into clear domain-scoped modules instead of co-living primarily in `src/app.rs` and a few large shared files.

This is a structural refactor, not a product-behavior rewrite.

Required outcome:

- preserve the current runnable UX and action behavior
- preserve existing test expectations except for safe relocation or equivalent test harness updates
- make the codebase easier to extend with the already-in-flight directions from:
  - `vk/9b67-feature-spec-mid`
  - `vk/0e32-feature-spec-und`
  - `vk/0afe-color-refactor`
- treat renderer screenshots as a regression gate: the tracked screenshots should show no intentional visual change before vs after the refactor

This spec is grounded in the current repository state, including `README.md`, `docs/specs/product-spec.md`, `docs/specs/feature-spec-midi-manipulation.md`, `docs/specs/feature-spec-mapping-discoverability.md`, `docs/specs/direct-ui-mapping-mode-spec.md`, `docs/feature-spec-undo.md`, `docs/planning/handoff-summary.md`, `docs/planning/implementation-plan.md`, `docs/planning/color-refactor-plan.md`, and the current `src/*` code.

## Problem

The current app surface is already feature-rich, but implementation responsibilities are not cleanly separated.

Current pressure points in the repo:

- `src/app.rs` is roughly 9k lines and currently mixes:
  - application state
  - action application
  - direct mapping state machine
  - pointer/touch routing
  - page rendering
  - timeline rendering
  - screenshot capture helpers
  - mapping conflict/replacement behavior
  - test-only helpers and a large test suite
- `src/actions.rs`, `src/project.rs`, and `src/mapping.rs` are also large and contain both domain logic and presentation-adjacent helpers
- color/styling logic remains largely inline, which conflicts with the desired theme extraction direction in `docs/planning/color-refactor-plan.md`
- upcoming undo/history work in `docs/feature-spec-undo.md` will be harder to add safely if mutable state boundaries remain blurred
- the recently expanded direct mapping and note-editing flows increase the cost of any change made inside one central file

This increases risk in three ways:

1. feature edits have wide blast radius
2. tests overfit file placement instead of domain boundaries
3. future feature merges will keep enlarging shared files instead of composing from reusable subsystems

## Refactor Goals

- split app behavior by domain and interaction layer, not by arbitrary helper buckets
- preserve the canonical action-driven architecture
- isolate UX/state-machine logic for mapping, discoverability, note editing, and recording so each can evolve independently
- extract common rendering/layout/palette patterns into shared utilities without creating a heavyweight UI framework
- prepare clean seams for:
  - direct mapping and discoverability growth
  - undo/history snapshots and domain ownership
  - theme/palette externalization
  - further timeline/note editing work
- keep the current persisted state shape and user-visible behavior stable unless a compatibility shim is required for internal cleanup

## Non-Goals

- redesigning the app UX during this refactor
- changing keyboard shortcuts, mapping targets, routing behavior, loop behavior, or recording semantics
- introducing a new retained-mode UI layer or style system
- removing the current action model in favor of direct widget mutation
- reducing test coverage as a shortcut for making the refactor easier

## Architectural Rules

### 1. Keep the action boundary canonical

All user and device inputs must continue to converge on `AppAction` plus action source metadata.

The refactor must not scatter behavior into page-local shortcut logic or widget-specific mutation paths.

Required preservation:

- keyboard bindings still resolve through `src/actions.rs`
- MIDI mappings still resolve to canonical targets/scopes before mutating state
- pointer and touch interactions still translate into canonical actions when they represent user commands
- direct mapping capture still commits through canonical target/scope descriptors, not ad hoc widget ids

### 2. Separate domain state from runtime/session glue

The refactor should make state ownership explicit so future undo/history work can snapshot clear slices.

The target structure should distinguish at least:

- project/content state
- page/editor state
- mapping/discoverability state
- runtime/session state
- render-only derived state and layout helpers

This does not require shipping undo in the refactor, but the module boundaries must stop making undo harder.

### 3. Separate rendering from behavior where practical

Layout math, color/styling decisions, and draw helpers should move out of feature state reducers when that can be done without changing behavior.

State mutation and drawing must remain coordinated, but the same function should not own both when a stable seam exists.

### 4. Prefer feature/domain modules over generic catch-all helpers

Good extraction targets are domains with stable meaning:

- direct mapping
- discoverability
- timeline/note editing
- transport strip
- routing page behavior
- mappings page behavior
- screenshot capture/specs
- theme/palette

Avoid creating vague dumping grounds such as `helpers.rs`, `shared.rs`, or `misc.rs`.

## Target Module Shape

The exact filenames may vary slightly, but implementation should converge on this responsibility split.

### App shell and orchestration

Keep `src/app.rs` as the top-level integration layer only.

It should own:

- the `App` struct and top-level startup/runtime wiring
- cross-domain coordination
- the main event loop integration points
- high-level page dispatch
- status propagation between domains

It should stop owning detailed logic for every feature.

### Suggested extracted modules

#### `app/state` or equivalent app-scoped state module

Own small app-level state structs that are currently embedded in `app.rs`, especially editor/runtime state that is not core project content.

Likely contents:

- overlay state
- status state
- direct mapping state structs/enums
- last-action status
- other app-only coordination state

#### `app/input`

Own pointer/touch/keyboard interaction routing that is currently mixed into the main app implementation.

Likely contents:

- pointer hit routing helpers
- pointer-to-action translation
- touch vs desktop branching where behavior differs
- hover/selection interaction policies

#### `app/render`

Own page dispatch and shared chrome rendering that currently lives in `app.rs`.

Likely contents:

- page tabs
- footer/status bar
- overlay dispatch
- transport strip chrome helpers shared across pages
- shared badge/chip drawing helpers that are app-specific rather than generic SDL helpers

#### `timeline_ui`

Own timeline-specific rendering and interaction support extracted from `app.rs`.

Likely contents:

- timeline page rendering
- track column/subcolumn rendering
- recording lane layout
- note/region draw helpers
- timeline-specific hit targets
- timeline track indicator target descriptors

This module should depend on the canonical project/transport/timeline models, not re-encode them.

#### `mapping_ui`

Own mappings-page rendering and direct-mapping UX behavior.

Likely contents:

- mappings page draw helpers
- direct mapping footer summaries
- direct mapping target discovery
- direct mapping conflict/replacement workflow helpers
- mapping badge summarization and rendering

#### `routing_ui` and `midi_io_ui`

Own rendering and interaction policies for their respective pages, especially field layout and pointer/touch hit handling.

#### `theme`

Add a dedicated theme/palette module as described in `docs/planning/color-refactor-plan.md`.

This refactor does not need to finish all theme migration, but it should create the module and move shared semantic colors/styles there wherever extraction is low-risk.

At minimum, new extractions must avoid adding more inline color literals to `app.rs`.

#### `capture` or `app_capture`

Move screenshot-capture specs and readback helpers out of the main app logic.

Likely contents:

- capture specs
- readback structs/helpers
- deterministic demo capture seed helpers

This keeps renderer regression tooling first-class without bloating page logic.

#### `mapping`, `timeline`, `project`, `transport`

Keep these domain modules as the model/behavior layer, but extract presentation-only helpers out of them when encountered.

## Behavior Preservation Requirements

The refactor must preserve the following current behavior contracts.

### Direct mapping UX flow

The behavior implemented in the current app and specified in `docs/specs/direct-ui-mapping-mode-spec.md` must remain stable.

Required preserved flow:

1. enter direct mapping mode from keyboard or mappings-page affordance
2. target actionable UI elements only
3. reuse canonical action target + scope descriptors
4. capture next qualifying keyboard or MIDI input
5. apply create/replace/move behavior deterministically
6. return to targeting mode with status feedback
7. when entered from the mappings page, return there after commit

Required preservation details:

- `Targeting`, `AwaitingInput`, and cancel/commit states remain explicit and testable
- `Escape` and `F8` still cancel instead of being capturable mappings
- keyboard capture precedence stays ahead of built-in shortcut fallback where currently tested
- pointer targeting while awaiting input can retarget without forcing an explicit cancel first

### Scope behavior

Scope resolution must remain canonical and centralized.

Required behavior:

- global controls resolve to global scope
- active-track controls resolve to active-track scope where the current UX implies relative behavior
- absolute-track controls retain absolute scope where the target is explicitly per-track
- mapping summaries, conflict rules, and replacement rules must continue to use canonical target+scope descriptors rather than page-local labels

This is especially important for the note-editing actions added around `vk/9b67-feature-spec-mid` and for future undo domain tagging from `vk/0e32-feature-spec-und`.

### Conflict and replacement rules

The direct mapping feature currently contains important replacement behavior that must not be lost in extraction.

Preserve these observable outcomes:

- unique target row reuse still updates the existing mapping row in place
- existing-source remap still moves or disables old target rows in the same cases currently covered by tests
- idempotent remaps do not create duplicate rows
- no hidden duplicate rows are introduced by refactor-only changes

The replacement/conflict decision logic should be extractable into a mapping-domain helper module with pure inputs/outputs so it is easier to test outside `app.rs`.

### Touch vs desktop interaction differences

The current app already supports mouse/touch for non-timeline chrome and direct mapping/discoverability affordances.

The refactor must preserve platform-mode behavior differences where they exist:

- desktop hover continues to drive discoverability/status summaries
- touch must not rely on hover-only affordances
- pointer-down handling for tabs, transport controls, mappings, MIDI I/O, and routing must stay behaviorally identical
- direct mapping targeting must continue to work when the source is pointer or touch

Any branching for desktop vs touch should move toward explicit policy helpers instead of remaining embedded across many rendering functions.

### Note editing and manipulation

The action-driven note-selection and note-manipulation workflow described in `docs/specs/feature-spec-midi-manipulation.md` must remain unchanged.

Important preservation points:

- additive selection hold behavior
- focus/anchor semantics
- playhead-based selection entry
- quantized vs unquantized nudge defaults
- active-track-relative default scope with support for absolute targeting through mappings

### Screenshot-equivalent UI output

Because this is a refactor, tracked renderer screenshots should remain visually unchanged unless an intentional cleanup is explicitly called out and approved.

Regression expectation:

- capture the tracked screenshots before and after
- compare them as a visual regression check
- treat differences as bugs unless they are intentional and documented

## Test And Verification Strategy

### Existing tests

The current test suite is heavily concentrated in `src/app.rs`, including direct mapping, note editing, routing, recording, and page behavior.

Refactor rule:

- keep the same coverage expectations
- tests may move with the code they validate
- assertions may be adjusted only where relocation changes test harness setup, not where product behavior changes

Preferred direction:

- pure domain helpers gain local unit tests in their owning modules
- app-level integration tests remain for end-to-end behavior that crosses modules
- avoid leaving all behavior tests in `app.rs` once the behavior has moved elsewhere

### Screenshot regression

Use the renderer-owned screenshot flow as a non-negotiable regression tool for this refactor:

- `powershell -ExecutionPolicy Bypass -File .\scripts\capture-ui-screens.ps1 -StateMode demo`
- `powershell -ExecutionPolicy Bypass -File .\scripts\review-ui-screens.ps1 -StateMode demo`

Required interpretation:

- for this refactor, screenshot diffs should be zero or trivially explainable by deterministic rendering noise
- if screenshots change materially, the refactor is not complete unless the change was separately intended and documented

### Completion confidence gate

Implementation follow-through on this spec must use the required confidence gate from the request.

Before declaring the refactor done, the implementation pass must report at least `84.7%` confidence based on:

- testing evidence: `40%`
- code review evidence: `30%`
- logical inspection evidence: `30%`

If below threshold, the implementation report must include:

- current score
- top remaining gaps
- minimum next checks needed to cross the threshold

## Recommended Refactor Sequence

### Phase 1: Inventory and guardrails

- inventory `app.rs` by responsibility cluster
- identify pure helpers, state machines, rendering clusters, and page-specific interaction handlers
- pin the regression baseline with tests and screenshots before structural edits

### Phase 2: Extract pure helpers and state types first

Start with low-risk moves that reduce file size without changing call flow:

- capture helpers
- mapping summary helpers
- color/palette helpers
- layout structs
- direct mapping state types
- badge/style helpers

### Phase 3: Extract feature-local UI modules

Move domain-specific rendering and hit-target logic out of `app.rs` in slices:

1. mappings/direct mapping/discoverability
2. routing + MIDI I/O page helpers
3. timeline rendering helpers and interaction descriptors
4. transport strip and shared footer/chrome

Each slice should preserve behavior and keep tests green before moving on.

### Phase 4: Normalize shared utilities

After the first extractions settle, consolidate genuinely shared patterns into narrowly named utility modules.

Good candidates:

- hit target rect helpers
- style structs for chips/badges/panels
- semantic color/palette lookups
- text/status summarization helpers

### Phase 5: Prepare for future work without implementing it

Leave clean extension seams for:

- undo domain snapshot boundaries
- additional direct mapping targets or conflict UI
- more timeline editing actions
- alternate themes

Do not ship partial undo/history or unfinished theming behavior as part of this refactor unless separately requested.

## Likely Code Touch Points

Primary refactor touch points:

- `src/app.rs`
- `src/actions.rs`
- `src/mapping.rs`
- `src/ui.rs`
- `src/pages.rs`
- `src/project.rs`
- `src/lib.rs`
- `src/app_ui/branding.rs`
- new modules under `src/` or `src/app/` for app shell, page UI, capture, and theme responsibilities

Likely supporting touch points:

- `README.md` only if the runnable app surface or documented architecture summary changes materially
- docs under `docs/dev/` or `docs/planning/` if the implemented module map diverges from this spec

## Acceptance Criteria

The refactor is successful when all of the following are true:

- `src/app.rs` is substantially smaller and no longer owns most feature-specific logic
- the codebase has clear domain-scoped modules for at least direct mapping/discoverability, timeline UI, page UI behavior, capture helpers, and theme/palette ownership
- canonical `AppAction` routing remains the single control boundary across keyboard, MIDI, pointer, and touch inputs
- direct mapping UX, scope behavior, and replacement/conflict outcomes remain behaviorally identical to the current tested surface
- note-editing actions continue to behave identically to the current tested surface
- existing tests continue to pass after relocation-adjusted test updates
- screenshot regression review shows no intentional visual changes before vs after
- new modules have names that reflect stable responsibilities rather than generic helper buckets
- the resulting structure makes the planned undo/history and theme work easier rather than harder

## Open Implementation Defaults

These defaults are chosen for the implementation unless a later follow-up spec overrides them:

- Prefer extracting modules in-place under `src/` first; only introduce deeper nested directories where they clearly improve ownership.
- Prefer pure helper extraction before event-loop restructuring.
- Prefer preserving current persisted data shapes and serialized field names.
- Prefer behavior-preserving wrappers/adapters over large one-shot rewrites.
- Prefer adding a `theme` module during this refactor even if only the first semantic palette slice migrates immediately.
- Prefer moving tests closer to the modules they validate, while keeping cross-module integration tests at the app level.
