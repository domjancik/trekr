# Modular Refactor Phase 2 Spec

## Summary

This spec defines a second-pass modularization after the first successful `app.rs` breakup.

Phase 1 reduced the single-file hotspot, preserved behavior, and already introduced initial shell, theme, input, and present-plan seams. Phase 2 should deepen those seams and reduce the size and responsibility breadth of the remaining large page/domain modules without changing UX, interaction rules, state semantics, persisted data shape, or screenshot output.

Required outcome:

- keep current behavior, tests, and tracked screenshots stable
- split the still-large page modules by subdomain, not by arbitrary helper type
- make timeline, mapping, theming, and presenter-related work easier to extend without recreating a new large-file hotspot
- keep the action model, scope behavior, conflict/replacement rules, and touch/desktop interaction rules unchanged

This phase is grounded in the current repository state after the first refactor pass, including:

- `docs/planning/modular-refactor-spec.md`
- `docs/planning/handoff-summary.md`
- `docs/planning/color-refactor-plan.md`
- `README.md`
- `docs/specs/feature-spec-mapping-discoverability.md`
- `docs/specs/direct-ui-mapping-mode-spec.md`
- `docs/specs/feature-spec-midi-track-effects.md`
- `docs/specs/feature-spec-quick-mapping-lookup.md`
- `docs/specs/feature-spec-timeline-control-contexts.md`
- `docs/specs/feature-spec-remarkable-paper-pro-move-eink.md`
- current code in `src/app.rs`, `src/app/shell_ui.rs`, `src/app/input.rs`, `src/app/mapping_ui.rs`, `src/app/io_pages.rs`, `src/app/timeline_ui.rs`, `src/theme.rs`, and `src/present.rs`

## Current Follow-On Problem

The first pass improved top-level ownership and established initial seams for shell UI, input routing, theme tokens, and present planning, but some extracted modules are still large enough to deserve their own internal seams and the new seams are still intentionally shallow.

Current pressure points:

- `src/app/timeline_ui.rs` is still very large and mixes:
  - timeline page shell rendering
  - transport-strip rendering
  - recording-lane rendering and geometry
  - timeline FX row layout and hit testing
  - pointer behavior and discoverability targets
  - timeline-local footer/help content
- `src/app/io_pages.rs` combines MIDI I/O rendering and routing-page rendering, which are related but distinct concerns
- `src/app/mapping_ui.rs` still combines:
  - direct mapping flow
  - discoverability overlay behavior
  - mappings target lookup behavior
  - mapping badges/summary rendering
- `src/app.rs` is much smaller than before but still owns a sizable reducer and integration surface
- `src/theme.rs` now exists, but semantic token adoption is still partial rather than complete
- `src/present.rs` now exists, but the current present-plan seam is intentionally minimal rather than a fuller presenter/runtime family

This is now a codebase maintenance problem, not an urgent correctness problem.

## Goals

- reduce the size of the remaining large page modules
- make ownership inside each page domain obvious from file names alone
- separate render/layout helpers from interaction/hit-testing helpers where doing so improves clarity
- deepen semantic color/theme adoption without changing visuals
- deepen the present/render seam only where it meaningfully improves ownership
- avoid creating circular dependencies between timeline, mapping, routing, theme, and presenter code
- keep test relocation close to the behavior being extracted

## Non-Goals

- redesigning timeline, mappings, routing, or MIDI I/O UX
- changing action names, key bindings, mapping targets, scope rules, or replacement/conflict behavior
- introducing a general widget framework
- combining presenter/runtime work into this pass unless needed only for a narrow shared color or render-type seam
- splitting files purely to satisfy a line-count target with no ownership benefit

## Recommended Module Splits

### 1. Timeline family

Recommended target structure:

- `src/app/timeline_page.rs`
  - timeline page shell
  - transport/header composition
  - high-level track-column orchestration
  - timeline discoverability target collection at page level
- `src/app/timeline_layout.rs`
  - visible track layout calculations
  - column and band geometry helpers
  - subcolumn rect helpers
  - compact row/slot layout helpers
- `src/app/timeline_recording.rs`
  - recording lane layouts
  - clip scrolling helpers
  - recording content drawing
  - clip hit testing and related footer/help content
- `src/app/timeline_fx_ui.rs`
  - FX row layout
  - FX row drawing
  - FX overflow/param/move/delete affordances
  - FX hit testing
  - FX-specific footer/help content
- keep any tiny shared timeline-only utilities either:
  - in `timeline_layout.rs`, or
  - in a very small `timeline_shared.rs` only if needed

Rules:

- page shell should orchestrate; it should not own every rectangle/math helper
- FX row rendering and FX hit testing should stay together unless a future reducer split clearly improves things
- recording-lane geometry and recording-lane interaction should stay close because they are tightly coupled
- discoverability targets should live near the render/hit geometry they describe

### 2. Mapping family

Recommended target structure:

