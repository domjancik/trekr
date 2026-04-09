# Feature Spec: Build Launcher

## Purpose

Define a separate but related launcher app that can:

- discover downloadable builds by branch/channel
- install/update those builds locally
- launch the selected build quickly

The launcher should keep Trekr's visual language and page-based UI path, while staying simpler in interaction density (in the spirit of vvvv gamma launcher).

This spec is grounded in current repo docs/code: `README.md`, `docs/specs/product-spec.md`, `docs/dev/architecture.md`, `docs/planning/implementation-plan.md`, `docs/planning/handoff-summary.md`, `src/actions.rs`, `src/app.rs`, `src/pages.rs`, `src/cli.rs`, and `src/state.rs`.

## Current Baseline (Grounding)

- Trekr already uses a canonical action layer (`AppAction`) across keyboard/pointer/touch/MIDI (`src/actions.rs`).
- The current app shell is page-based (`Timeline`, `Mappings`, `MIDI I/O`, `Routing`) with consistent tab and footer/status behavior (`src/pages.rs`, `src/app.rs`).
- Pointer/touch handling is centralized and page-widget driven (`src/page_widgets.rs`, `src/app.rs`).
- Persistence is JSON-file based with stable serde structs (`src/state.rs`).
- CLI already supports multiple launch modes and a terminal launcher (`src/cli.rs`, `src/bin/trekr-tui.rs`).

Implication: the launcher should reuse this architecture pattern instead of introducing a one-off UI/runtime model.

## Goals

- Provide a clean launcher to run **latest stable** and selected **feature branch** builds.
- Support download/update/install from finalized GitHub release artifacts with explicit status and recovery.
- Reuse Trekr’s interaction model (actions, pages, keyboard + pointer/touch parity).
- Keep install/replace behavior deterministic and safe.

## Non-Goals (V1)

- Requiring local source build toolchains for normal end-user installs.
- Full package manager features (delta patching, rollback graphs, dependency solving).
- Automatic branch discovery from arbitrary remotes without configured source.

## Launcher Information Model

- **Channel**: logical source (e.g., `stable/main`, `feature/<branch>`).
- **Remote Build**: downloadable artifact metadata (version label, commit, build date, URL, checksum).
- **Installed Build**: local extracted build tied to channel + commit.
- **Launch Target**: the build selected to run for a channel.
- **Install Job**: fetch/verify/extract operation with progress and terminal state.

## UX Flow

### App Shell and Navigation

Keep Trekr-style full UI path (tab/page navigation, footer status, compact chips), but launcher-specific pages:

1. **Launch** (default): quick run, current selected build per channel.
2. **Channels**: branch/channel selection and pinning.
3. **Downloads**: remote builds, install/update actions, progress/errors.
4. **Settings**: storage path, cleanup rules, source endpoint config.

Navigation behavior mirrors current Trekr conventions:

- `Tab`/`Shift+Tab` next/previous page
- `F1..F4` direct page jumps
- `Up/Down`, `Q/E`, `Enter` for list/field flow
- pointer/touch can directly target controls

### Primary User Journeys

1. Open launcher → sees stable channel ready state.
2. User selects `feature/my-branch` in Channels.
3. Downloads page shows newest remote build for that branch.
4. User installs it (or updates existing install).
5. Launch page now offers one-tap run for stable or that feature branch.

## Action Model Reuse

Launcher should keep the same action-first boundary:

- inputs (keyboard/pointer/touch/internal download events) resolve to canonical launcher actions
- UI and backend mutate state only through applied actions

Recommended: introduce `LauncherAction` + `LauncherActionEvent` parallel to existing `AppAction`/`ActionEvent`, reusing `ActionSource`.

## Scope Behavior

Launcher actions should use explicit scopes:

- **Global**: app-level (switch page, open settings, refresh all channels).
- **Channel scope**: specific branch channel (`stable/main`, `feature/x`).
- **Install scope**: specific installed build entry (remove, set default, launch).

