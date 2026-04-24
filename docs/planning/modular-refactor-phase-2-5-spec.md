# Modular Refactor Phase 2.5 Structural Polish Spec

## Goal

Apply a light navigability polish after the Phase 2 ownership refactor without changing behavior, introducing presenter architecture, or triggering a broad rename spree.

## Scope

1. Convert the app entry module from `src/app.rs` to `src/app/mod.rs`.
2. Group the stabilized app families into native directory modules:
   - `src/app/mapping/`
   - `src/app/timeline/`
   - `src/app/shell/`
3. Group the lightweight app-only support helpers under `src/app/support/`.
4. Preserve behavior, tests, screenshots, and existing module ownership.

## Non-goals

- No presenter or rendering-plan redesign.
- No app/core split; that remains Phase 3.
- No mass rename churn for symmetry alone.
- No movement of stable shared/core modules out of `src/`.

## Intended shape

- `src/` continues to own shared/core/runtime modules.
- `src/app/` owns the app shell plus page-family modules.
- `src/app/mod.rs` becomes the native app composition root.
- `src/app/mapping/` owns mapping page helpers, lookup, mapping input, and mapping UI helpers.
- `src/app/timeline/` owns timeline page/layout/ui/track/fx/recording modules.
- `src/app/shell/` owns shell UI/layout/scaling modules.
- `src/app/support/` owns lightweight app-only helper modules such as labels, helper rect utilities, IO helpers, and page action dispatch helpers.

## Acceptance criteria

- `src/app.rs` no longer exists; `src/app/mod.rs` builds cleanly.
- Mapping, timeline, and shell families live under directory modules.
- Shared lightweight app helpers live under `src/app/support/`.
- No behavior change.
- Full tests pass, screenshot capture passes, and tracked screenshots remain unchanged.
- No ephemeral artifacts are committed.

## Notes

This is a structural follow-up only. If additional support buckets (`support/`, `common/`) still look useful after this pass, they should be evaluated separately to avoid mixing navigability cleanup with architectural work.

## Merge guidance for older branches

Branches based on the pre-refactor flat app layout should rebase first, then re-home their changes into the new family directories instead of reviving the previous file layout. When resolving conflicts, preserve the feature-owner structure, keep `src/app/mod.rs` thin, and verify that helper/function ownership does not get duplicated across old and new locations.
