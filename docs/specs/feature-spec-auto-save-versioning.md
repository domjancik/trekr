# Feature Spec: Auto-save and Versioned State Files

## Summary

Add automatic state saving around the existing persisted-state flow so `trekr` continuously keeps:

- one stable working state file
- one timestamped version file for each committed auto-save point

This feature extends the current single-file persistence model instead of replacing it. The app already loads a persisted JSON state on launch and writes it back on clean exit through `src/cli.rs` and `src/state.rs`. This spec defines how that flow becomes in-session, action-aware, and recoverable without introducing a full project browser or branching/version-history UI.

## Current Baseline

Current repository behavior:

- `cargo run -- run` uses persisted state from `artifacts/state/last-run.json` when available
- the app falls back to demo state when the persisted file is missing or unreadable
- state is only written on clean interactive exit in persisted mode
- persisted data is `PersistedAppState` in `src/state.rs`
- persistence currently covers project/editor state such as `Project`, page state, mappings, timeline flow, and transport/playhead ticks
- the app already has a footer/status model in `src/app.rs` that can surface lightweight save feedback
- all user inputs already normalize into `AppAction` via `src/actions.rs`

This means auto-save/versioning should be designed as an incremental extension to an existing persisted-state pipeline, not a greenfield save system.

## Goals

- preserve work without requiring a clean quit
- keep the primary persistence target simple: one known working file
- produce timestamped restore points without requiring manual user naming
- reuse the canonical action model for save-triggering behavior where user actions are involved
- make save status understandable from the current timeline-first UI and footer model
- keep behavior consistent across keyboard, MIDI mapping, pointer, touch, and internal triggers
- define deterministic replacement/conflict rules for stable file and version files

## Non-Goals

- full project/file browser UI in this slice
- multi-user collaboration or merge resolution
- cloud sync
- arbitrary manual version labels/comments
- undo/redo replacement
- saving every transient runtime-only value
- background saving for `demo` or `empty` mode without an explicit persisted target

## Core User Model

The user works inside one active persisted session.

That session has two persistence outputs:

1. **Working state file**: the canonical latest state for normal relaunch.
2. **Version file**: an immutable timestamped snapshot created whenever auto-save commits a meaningful change.

On the next app launch in persisted mode:

- the app loads the working state file first
- version files are recovery/history artifacts, not the default launch target

## Terms

- **working file**: the current single-file target already represented by `--state-file`
- **version file**: timestamped snapshot written alongside or under a sibling versions directory
- **dirty state**: serializable app state differs from the last successfully saved working state
- **save trigger**: an event that asks the app to commit dirty state
- **save scope**: which parts of in-memory state are included in serialized persistence

## File Model

### Working File

The existing `--state-file` path remains the source of truth for the active session.

Example:

- working file: `artifacts/state/last-run.json`

### Version File Location

Each working file gets a sibling version directory.

Recommended shape:

- working file: `artifacts/state/last-run.json`
- versions dir: `artifacts/state/last-run.versions/`
- version file: `artifacts/state/last-run.versions/2026-04-13T10-48-22-531Z.json`

This keeps version history obviously tied to one working file while avoiding collisions between different `--state-file` targets.

### Timestamp Format

Use a lexically sortable UTC timestamp in the filename.

Recommended format:

- `YYYY-MM-DDTHH-MM-SS-mmmZ.json`

Use filesystem-safe separators so filenames sort chronologically and remain portable.

## Save Scope

Auto-save and version saves use the same serialized payload as the current persisted-state model unless explicitly excluded.

Included scope should match `PersistedAppState`:

- `project`
- `page_state`
- `timeline_flow`
- `mappings`
- `transport_ticks`
- `playhead_ticks`

Not included unless later promoted into persisted state:

- runtime-only device handles/connections
- queued/runtime-only ephemeral state already excluded by the data model
- hover-only/footer-only UI state
- temporary direct-mapping targeting state unless later made persistable on purpose

Rule: auto-save must serialize the same durable state that clean-exit save would serialize, so relaunch behavior stays predictable.

