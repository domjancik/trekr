# Feature Spec: Mapping Discoverability

## Summary

Users should be able to inspect "what control surfaces trigger this action?" directly from the UI without navigating away to the mappings page.

The feature adds lightweight mapping discoverability to action-bearing UI elements such as transport controls, track state controls, loop controls, routing toggles, and page-level utility actions.

The recommended shape is:

- default behavior: hover an action element to show its current mappings in a status bar
- optional secondary behavior: toggle an inline mappings overlay with its own dedicated shortcut that renders compact mapping badges next to eligible action elements

This keeps the default interface clean while still supporting faster visual lookup when the user explicitly wants persistent mapping hints.

## Problem

The current prototype exposes mappings in two places:

- the dedicated mappings page
- the global quick mappings overlay

Both are useful, but neither answers the question at the moment of interaction:

"What is mapped to this specific button, toggle, or track action right here?"

That gap hurts learnability in three ways:

- a user looking at the timeline or routing page cannot discover related bindings in context
- a controller-heavy workflow still requires page switching or memory recall
- the existing mappings overlay is mapping-centric rather than action-centric

## Goals

- make mappings inspectable from the page where the action lives
- preserve the fixed-fit UI model and avoid turning every page into a dense legend
- support keyboard/mouse-first desktop usage now, while leaving room for touch-friendly invocation later
- represent keyboard, MIDI, and OSC mappings through the same action-centric summary

## Non-Goals

- full in-place mapping editing from arbitrary pages
- conflict resolution UI beyond surfacing that multiple bindings exist
- tooltip-heavy floating windows that obscure the track/timeline content
- replacing the mappings page as the canonical place for broad mapping management

## User Stories

- As a user hovering `Play/Stop`, I can immediately see the keyboard, MIDI, and OSC bindings mapped to that action.
- As a user exploring track controls, I can inspect mappings for `Arm`, `Mute`, `Solo`, `Passthrough`, and loop actions without leaving the timeline.
- As a user learning the app on a hardware controller, I can turn on a temporary overlay that labels actionable UI with their mapped controls.
- As a user on a dense screen, I can leave the overlay off and still get mapping context on demand.

## Proposed UX

### 1. Hover-to-Status Baseline

When the pointer hovers an actionable element, the app shows a concise mapping summary in the status area.

Example format:

`Play/Stop | Keys: Space | MIDI: CC20, Note C2 | OSC: /transport/play | 4 bindings`

Behavior:

- only elements with a known `AppAction` or action family participate
- if no mapping exists, show the action label plus `No mappings`
- if multiple mappings exist, group by source kind instead of listing every detail verbosely
- if the user moves off the element, the status area returns to page-default status text

Recommended initial scope:

- transport strip controls
- timeline header actions such as `Reset Song Loop`
- track state actions visible in timeline columns
- routing page fields and toggles
- mappings page controls only where it helps discover global actions rather than self-referential editor fields

### 2. Optional Inline Overlay

The user can toggle a discoverability overlay that draws compact badges adjacent to actionable elements.

Example badge content:

- `Space`
- `CC20`
- `/transport/play`
- `+3` to indicate additional bindings not expanded inline

Behavior:

- overlay is off by default
- overlay has a dedicated shortcut rather than reusing the existing mappings quick overlay
- overlay is page-local in rendering but action-global in meaning
- badges should prefer the most recognizable bindings first: keyboard, then MIDI, then OSC
- crowded controls may collapse into a count badge rather than rendering every mapping
- hover can still refine the summary in the status area even when overlay is on
- default and user-defined mappings should be visually distinguished by color so user-created bindings read as additive rather than canonical

## Interaction Model

### Status Source

V1 should add a renderer-level footer/status bar inside the app.

The footer should support at least two message types:

- hover-driven mapping summary
- last performed action when no hover-specific message is active

The current window-title status path can remain as a redundant desktop convenience, but it should not be the primary discoverability surface.

