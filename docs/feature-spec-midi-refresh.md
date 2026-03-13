# Feature Spec: MIDI Refresh

## Summary

This feature adds automatic MIDI device refresh and reconnection handling so the app recovers cleanly when USB or virtual MIDI ports disappear and later return.

The intent is to keep the current MIDI-first workflow usable without forcing the user to restart the app or manually rebuild routing after a cable, hub, controller, or synth reconnects.

The recommended V1 shape is:

- detect topology changes through a shared MIDI device refresh path
- preserve logical assignments by device name when ports disappear and reappear
- keep missing routes and mappings explicit instead of silently replacing them with another device
- surface offline/reconnected status on the existing `MIDI I/O`, `Routing`, and `Mappings` workflows without adding a new page

This spec is grounded in the current repository shape:

- `docs/handoff-summary.md` marks robust hot-plug refresh/reconnect UX as not implemented
- `docs/implementation-plan.md` already calls out device refresh and reconnect handling as a planned item
- `src/midi_io.rs` currently scans ports by visible name and preserves selected defaults by matching names
- `src/app.rs` currently syncs MIDI input subscriptions from routing state and mapping-learn state, but does not own a durable reconnect model for missing defaults/routes

## Problem

Today the app can enumerate MIDI devices and preserve currently resolved default selections across a rescan, but the overall reconnect story is incomplete:

- a disconnected device may vanish from the current catalog until the app explicitly rescans
- per-track routing stores a `MidiPortRef` by name, but the UI does not clearly communicate unresolved/offline routes
- mapping device filters also use device names, but there is no refresh-specific UX that explains when a filtered device is temporarily absent
- output connections are dropped on send failure, but recovery depends on the catalog and routing state becoming current again

This causes friction in normal hardware workflows:

- USB devices reconnect with a short outage
- a hub or dock power-cycles
- a synth or controller is restarted during a session
- a virtual MIDI port restarts while the app keeps running

## Goals

- automatically refresh MIDI device lists when devices disconnect or reconnect
- preserve user intent for defaults, per-track routing, and device-specific mappings
- avoid destructive fallback behavior when a missing device cannot be resolved
- reuse the existing action/state model instead of introducing page-specific refresh logic
- keep the behavior understandable on both desktop and touch-driven usage

## Non-Goals

- deep hardware identity beyond what `midir` and the current app model expose
- perfect physical-device disambiguation when the OS presents duplicate port names
- a full background device diagnostics page
- automatic remapping from one device name to a different device name
- redesigning the routing or mappings pages beyond the minimum status/UI needed for reconnect clarity

## User Stories

- As a user with a controller connected over USB, I can unplug and reconnect it and have its track input and mappings start working again without restarting the app.
- As a user with a synth routed as a track output, I can power-cycle it and have playback resume on the same routed port when that port name returns.
- As a user with device-specific mappings, I can see when the mapped device is offline rather than having the app silently bind that mapping to another input.
- As a touch user on a small screen, I can understand that a route is offline and later reconnected without relying on hover-only affordances.

## Current Model To Reuse

The current codebase already provides the right foundations for a V1 reconnect feature:

- `MidiDeviceCatalog::scan()` builds the current input/output catalog from `midir`
- `MidiDeviceCatalog::with_preserved_selection()` already preserves selected defaults by matching device names
- track routing stores `input_port` and `output_port` as `MidiPortRef`
- mapping device filters store a specific `source_device_label` string or `Any MIDI`
- `sync_midi_inputs()` already rebuilds live input subscriptions from track routes, page state, and MIDI learn state

The refresh feature should build on those same concepts rather than inventing a parallel model.

## Proposed UX

### 1. Automatic Refresh Baseline

The app should refresh MIDI device availability automatically while running.

Recommended triggers:

- periodic background rescan at a low fixed cadence suitable for UI-thread polling in the current architecture
- immediate refresh after a MIDI input connection attempt fails
- immediate refresh after a MIDI output send/connect failure indicates a port is gone