## Save Triggers

### Trigger Types

Auto-save should support two trigger classes:

1. **Action-driven debounce trigger**
   - fires after a mutating action leaves the project dirty
   - debounced so bursts of edits become one save
2. **Lifecycle trigger**
   - on clean exit, flush pending dirty state immediately

### Mutating Action Rule

Only actions that change persisted state should mark the session dirty.

Examples from the current action model that should dirty state when successful:

- transport settings changes that persist
- loop edits
- track arm/mute/solo/passthrough changes if they are part of persisted project state
- mapping add/remove/edit
- routing changes
- note/clip edits
- page-state changes that are already persisted, such as current page or selection fields

Non-mutating/runtime-only actions should not create version files by themselves.

Examples:

- hover updates
- discoverability overlay hover summaries
- transient direct-mapping targeting hover state
- device scans that do not alter persisted preferences

### Debounce Behavior

Recommended initial behavior:

- start or refresh an auto-save timer after each successful mutating action
- commit after a short quiet period
- use one shared timer for the session rather than per-feature timers

Recommended target: `750 ms` to `2000 ms` quiet period, implementation-tunable.

## Save Pipeline

When an auto-save fires for a dirty session:

1. capture the current `PersistedAppState`
2. serialize once in memory
3. write/update the working file
4. write the timestamped version file
5. mark the session clean only after both writes succeed
6. surface save success or failure in the footer/status area

If implementation needs crash-safety, use temp-file-plus-rename semantics for the working file first.

## Conflict and Replacement Rules

### Working File Replacement

The working file is always replaced by the newest successful save.

Rules:

- latest successful save wins
- partial failure must not corrupt the existing working file
- failed writes leave the session dirty
- a later successful save may replace the stale working file

### Version File Replacement

Version files are append-only in normal operation.

Rules:

- a successful auto-save creates exactly one new version file
- version files are never rewritten in place
- if a generated timestamp collides, append a deterministic numeric suffix rather than overwriting
- if no persisted state changed since the last successful save, do not create a duplicate version file

### Dirty-State Comparison Rule

Version creation should be gated by meaningful serialized-state change, not merely by trigger occurrence.

That means:

- repeated save timers with identical serialized payload create no new version file
- clean exit after an already-saved steady state creates no additional version file

### Failure Rule

If working-file write succeeds but version-file write fails:

- treat the overall save as failed for status purposes
- keep the session dirty
- surface a warning that recovery history is incomplete
- allow the next save attempt to retry both targets

This keeps the feature honest: the spec promises both a stable file and a version file.

## UX Flow

### Normal Flow

1. user edits state through any supported control surface
2. app marks the session dirty
3. footer/status indicates pending auto-save
4. after debounce, app saves working file and version file
5. footer/status briefly confirms auto-save success
6. app returns to normal footer behavior

### On Launch

If the working file exists and loads successfully:

- launch from the working file as today
- no version selection prompt in this slice

If the working file is unreadable but version files exist:

- this slice may continue current fallback-to-demo behavior by default
- implementation should log or surface that recovery candidates exist
- explicit recovery UI is a later enhancement, not required for this spec

### On Exit

If dirty:

- perform an immediate save flush before shutdown finishes

If clean:

- exit without extra writes

## Footer and Status Behavior

The current footer in `src/app.rs` is the right V1 save-feedback surface.

Expected states:

- `Auto-save pending…`
- `Saved <time>` or `Auto-saved`
- `Save failed: <short reason>`

Rules:

- save failure should remain visible longer than normal action feedback
- hover discoverability can still win while active, but save errors should be sticky until replaced by a later save result or dismissal policy
- successful save messaging can be brief and then yield back to the normal footer fallback model

## Action Model Reuse

Auto-save should stay aligned with the canonical action architecture.

Rules:

- keyboard, MIDI, pointer, touch, remote, and internal automation all continue to mutate state through `AppAction`
- dirty marking should happen where actions are applied, not separately inside each input surface
- internal timer-driven save execution may use an internal save request path, but it should still flow through a centralized save coordinator rather than ad hoc page code

