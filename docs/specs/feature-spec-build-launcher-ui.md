# Feature Spec: Build Launcher UI (Trekr-Style SDL App)

## Purpose

Define the in-canvas launcher UI that sits on top of the existing launcher backend so users can browse branches, install/update builds, and run them without using terminal commands.

This spec is grounded in current repo docs/code:

- `docs/planning/handoff-summary.md`
- `README.md`
- `docs/specs/product-spec.md`
- `docs/specs/feature-spec-build-launcher.md`
- `src/actions.rs`, `src/app.rs`, `src/pages.rs`, `src/page_widgets.rs`, `src/ui.rs`
- `src/launcher/*` (current CLI launcher backend and state model)

## Current Baseline

- Trekr’s main app already has a page-shell UI with keyboard + pointer/touch parity and a shared action model (`AppAction` + `ActionSource`).
- A separate launcher backend now exists (`src/launcher/*`) with branch listing, release-artifact-first install, optional source fallback, run, and persisted launcher state.
- Missing piece: a native SDL launcher UI that matches Trekr’s style and interaction path.

Implication: UI work should reuse Trekr’s page, widget, and action architecture, while calling into the existing launcher backend modules.

## Goals

- Deliver a native launcher UI in Trekr visual style and navigation pattern.
- Reuse backend modules (`catalog`, `installs`, `process`, `state`) instead of rewriting install/run logic.
- Keep artifact installs as the default user path; source build remains explicitly optional.
- Keep action-first behavior across keyboard, pointer, and touch.
- Make “run latest main” and “switch to feature branch” first-class one/two-tap flows.

## Non-Goals (This Slice)

- Replacing backend install/build behavior (that remains source-based for now).
- Building new renderer framework separate from Trekr’s current UI stack.
- Introducing MIDI mapping pages/features into launcher UI V1.

## UX Flow

### Page Model (Mirrors Trekr Pattern)

Launcher has four pages, with Trekr-style tab/header/footer flow:

1. **Launch** (`F1`)
   - Primary CTA cards for installed channels (default includes `main`).
   - “Run” button and quick argument chips (`Project`, `Window`, `State Mode`).
2. **Branches** (`F2`)
   - Remote branch list + filter.
   - Select branch, mark tracked, set default launch branch.
3. **Installs** (`F3`)
   - Install/update status per tracked branch.
   - Job progress and last result (success/error).
4. **Settings** (`F4`)
   - Repo URL, install root, cleanup policy, and default run arguments.
   - Includes default UI scale, state-file path selection, and source-fallback toggle.

### Navigation & Interaction

Match Trekr conventions:

- `Tab` / `Shift+Tab`: next/previous page
- `F1`..`F4`: direct page
- `Up` / `Down`: row selection
- `Q` / `E`: adjust selected field value
- `Enter`: activate selected action
- pointer/touch: direct hit targets for rows/chips/buttons

### Primary Scenarios

1. Open launcher UI → Launch page shows `main` card with run status.
2. Go to Branches → choose `feature/x`.
3. Go to Installs → trigger install/update for `feature/x`.
4. Return to Launch → run `feature/x` with selected args.

## Action Model Reuse

Introduce launcher UI action layer parallel to `AppAction`:

- `LauncherAction` + `LauncherActionEvent` (+ existing `ActionSource` semantics)
- Raw input resolves into canonical launcher actions first.
- UI and backend mutations occur only via action apply path.

Recommended actions:

- page navigation (`ShowPage`, `ShowNextPage`, `ShowPreviousPage`)
- row/field selection (`SelectPrevItem`, `SelectNextItem`, `AdjustBackward`, `AdjustForward`)
- execution (`InstallBranch`, `RunBranch`, `RefreshBranches`, `SetDefaultBranch`)
- argument edits (`SetRunProject`, `CycleWindowMode`, `CycleStateMode`)
- conflict resolution (`ConfirmReplace`, `CancelPending`)

## Scope Behavior

Launcher action scope must be explicit:

- **Global scope**: page navigation, refresh all, settings edits.
- **Branch scope**: branch-specific actions (track/install/set default).
- **Install scope**: actions against a specific installed artifact.

Rules:

- Pointer/touch activation resolves to clicked item scope, not merely current list selection.
- Keyboard actions operate on focused/selected item in current page context.
- Launch argument edits on Launch page are per-branch unless explicitly marked global default.

## Conflict & Replacement Rules (UI Behavior)

UI must expose existing backend behavior clearly:

1. **Duplicate commit install**
   - Show non-error “Already installed” toast/status.
2. **New commit same branch**
   - Replace active pointer to newest build, retain older build entries per cleanup policy.
3. **Install while job active (same branch)**
   - Prompt: `Cancel and restart` (default) or `Keep running`.
4. **Run with missing install**
   - Offer inline action: `Install now`.
5. **Run while same branch process active**
   - Prompt: `Focus running` (default) or `Relaunch`.
6. **Install error**
   - Show actionable state with `Retry` and `Details`.
7. **Artifact missing**
   - Show `No matching release artifact` with optional `Enable source fallback` path.

## Desktop vs Touch

### Desktop

- Hover reveals richer metadata (commit, path, command preview).
- Footer shows current action hint/status like Trekr.
- Double-click branch row may trigger primary action (`Track`/`Install`) if enabled.

### Touch

- No hover dependency; key status always visible on cards/rows.
- Larger touch targets for run/install chips.
- Confirmation dialogs/sheets for destructive or interrupting actions.

### Shared

- Same canonical action outcomes regardless of input source.
- Error and job status visible both inline and in footer/status bar.

## Acceptance Criteria

1. Launcher opens as native SDL UI (not terminal-only) in Trekr style.
2. Four-page launcher navigation works via keyboard and pointer/touch.
3. User can select a branch and trigger install/update from UI.
4. User can run installed `main` directly from Launch page.
5. User can run selected feature branch from Launch page.
6. User can set run args (`project/state file`, `window mode`, `state mode`, `ui scale`) in UI and those args are used when launching.
7. Artifact install is default; source build fallback requires explicit opt-in in settings/CLI.
8. User can choose state files from `Documents/trekr/artifacts/state` and set a new state file path.
9. User can configure installation directory from settings.
10. Conflict/replacement prompts are shown for concurrent install/run edge cases.
11. Job progress and errors are visible without opening terminal logs.
12. Launcher UI state persists across restarts.
13. Existing Trekr app runtime/UI remains unchanged when launched directly.

## Likely Code Touch Points

- `src/bin/trekr-launcher.rs`
  - switch from CLI-only entrypoint to SDL launcher UI entrypoint (CLI remains as subcommand mode or separate flag)
- New UI modules under `src/launcher/ui/`:
  - `app.rs` (launcher UI root state + event loop integration)
  - `actions.rs` (launcher UI canonical actions)
  - `pages.rs` (page/selection state)
  - `widgets.rs` (page rendering and pointer hit-testing)
- Reuse:
  - `src/ui.rs` for layout/text helpers
  - `src/app_ui/branding.rs` for shared branding treatment
  - `src/launcher/catalog.rs`, `installs.rs`, `process.rs`, `state.rs` backend operations
- Optional shared abstractions:
  - extract common page-shell rendering primitives from `src/app.rs` into a reusable helper module

## Open Questions

- Should launcher UI and CLI share one binary mode flag (`--cli`) or remain separate binaries?
- Should `Branches` page show all remote branches or tracked subset by default?
- Should argument presets be global defaults plus per-branch overrides in V1, or per-branch only?