This keeps the implementation aligned with the current app loop while still recovering quickly after reconnect.

### 2. Stable Intent, Explicit Availability

When a device disappears, the app should preserve the user's intended binding by name, but mark it unavailable.

That rule applies to:

- default input selection
- default output selection
- per-track input routing
- per-track output routing
- device-specific MIDI mappings

The UI should distinguish:

- `available and selected`
- `available but not selected`
- `offline but still intended`

The app should not silently replace an offline route or mapping with the first available port.

### 3. Reconnect By Name

If a port later reappears with the same visible name:

- defaults should resolve back to that port automatically
- track routes referencing that name should become live again automatically
- device-specific mappings referencing that name should start matching input events again automatically
- MIDI learn should continue to listen to the current live device list, not to stale missing ports

This is the correct V1 behavior because the current repository model is already name-based.

### 4. Visible Offline/Reconnected Status

The `MIDI I/O` page should become the primary status surface for refresh state.

Recommended additions:

- a small header status line such as `Auto refresh on`
- per-row status treatment for `Default`, `Selected`, `Offline`, and `Reconnected`
- a transient status message after refresh events, for example `Digitone output reconnected`

The `Routing` page should show when the current track's routed input/output device name is unresolved.

The `Mappings` page should show when a MIDI mapping targets a specific device name that is not currently available.

## Action Model Reuse

Refresh should flow through one canonical app path.

Recommended shape:

- add a canonical refresh action or reducer entry, for example `AppAction::RefreshMidiDevices`
- allow that same refresh path to be triggered by:
  - background/system events
  - a future manual retry command
  - failure recovery after an I/O error

The important rule is that catalog replacement, route resolution, mapping availability, and input resubscription should happen through one shared state transition.

System-triggered refreshes should not introduce a separate page-local behavior model.

### Manual Retry

Automatic refresh is the main feature, but a manual retry affordance is still useful.

Recommended V1 surface:

- desktop: a keyboard-accessible action and/or small `Rescan` affordance on `MIDI I/O`
- touch: a tappable `Rescan` affordance on `MIDI I/O`

Manual retry must call the same canonical refresh path as the automatic mechanism.

## Scope Behavior

### Global Defaults

Global default input/output selections should behave as preferred assignments, not just indexes into the currently visible catalog.

If the selected default device disappears:

- the preferred device name remains the intended default
- the currently resolved active index becomes empty until that name returns
- the `MIDI I/O` page should show that the default is offline instead of silently moving the default to another device

This is a deliberate change from the current fallback-to-first-port behavior in `with_preserved_selection()` for defaults.

### Per-Track Routing

Per-track routing already stores device names. That should remain the authoritative routing intent.

If a routed input/output device disappears:

- the track keeps the same stored `MidiPortRef` name
- note capture or playback on that route becomes inactive while the device is absent
- the routing field renders an offline state for that name
- reconnecting a port with the same name restores the route automatically

No automatic substitution should occur.

### Device-Specific Mappings

Mappings with `source_device_label = Any MIDI` remain unaffected by reconnect behavior beyond the refreshed live input list.

Mappings that target a specific device name should:

- remain bound to that exact name while offline
- show an offline badge or text treatment on the mappings page
- resume matching events automatically when the same device name reappears

## Conflict And Replacement Rules

### Missing Device

When a previously assigned device is absent:

- preserve the assignment by name
- mark it offline
- do not retarget to another available device

### Same Name Returns

When the same device name returns:

- treat it as the intended replacement for the missing route/default/mapping
- restore connectivity automatically

### Different Name Appears

When a likely replacement appears under a different name:

- do not auto-bind it
- require explicit user selection

### Duplicate Names

If the OS exposes multiple ports with the same visible name, V1 should treat name matching as ambiguous but still deterministic according to the scan order returned by `midir`.

This limitation should be documented in the implementation notes and is acceptable for V1 because the current repository has no stronger device identity model.

### Active Failure During Playback Or Recording

If an output disappears during playback or passthrough:

- the failed send should drop the dead connection
- the app should refresh the catalog
- subsequent sends should retry once the named port resolves again

If an input disappears during recording:

- already-recorded note data in the active take must remain intact
- no synthetic note-off repair logic is required in this slice beyond existing transport/recording behavior
- new input from that route resumes only after the named port is visible again

## Desktop And Touch Differences

Desktop behavior:

- hover-capable surfaces may expose richer status text in the footer or window title
- keyboard users can still inspect the `MIDI I/O` page and optional manual retry affordance

Touch behavior:

- offline/reconnected state must be visible inline without hover
- the manual retry affordance, if present, must be tappable
- reconnect recovery itself must require no gesture once the device returns

The core refresh behavior is identical across input modes. Only the status-discovery affordance differs.

## Data Model Notes

The current catalog model stores selected defaults as resolved indexes. That is not enough to represent `preferred but currently offline`.

Recommended V1 extension:

- store preferred default input/output names separately from resolved current indexes
- continue storing per-track routes by `MidiPortRef` name
- continue storing mapping device filters by string label

This keeps default selection behavior consistent with routing and mappings, which already preserve intent by name.

## Decision Defaults

The following decisions close the initial open questions and should be treated as the default implementation target:

- Refresh cadence estimate: run periodic refresh at about `1 Hz` (`~1000 ms`) on the current UI-thread model, with immediate refresh on MIDI I/O failure signals. If low-end profiling shows this is too expensive, relax to `2 Hz` intervals before changing architecture.
- Status noise policy: keep refresh behavior silent by default and only surface status text when device state actually changes (disconnect/reconnect/offline resolution).
- Missing-default visibility: show explicit offline entries on `MIDI I/O` for preferred default input/output names that are currently unavailable.
- Reconnect warning UX: when playback output was interrupted, show explicit user options instead of a passive notice only.
  - recommended first shape: `Dismiss` and `Send All Notes Off` options in the transient reconnect surface.

## Implementation Notes

Suggested incremental implementation:

1. Add a single refresh helper that rescans devices, resolves preferred defaults, preserves name-based routes, and resyncs inputs.
2. Add app-level refresh scheduling or polling cadence suitable for the current UI-thread architecture.
3. Trigger the same refresh helper from MIDI input/output failure paths.
4. Extend page rendering so missing devices are visible on `MIDI I/O`, `Routing`, and `Mappings`.
5. Add tests for disappearance/reappearance of defaults, routes, and mappings.

Likely code touch points:

- `src/midi_io.rs`: catalog scan/selection model, connection failure behavior, optional refresh metadata
- `src/app.rs`: refresh scheduling, shared refresh reducer/helper, failure-triggered refresh, UI status rendering, input resubscription
- `src/routing.rs`: no major model rewrite expected, but routing availability helpers may belong here or adjacent app helpers
- `src/mapping.rs`: device-availability helpers for specific MIDI source labels
- `src/pages.rs`: optional page-state additions if last-refresh or inline status state is persisted
- `README.md`: update user-facing MIDI I/O behavior notes once implementation lands

## Acceptance Criteria

- While the app is running, disconnecting and reconnecting a MIDI device with the same visible name restores its prior default/route/mapping behavior without requiring an app restart.
- If a selected default input or output disappears, the app preserves that default as an offline preference instead of silently switching to another available device.
- If a track input/output route disappears, the route remains assigned by name and becomes active again automatically when that same name returns.
- If a mapping targets a specific MIDI device name, it remains bound to that name while offline and resumes working when that same name returns.
- `Any MIDI` mappings continue to match any currently available MIDI input after refresh.
- Automatic refresh uses the same canonical state transition as any manual rescan affordance.
- The UI shows offline/unavailable state inline on the relevant pages without relying on hover.
- Disconnect/reconnect handling does not clear recorded track content, mappings, or routing assignments.

## Remaining Questions

- should the reconnect options be rendered as a modal, inline transport chip, or footer action prompt in the first implementation slice