- `src/app/direct_mapping_ui.rs`
  - direct-mapping overlay and status behavior
  - direct-mapping target selection/hit testing
  - source capture display helpers
- `src/app/mapping_lookup.rs`
  - target lookup state transitions
  - query/filter/highlight behavior
  - lookup layout and drawing
- `src/app/discoverability_ui.rs`
  - discoverability summaries
  - badges and overlays
  - hover target summaries and badge rendering
- keep low-level labels in `src/app/labels.rs`

Rules:

- direct mapping must keep reusing canonical actions and target labels
- lookup behavior must keep current scope validation and escape/cancel rules
- discoverability overlays must keep current touch/desktop semantics and must not become a second action registry

### 3. I/O and routing family

Recommended target structure:

- `src/app/midi_io_page.rs`
  - MIDI I/O page rendering and pointer handling
  - device list rendering and hit testing
- `src/app/routing_page.rs`
  - routing page shell
  - routing field layout and rendering
  - routing pointer behavior
  - routing field display-value helpers that are truly page-local

Rules:

- routing and MIDI I/O can still share tiny helpers, but their primary files should reflect distinct page semantics
- routing should remain the owner of routing-field presentation logic, not generic app shell code

### 4. Theme and color seam

Current status:

- `src/theme.rs` exists and already holds an initial semantic token set

Recommended target structure:

- continue growing `src/theme.rs` or split to `src/app/theme.rs` only if ownership becomes clearer
  - semantic color tokens only
- optional follow-on subfiles if justified:
  - `theme/timeline.rs`
  - `theme/mappings.rs`
  - `theme/routing.rs`

Rules:

- use semantic names such as `accent_warning`, `panel_border_selected`, `timeline_loop_active`, not raw color-purpose guesses like `yellow_2`
- Phase 2 should expand semantic token usage across remaining inline colors and style constants, not redesign the palette
- screenshot output must remain unchanged
- newer components should consume the same semantic tokens rather than adding new inline colors by default

## Interaction Model Preservation Requirements

The following must remain true after Phase 2:

- direct mapping still targets canonical actions and preserves current replacement/reuse rules
- mapping lookup still filters and commits targets with the current scope/conflict behavior
- discoverability still reflects the same action model used by keyboard, pointer, and mapping paths
- timeline FX row hit regions still prioritize the same controls and selection behavior
- routing toggle/adjust/set behavior still matches current pointer semantics
- touch and desktop coordinate handling stays unchanged
- presenter/runtime seams remain render-backend-facing, not product-semantics-facing

## Test Strategy

- no existing test may be dropped solely because of Phase 2 modularization
- tests may be relocated, split, or minimally adapted to match new module boundaries
- page-domain tests should move closer to their new modules when that improves clarity
- cross-page integration tests should remain at the app level
- tracked screenshots remain a regression gate and should show no intentional change

Recommended checks for each extraction slice:

- `cargo check`
- targeted domain tests for the extracted area
- periodic full `cargo test`
- screenshot capture and pixel comparison before committing the pass complete

## Acceptance Criteria

Phase 2 is complete when:

- `timeline_ui.rs` has been broken into smaller domain-scoped files with clear ownership
- `mapping_ui.rs` has been broken into direct-mapping, lookup, and discoverability-oriented files
- `io_pages.rs` has been split into clearer page-specific files if that split improves ownership without needless churn
- semantic theme tokens are used broadly enough that new shared chrome and page work no longer default to ad hoc inline colors
- the present-plan seam is either kept intentionally minimal and documented, or expanded into a clearer presenter/runtime family if follow-on work actually requires it
- more page-local tests have moved out of `src/app.rs` where ownership is now clear
- full `cargo test` passes
- tracked screenshots remain unchanged unless an explicit visual change is separately approved
- README only changes if runnable app surface changes, which is not expected for this phase

## Likely Code Touch Points

- `src/app.rs`
- `src/app/shell_ui.rs`
- `src/app/input.rs`
- `src/app/labels.rs`
- `src/app/mapping_ui.rs` or its replacements
- `src/app/io_pages.rs` or its replacements
- `src/app/timeline_ui.rs` or its replacements
- `src/theme.rs`
- `src/present.rs`
- tests in `src/app.rs` and any relocated module-local test blocks

## Suggested Execution Order

1. split `timeline_ui.rs` by recording/layout/FX/page shell
2. split `mapping_ui.rs` by direct mapping / lookup / discoverability
3. split `io_pages.rs` only if the ownership improvement is still worth the churn
4. deepen semantic theme token adoption with zero screenshot drift
5. decide whether the minimal `present.rs` seam should stay minimal or expand for concrete follow-on runtime work
6. relocate more page-local tests and run full regression and screenshot gate

## Confidence Gate

Reuse the same completion gate as Phase 1.

Before declaring Phase 2 done, confidence must reach at least `84.7%` based on:

- Testing: `40%`
- Code review: `30%`
- Logical inspection: `30%`

If confidence is below threshold, report the remaining gaps and minimum next checks instead of declaring completion.
