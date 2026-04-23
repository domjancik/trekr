# Modular Refactor Phase 3 Spec

## Summary

Phase 3 is a follow-on architecture pass that is intentionally **out of scope for Phase 2**.

Phase 2 focused on reducing `app.rs`, breaking up page/UI hotspots, relocating page-local tests, and deepening theme/module ownership while preserving behavior and screenshots.

Phase 3 should address the next larger architectural question:

- whether `App` should keep owning the full action reducer and orchestration surface, or
- whether the codebase should introduce a clearer **frontend/app vs core/engine** split that can support headless execution and future alternate frontends.

This phase should only begin after the Phase 2 modular refactor is considered stable.

## Why Phase 3 Exists

The current `App` type still acts as the composition root for multiple concerns at once:

- app/page state
- overlay and direct-mapping state
- input routing
- renderer-facing orchestration
- MIDI runtime coordination
- project/transport mutation via `apply_action`

That is acceptable for the current SDL-driven application shape, but it means the system still has a strong coupling between:

- UI concerns
- runtime/device concerns
- domain mutations

If Trekr later needs a headless mode, alternate frontend, automation harness, or separate control surface process, a stricter split will likely be justified.

## Goals

- define a clean boundary between **core domain logic** and **frontend/app orchestration**
- make it possible to run meaningful application behavior without SDL rendering
- reduce `App` from stateful god-object/coordinator into a thinner frontend shell
- isolate deterministic state mutation from device/runtime side effects
- make action handling easier to test without constructing the full interactive app shell
- preserve current behavior, command semantics, mapping rules, and runtime expectations

## Non-Goals

- redesigning UX, pages, controls, or mapping behavior
- changing persisted project shape unless strictly necessary
- rewriting the MIDI engine or transport model from scratch
- introducing network/distributed control in this phase
- replacing the existing app action model in one risky step

## Proposed Architecture Direction

### 1. Split by responsibility, not by technology alone

Recommended top-level direction:

- `core` / `engine` / `session` layer
  - deterministic project/transport/edit state mutation
  - mapping target resolution rules
  - note/timeline/domain transforms
  - pure or mostly-pure reducers/commands
- `app` / `frontend` layer
  - page state
  - overlays
  - discoverability presentation
  - pointer/keyboard routing
  - SDL rendering
  - runtime/device coordination
- `runtime` adapters
  - MIDI input/output runtime
  - Link runtime
  - persistence glue
  - present/display glue

The exact naming may vary, but the boundary should be explicit.

### 2. Separate action families

The current `AppAction` enum mixes:

- domain mutations
- page navigation
- transient interaction modes
- runtime-triggering operations

Phase 3 should evaluate splitting this into clearer families such as:

- **core commands**
  - transport toggle
  - loop/note/track edit operations
  - mapping commit/update operations
- **frontend actions**
  - page changes
  - overlay toggles
  - selection/highlight-only behavior
  - temporary direct-mapping / lookup modes
- **runtime effects/events**
  - MIDI input delivery
  - device refresh requests
  - all-notes-off / monitor-side effects

This does **not** require deleting `AppAction` immediately.

A transitional adapter layer is preferred.

### 3. Move deterministic reducers below the app shell

Candidate logic to move into a core layer over time:

- transport state mutations
- loop edits and quantized loop changes
- note selection / note edit commands
- recording clip operations
- track arm/mute/solo/passthrough semantics where they are domain-owned
- mapping scope validation and mapping commit rules
- timeline FX chain mutations that do not require device/runtime coordination

Candidate logic to keep at app/frontend level:

- pointer hit routing
- page-specific field selection state
- overlays and discoverability UI state
- direct manipulation affordances
- SDL present/layout logic

Candidate logic to keep at runtime adapter level:

- MIDI device enumeration/refresh
- actual note send / all-notes-off I/O
- Link adapter interaction
- environment/persistence bridges

## Headless Mode Implications

A successful Phase 3 should make a future headless mode realistic by allowing:

- project/session creation without SDL window setup
- deterministic command execution without renderer dependencies
- scriptable transport/edit/mapping workflows
- test harnesses that drive the core layer directly

A headless mode does **not** need to be delivered in Phase 3, but the architecture should stop actively preventing it.

## Suggested Intermediate Seams

To reduce risk, prefer staged seams instead of a large rewrite.

### Seam A: command context

Introduce a narrow command context that owns only core mutable state required for deterministic updates.

Example responsibilities:

- project
- transport-related state
- mapping table
- selected track/note/clip ids when those are domain-owned

### Seam B: app view state

Group frontend-only state separately.

Example responsibilities:

- current page
- overlay state
- lookup/direct-mapping transient state
- selected page field
- viewport/UI-scale values

### Seam C: side-effect sink

Instead of performing side effects inline in every reducer path, collect effect intents such as:

- refresh MIDI devices
- send all notes off
- update runtime routing subscriptions
- emit status message

The frontend/runtime shell can then execute them.

## Test Strategy

Phase 3 should add tests at three levels:

- **core reducer tests**
  - deterministic command/state mutation
- **frontend interaction tests**
  - page routing and transient mode behavior
- **runtime adapter tests**
  - device/runtime glue with fakes where possible

Rules:

- do not drop existing behavior tests during the split
- move tests downward when ownership becomes clearer
- preserve screenshot regression gates for frontend-visible behavior
- retain full `cargo test` as the branch-level integration gate

## Migration Strategy

Recommended order:

1. identify the smallest coherent set of core-owned mutations
2. introduce a transitional adapter from `AppAction` to core commands
3. move deterministic tests with that slice
4. introduce side-effect intents where reducer paths currently mix mutation and runtime work
5. repeat per domain slice
6. only after several slices succeed, reevaluate whether `AppAction` should be split formally

## Acceptance Criteria

Phase 3 should be considered complete when:

- a clearly named core/engine boundary exists
- meaningful domain mutations can run without the full SDL app shell
- `App` is primarily frontend orchestration rather than the only mutation owner
- side effects are more explicitly separated from deterministic state mutation
- existing behavior and regression expectations remain intact
- screenshot output remains unchanged unless separately approved

## Risks

- over-splitting too early and creating excessive adapter boilerplate
- moving page-specific selection state into core prematurely
- entangling runtime effects with deterministic reducers during migration
- trying to fully replace `AppAction` in one pass

## Recommendation

Treat this as a deliberate **Phase 3 architecture track** after Phase 2 stabilizes.

Do not fold it into the remaining Phase 2 cleanup.