### Discoverability Trigger

Desktop V1:

- mouse hover shows status summary
- a dedicated keyboard shortcut enables/disables the inline overlay

Later touch-friendly fallback:

- press-and-hold or selection focus can expose the same mapping summary without relying on hover

### Action Resolution

Each eligible UI element should declare the canonical `AppAction` it represents, or an action descriptor that can resolve to one or more actions.

Examples:

- `Play/Stop` -> `AppAction::TogglePlayback`
- `Track Arm` on the active track column -> action family `Track Arm` with `Active Track` scope
- `Reset Song Loop` -> `AppAction::ResetGlobalLoop`

The discoverability layer should summarize mappings against the same canonical action model already used by keyboard, MIDI, and future OSC bindings.

## Information Rules

- show only currently active mappings from the in-memory mapping list plus built-in keyboard bindings
- hide disabled mappings in the default discoverability surface
- preserve scope information when it matters, such as `Track Arm (Active Track)` versus `Track Arm (Track 3)`
- where an on-screen control implies active-track scope, active-track mappings should be listed first
- conflict state can be summarized as `Overlapping bindings` or `N bindings` instead of full conflict diagnostics

Future note:

- a later mapping-focused overlay mode may surface disabled mappings in a dimmed treatment, but that is out of scope for this slice

## Visual Constraints

- no floating tooltip boxes over the timeline in V1
- status bar text should stay single-line and clipped predictably
- inline badges must not shift layout; they should occupy reserved micro-slots or draw in adjacent dead space
- overlay density must degrade gracefully on narrow track columns
- color is the default visual marker distinguishing built-in versus user-defined mappings

## Implementation Notes

Suggested incremental implementation:

1. Add a small action-discoverability model that maps rendered hit targets to canonical actions.
2. Add mapping-summary helpers that collect matching keyboard, MIDI, and OSC bindings for an action/scope pair.
3. Surface hover state from pointer movement, not only pointer down.
4. Add a renderer footer that can show either hover mapping summaries or last-action feedback.
5. Add an optional discoverability overlay toggle with its own dedicated shortcut once the summary model is stable.

Likely code touch points:

- `src/app.rs`: pointer hit targets, hover state, status rendering, overlay rendering
- `src/actions.rs`: keyboard binding summary access
- `src/mapping.rs`: action-to-binding summary helpers
- `src/pages.rs` or adjacent UI state: discoverability/overlay state if promoted beyond a local app detail

## Acceptance Criteria

- Hovering an eligible action element surfaces a mapping summary without changing pages.
- The summary includes keyboard bindings and any enabled MIDI/OSC mappings targeting the same action.
- Elements with no mapping still surface their action name and an explicit unmapped state.
- Built-in/default mappings and user-defined mappings are visually distinguishable by color.
- Disabled mappings are not shown in the default discoverability footer or overlay.
- The default UI remains readable with discoverability overlay disabled.
- The footer can show the last performed action when there is no hover-specific mapping summary.
- If the overlay is enabled, badges do not break fixed-fit layout on the supported demo viewport.

## Scope Decision: Track Controls

Track-column controls should use active-track scope only in V1.

Rationale:

- the current interaction model is active-track-centric for arm, mute, solo, passthrough, and loop actions
- rendering per-column absolute mapping hints would imply a stronger direct-manipulation contract than the current action model actually guarantees
- active-track-only discoverability is simpler, less ambiguous, and fits the fixed-fit layout better

Absolute per-track bindings still exist in the mapping model, but they should remain discoverable through the mappings page until a later direct mapping mode is designed.

## Follow-On Feature

This discoverability work should be treated as a foundation for a later direct mapping mode:

- enter mapping mode
- select an actionable UI element
- capture the next input
- create or replace a mapping for that element's canonical action

That follow-on flow is intentionally out of scope for this spec and should be designed separately on top of the same action-hit-target model.