Rule: actions triggered from a channel card/list row must resolve to that channel, never “currently highlighted” implicitly if a direct hit target exists.

## Conflict / Replacement Rules

### Install Replacement

- Same channel + newer commit => replace channel “active build pointer” to new install.
- Existing install folders are retained until cleanup policy runs (default: keep last 2 per channel).
- User can pin an older install to prevent cleanup.

### Duplicate Build

- If exact commit already installed for channel: operation is idempotent; show “Already installed”.

### Concurrent Jobs

- Starting install for channel with active install job:
  - default action: cancel previous and start newest request (explicit confirmation on touch + desktop)
  - alternative: keep existing job and ignore duplicate trigger if same commit

### Running Process Conflict

- If launching a channel while another build from same channel is running:
  - default: focus existing process if alive
  - optional secondary action: terminate and relaunch

### Corrupt/Partial Install

- Failed checksum or incomplete extract marks install as `Invalid`.
- Invalid install cannot be launched; user gets `Retry` or `Remove`.
- Source-build fallback is opt-in only (`--allow-source-build` or settings toggle), not default.

## Desktop vs Touch Interaction

### Desktop

- Hover exposes richer row status (commit/date/path) and shortcut hints.
- Single click selects; double-click or explicit `Run` launches.
- Keyboard-first workflow remains complete.

### Touch

- No hover dependency: row cards always show core status/actions.
- Hit targets must be larger; destructive actions require confirm sheet.
- Long-press opens row secondary actions (pin/remove/open folder).

### Shared

- Same canonical action results across desktop/touch.
- Progress/error state visible inline and in footer/status.

## Acceptance Criteria

1. Launcher runs as a separate app entry point from Trekr.
2. User can switch between four launcher pages with keyboard and pointer/touch.
3. User can add/select feature branch channels and see latest remote build metadata.
4. User can install latest build for stable/main and for a selected feature branch.
5. User can launch installed stable build and installed feature branch build.
6. Duplicate install requests for same commit are handled idempotently.
7. Replacement behavior for newer same-channel builds follows explicit rules and updates active pointer.
8. Failed download/verify/extract states are visible and recoverable (retry/remove).
9. Desktop and touch flows are both supported without hover-only blockers.
10. Launcher state persists (channels, selected builds, pinned installs, job history summary).
11. Existing Trekr app behavior remains unchanged when running `trekr` directly.
12. The launcher can still invoke Trekr builds that preserve current CLI launch options.

## Likely Code Touch Points

- `src/bin/`:
  - add a separate launcher binary entrypoint (e.g., `trekr-launcher.rs`)
- `src/cli.rs`:
  - optionally add a launcher command handoff and shared launch argument generation
- `src/actions.rs` (or new `src/launcher/actions.rs`):
  - launcher action definitions and keyboard binding resolver
- `src/pages.rs` (or new launcher page-state module):
  - launcher page enum/state (`Launch`, `Channels`, `Downloads`, `Settings`)
- `src/app.rs` / `src/page_widgets.rs`:
  - Trekr-style page rendering and pointer/touch routing patterns reusable for launcher UI
- `src/ui.rs` and `src/app_ui/branding.rs`:
  - shared styling primitives and branding treatment
- `src/state.rs`:
  - launcher persisted state schema/file helpers (parallel to current app state)
- new modules likely needed:
  - `launcher_catalog` (remote build metadata fetch/parse)
  - `launcher_installs` (filesystem layout, cleanup, pinning)
  - `launcher_jobs` (download/verify/extract state machine)
  - `launcher_process` (spawn/focus/terminate launched builds)

## Open Questions

- Source of “latest feature branch builds”: GitHub Releases, CI artifacts, or both?
- Should launcher support authenticated/private artifact endpoints in V1?
- Should “stable/main” always auto-update on startup, or be manual refresh only?
