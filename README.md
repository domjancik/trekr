# trekr

Native MIDI-first tracker/player/looper for small PCs with a portable path to mobile-class targets.

## Docs

- `docs/product-spec.md`: product behavior, UX model, workflows, and MVP scope.
- `docs/architecture.md`: engine architecture, portability constraints, and stack options.
- `docs/implementation-plan.md`: milestone order, module breakdown, and delivery sequence.

## Current Direction

- Primary target: small-form-factor desktop systems.
- Secondary target: iOS/Android if the chosen stack supports it cleanly.
- V1 focus: MIDI sequencing, routing, passthrough, and loop-based recording.
- Audio follows MIDI-first V1 and should layer onto the same timeline and routing model later.
- Chosen implementation stack: Rust with a lightweight native rendering and I/O stack.
- SDL3 is built from source in the current scaffold so local builds do not depend on a preinstalled SDL runtime.

## Current Runnable Slice

`cargo run` opens a native SDL3 window with:

- fixed-fit per-track paired columns in the form `full | detail | full | detail`
- default vertical-time layout with time moving downward
- a page shell for `Timeline`, `Mappings`, `MIDI I/O`, and `Routing`
- real MIDI device enumeration via `midir`
- basic routed MIDI note playback on track output ports/channels
- in-canvas bitmap text labels for pages, tracks, ports, mappings, and routing values
- active-track highlighting
- a moving playhead
- per-track loop preview

Current controls:

- `Tab` / `Shift+Tab`: next/previous page
- `F1` / `F2` / `F3` / `F4`: show timeline, mappings, MIDI I/O, or routing page
- `Up` / `Down`: select current page item
- `Q` / `E`: adjust current page item
- `Enter`: activate/toggle current page item
- `Space`: play/stop
- `Home`: reset the global song loop to the full song range
- `[` / `]`: set current-track loop start/end at playhead
- `,` / `.`: nudge current-track loop backward/forward by one quantize step
- `-` / `=`: shorten/extend current-track loop by one quantize step
- `/` / `\`: half/double current-track loop length
- `Shift+[` / `Shift+]`: set global loop start/end at playhead
- `Shift+,` / `Shift+.`: nudge global loop backward/forward by one quantize step
- `Shift+-` / `Shift+=`: shorten/extend global loop by one quantize step
- `Shift+/` / `Shift+\`: half/double global loop length
- `G`: toggle global loop enable
- `L`: toggle current track loop enable
- `A`: arm current track
- `M`: mute current track
- `S`: solo current track
- `I`: toggle current track passthrough
- `Left` / `Right`: select previous/next track directly
- `1`-`9`: select track by absolute index
- `Escape`: quit

The timeline page also exposes a clickable `Reset Song Loop` button that triggers the same action as `Home`.

The `Mappings` page is currently a non-editable quick overview of all key, MIDI, and OSC bindings.

## UI Review Loop

The repo includes a scripted screenshot-and-review loop for visual QA:

- `scripts/capture-ui-screens.ps1`: asks `trekr` itself to render `timeline`, `mappings`, `midi-io`, and `routing` screenshots into `artifacts/screenshots`
- `scripts/review-ui-screens.ps1`: calls `codex exec` with those screenshots attached and writes findings to `artifacts/reviews/ui-findings.md`
- `scripts/run-ui-review.ps1`: runs both steps in sequence

The capture path is renderer-owned rather than desktop-owned:

- screenshots are exported from the SDL drawing layer
- capture runs against an offscreen software surface, so other desktop apps do not leak into the images

Example:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-ui-review.ps1
```