Recommended design split:

- user-facing edit actions remain `AppAction`
- save scheduling/execution can be internal app/runtime events rather than public remappable actions

This avoids polluting the user mapping surface with implementation-only debounce mechanics while still reusing the action boundary for mutation detection.

## Interaction Differences: Desktop vs Touch

### Desktop

Desktop users already have dense keyboard and pointer workflows.

Expected desktop behavior:

- no extra confirmation for normal auto-save
- save status appears in footer
- future manual save shortcuts may be added, but are not required for this slice

### Touch

Touch users do not have hover and may rely more on direct visual confirmation.

Expected touch behavior:

- footer messaging must be readable without hover dependency
- save feedback should not require right-click, hover, or keyboard-only affordances
- touch-triggered edits should enter the same dirty/debounce pipeline as keyboard and pointer edits

### Shared Rule

There is no platform-specific save logic split. Only presentation differs.

## State-Mode Behavior

### Persisted Mode

Full feature behavior applies only when the app has a writable persisted target:

- auto-save updates working file
- auto-save creates version files
- clean exit flush still applies

### Demo and Empty Modes

`demo` and `empty` launches should not silently start writing auto-save/version history unless the user explicitly provided a persisted target strategy.

V1 rule:

- `--state-mode demo` and `--state-mode empty` remain non-auto-saving by default
- if later combined with an explicit save/create-project flow, that becomes a separate mode transition and is out of scope here

## Recovery and Retention

### Recovery

This spec only guarantees that timestamped recovery files exist.

Not required in this slice:

- in-app version browser
- restore dialog
- diff viewer

### Retention

Retention should be bounded so auto-save does not grow forever.

Recommended initial policy:

- keep recent versions per working file
- prune oldest versions beyond a configured cap

Recommended starter cap:

- `50` to `200` version files per working file

Pruning should happen only after a new save succeeds.

## Acceptance Criteria

- persisted interactive sessions auto-save after successful persisted-state mutations and a debounce quiet period
- each successful auto-save updates the working file configured by `--state-file`
- each successful auto-save also creates one timestamped version file tied to that working file
- identical serialized state does not create duplicate version files
- auto-save never silently enables itself for `demo` or `empty` mode sessions
- clean exit flushes pending dirty state in persisted mode
- footer/status feedback communicates pending, success, and failure states without requiring hover
- pointer, touch, keyboard, MIDI mapping, and internal action paths all feed the same dirty/save pipeline when they mutate persisted state
- a failed save does not mark the session clean
- version filenames sort chronologically and do not overwrite prior versions

## Likely Code Touch Points

Primary implementation files, based on the current repo shape:

- `src/cli.rs`
  - extend launch behavior beyond clean-exit-only save
  - define when auto-save is enabled from `StateMode` and `state_file`
- `src/state.rs`
  - add working/version path helpers
  - add atomic write helpers and version-file creation
  - add serialized-state equality/hash support if needed for duplicate suppression
- `src/app.rs`
  - track dirty state, last-saved state fingerprint, save timer, and footer status
  - mark dirty after successful mutating actions
  - drive save scheduling and shutdown flush
- `src/actions.rs`
  - no large surface expansion required, but the implementation should rely on the existing canonical action boundary to detect persisted mutations
- `src/project.rs`
  - confirm which track/transport fields are intentionally durable versus runtime-only
- `src/pages.rs`
  - verify which page/selection fields should continue to persist and therefore trigger dirty state
- `README.md`
  - update runtime docs once the feature is implemented so launch/save behavior matches reality
- `docs/specs/product-spec.md`
  - update the broader save/load product language after implementation lands

## Open Questions

- should page-navigation-only changes remain persisted and therefore trigger version creation, or should versioning focus more narrowly on project/mapping edits
- should save failures block quit on desktop, or only warn and continue exit
- should retention be fixed in code first or exposed later as config
- should recovery from a broken working file automatically try the newest valid version file, or remain manual in a later slice
