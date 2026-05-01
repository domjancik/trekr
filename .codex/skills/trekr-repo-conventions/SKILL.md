---
name: trekr-repo-conventions
description: Apply the canonical trekr repository conventions for file placement, module ownership, naming, validation, screenshot hygiene, and refactor discipline. Use when changing or reviewing Rust app modules, page UI, runtime helpers, README/docs, screenshots, or structural cleanup so edits and review judgments stay aligned with the post-Phase-2 modular layout.
---

# Trekr Repo Conventions

Use this skill whenever you edit or review trekr code or docs and need the change or review to fit the established structure.

## Repo contract

- Keep `README.md` aligned with the current runnable app surface when pages, controls, workflows, or launch commands change.
- Keep tracked renderer screenshots current when the main screens change:
  - `artifacts/screenshots/timeline.png`
  - `artifacts/screenshots/mappings.png`
  - `artifacts/screenshots/mappings-overlay.png`
  - `artifacts/screenshots/midi-io.png`
  - `artifacts/screenshots/routing.png`
- Refresh screenshots with:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\capture-ui-screens.ps1 -StateMode demo
```

- Refresh the visual review when layout changes materially:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\review-ui-screens.ps1 -StateMode demo
```

- Do not commit ephemeral state under:
  - `artifacts/archive/`
  - `artifacts/state/`
  - `docs/artifacts/`
  - `scripts/artifacts/`
- Do not remove git submodule references such as `vendor/ableton-link`.

## Canonical code organization

### `src/` root

Use root modules for shared or domain-level code:

- domain/runtime behavior like `mapping.rs`, `midi_fx.rs`, `midi_io.rs`, `project.rs`, `timeline.rs`, `transport.rs`, `routing.rs`
- shared app-wide support like `ui.rs`, `theme.rs`, `present.rs`, `pages.rs`
- platform/runtime integration like `link.rs`, native bridge files, CLI, and top-level wiring

Do not move page-local UI logic into root just for convenience.

### `src/app/`

Use `src/app/` for app-owned orchestration, page behavior, reducers, local runtime helpers, and UI interaction code.

Current families:

- `src/app/mod.rs`: app integration root; keep it as orchestrator, not a dumping ground
- `src/app/mapping/`: `page.rs`, `lookup.rs`, `input.rs`, `ui.rs`
- `src/app/timeline/`: `page.rs`, `layout.rs`, `track_ui.rs`, `recording.rs`, `fx_ui.rs`, `ui.rs`
- `src/app/shell/`: `ui.rs`, `layout.rs`, `scaling.rs`
- `src/app/support/`: lightweight app-only helpers like labels, page actions, I/O helpers, and text/contrast helpers
- top-level app modules for substantial non-family features such as `capture.rs`, `input.rs`, `midi_io_page.rs`, `routing_ui.rs`, `note_runtime.rs`, `stored_loops.rs`, `types.rs`, `direct_mapping_ui.rs`, and `discoverability_ui.rs`

## File placement rules

- Put feature-specific code with its owning family.
- Put reusable geometry/text/layout helpers in `src/ui.rs`.
- Put semantic colors/tokens in `src/theme.rs`.
- Put app-only helper functions in `src/app/support/`.
- Keep tests near the owning module when behavior is feature-local.
- Leave only true cross-feature integration coverage in `src/app/mod.rs`.

Avoid:

- new catch-all helper modules
- growing `src/app/mod.rs` for convenience
- broad folder churn without a clear ownership improvement
- presenter-style architecture unless it is explicitly planned

## Naming conventions

- Prefer short snake_case file names that match responsibility.
- Inside family folders, use names like:
  - `page.rs` for page-level draw orchestration
  - `layout.rs` for geometry/layout structures
  - `ui.rs` for family-local interactions or discoverability
  - `recording.rs`, `fx_ui.rs`, `track_ui.rs`, `lookup.rs`, `input.rs` for narrower slices
- Use `*_ui.rs` only when the file is actually UI-centric rather than domain logic.
- Prefer ownership names over generic names like `helpers.rs`.

## Refactor discipline

When moving code:

- move tests with the behavior instead of deleting them
- preserve action semantics and call paths unless the task explicitly changes behavior
- search for old function names after extraction to ensure there is one owner and no duplicate copy left behind
- keep refactors structural unless the task explicitly asks for behavior changes

## Validation order

For refactors or UI-affecting changes, prefer:

1. `cargo fmt`
2. `cargo check` or `cargo xtask check`
3. targeted `cargo test` for touched modules
4. full `cargo test` when feasible
5. screenshot capture when main screens or renderer output changed
6. screenshot review when layout changed materially

Report validation clearly as:

- focused test status
- full-suite status
- screenshot capture/review status
- known environment-specific failures

## Review expectations

For structural work, success means:

- clearer ownership
- smaller integration surface in `src/app/mod.rs`
- no dropped tests
- no duplicate implementations left behind
- preserved UI behavior and screenshot expectations

## Default decision rules

When unsure:

- prefer the existing family structure over inventing a new top-level bucket
- prefer `src/app/support/` over new vague utility files
- prefer local feature tests over central test accumulation
- prefer small targeted moves over sweeping renames
- prefer preserving current app behavior over speculative cleanup
