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
- active-track highlighting
- a moving playhead
- per-track loop preview

Current controls:

- `Space`: play/stop
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
- `Left` / `Right`: select previous/next track
- `1`-`9`: select track by absolute index
- `Escape`: quit
